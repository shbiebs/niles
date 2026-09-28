"""Arm T of E27 (cycle 14, R2-03): TigerBeetle as the ledger floor, its change-data capture
through RabbitMQ, and a line protocol the REV sidecar (`crates/rev-sidecar`, `--ledger
driver:…`) uses as its ledger.

Lives outside both workspaces, as the round-2 order requires, and uses the official Python
client (`tigerbeetle` 0.17.9 from PyPI, approved 2026-09-28) and a minimal AMQP 0-9-1 consumer
written here (`amqp.py`). Standard library otherwise.

The mapping, stated once:

* a key `(acct, cur)` is TigerBeetle account `acct * 16 + cur + 1` on ledger `cur + 1`, code
  1, flag `history` (so its balance can be read as of a timestamp);
* a two-leg transaction is one TigerBeetle transfer — debit the paying account, credit the
  receiving one — with `user_data_128` the transaction id, `user_data_64` the ledger epoch
  (the oracle's index) and `user_data_32` the value day plus 2^31 (days can be negative);
* an epoch's timestamp is TigerBeetle's timestamp of its last transfer: from the create
  results of each loaded batch, and from the CDC message for every later write;
* an upquery for a key at epoch `e` is the key's balance at that timestamp
  (`get_account_balances`, reversed, limit 1): an index lookup, one row, not a fold.

The line protocol (one request per line, one reply, `ERR <why>` on failure):

    LOAD <e> <n>  + n lines "<txn> <from> <to> <cur> <amt> <day>"   -> OK
    CDC                                          -> OK <timestamp of the last loaded epoch>
    WRITE <e> <txn> <from> <to> <cur> <amt> <day> -> OK
    UP <acct> <cur> <e>                          -> OK <value> <rows>
    SUBSCRIBE                                    -> a stream: "E <e>", "D <acct> <cur> <delta>"…, "."
    LEGS                                         -> "L <acct> <cur> <amt>"…, "END"
    HEAD                                         -> OK <e>
"""

import argparse
import bisect
import json
import os
import socketserver
import sys
import threading

import tigerbeetle as tb

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import amqp  # noqa: E402

BATCH_MAX = 8189
# Value days can be before the ledger's day 0 (back-valued postings); user_data_32 is unsigned.
DAY_OFFSET = 1 << 31


def account_id(acct, cur):
    return acct * 16 + cur + 1


def key_of(account):
    return (account - 1) // 16, (account - 1) % 16


class Ledger:
    def __init__(self, args):
        self.args = args
        self.client = tb.ClientSync(cluster_id=args.cluster, replica_addresses=args.tb)
        self.lock = threading.Lock()
        self.accounts = set()
        self.epochs = []      # sorted epochs with a known timestamp
        self.ts = {}          # epoch -> timestamp of its last transfer
        self.subscribers = []
        self.sub_lock = threading.Lock()

    # -- accounts and transfers ---------------------------------------------------------
    def ensure_accounts(self, keys):
        new = []
        with self.lock:
            for k in keys:
                if k not in self.accounts:
                    self.accounts.add(k)
                    new.append(k)
        for i in range(0, len(new), BATCH_MAX):
            chunk = new[i:i + BATCH_MAX]
            res = self.client.create_accounts([
                tb.Account(id=account_id(a, c), ledger=c + 1, code=1,
                           flags=tb.AccountFlags.HISTORY)
                for (a, c) in chunk
            ])
            ok = (tb.CreateAccountStatus.CREATED, tb.CreateAccountStatus.EXISTS)
            errs = [r for r in res if r.status not in ok]
            if errs:
                raise RuntimeError(f"create_accounts: {errs[:3]}")

    def transfers(self, e, rows):
        keys = set()
        out = []
        for (txn, frm, to, cur, amt, day) in rows:
            keys.add((frm, cur))
            keys.add((to, cur))
            out.append(tb.Transfer(id=txn, debit_account_id=account_id(frm, cur),
                                   credit_account_id=account_id(to, cur), amount=amt,
                                   user_data_128=txn, user_data_64=e,
                                   user_data_32=day + DAY_OFFSET,
                                   ledger=cur + 1, code=1))
        self.ensure_accounts(sorted(keys))
        return out

    def record(self, e, ts):
        with self.lock:
            if e not in self.ts:
                bisect.insort(self.epochs, e)
            self.ts[e] = max(ts, self.ts.get(e, 0))

    def ts_at(self, e):
        with self.lock:
            i = bisect.bisect_right(self.epochs, e)
            return self.ts[self.epochs[i - 1]] if i else 0

    def load(self, e, rows):
        res = self.client.create_transfers(self.transfers(e, rows))
        errs = [r for r in res if r.status != tb.CreateTransferStatus.CREATED]
        if errs or len(res) != len(rows):
            raise RuntimeError(f"create_transfers (load, epoch {e}): {errs[:3]}")
        # 0.17's results carry each event's commit timestamp; the batch's last is the epoch's.
        self.record(e, max(r.timestamp for r in res))

    def write(self, e, row):
        res = self.client.create_transfers(self.transfers(e, [row]))
        errs = [r for r in res if r.status != tb.CreateTransferStatus.CREATED]
        if errs or len(res) != 1:
            raise RuntimeError(f"create_transfers (epoch {e}): {errs}")

    def upquery(self, acct, cur, e):
        ts = self.ts_at(e)
        if ts == 0:
            return 0, 0
        f = tb.AccountFilter(account_id=account_id(acct, cur), user_data_128=0, user_data_64=0,
                             user_data_32=0, code=0, timestamp_min=0, timestamp_max=ts, limit=1,
                             flags=tb.AccountFilterFlags.DEBITS | tb.AccountFilterFlags.CREDITS
                             | tb.AccountFilterFlags.REVERSED)
        b = self.client.get_account_balances(f)
        if not b:
            return 0, 0
        return b[0].credits_posted - b[0].debits_posted, 1

    def all_transfers(self):
        for ledger in (1, 2):
            tmin = 0
            while True:
                q = tb.QueryFilter(user_data_128=0, user_data_64=0, user_data_32=0,
                                   ledger=ledger, code=0, timestamp_min=tmin, timestamp_max=0,
                                   limit=BATCH_MAX, flags=tb.QueryFilterFlags.NONE)
                page = self.client.query_transfers(q)
                yield from page
                if len(page) < BATCH_MAX:
                    break
                tmin = page[-1].timestamp + 1

    def head(self):
        best = 0
        for ledger in (1, 2):
            q = tb.QueryFilter(user_data_128=0, user_data_64=0, user_data_32=0, ledger=ledger,
                               code=0, timestamp_min=0, timestamp_max=0, limit=1,
                               flags=tb.QueryFilterFlags.REVERSED)
            page = self.client.query_transfers(q)
            if page:
                best = max(best, page[0].user_data_64)
        return best

    # -- change-data capture --------------------------------------------------------------
    def start_cdc(self):
        host, port = self.args.amqp.rsplit(":", 1)
        with open(self.args.amqp_password_file) as f:
            password = f.read().strip()
        setup = amqp.Connection(host, int(port), self.args.amqp_user, password)
        setup.exchange_declare(self.args.exchange)
        setup.queue_declare(self.args.queue)
        setup.queue_purge(self.args.queue)
        setup.queue_bind(self.args.queue, self.args.exchange)
        setup.close()
        consumer = amqp.Connection(host, int(port), self.args.amqp_user, password)
        del password
        threading.Thread(target=self.consume, args=(consumer,), daemon=True).start()
        with self.lock:
            last = self.epochs[-1] if self.epochs else 0
        return self.ts_at(last)

    def consume(self, conn):
        try:
            for body in conn.consume(self.args.queue):
                m = json.loads(body)
                t = m["transfer"]
                e = int(t["user_data_64"])
                amt = int(t["amount"])
                # The epoch's timestamp is recorded before its deltas are forwarded, so an
                # upquery at an epoch the sidecar has published always has a timestamp.
                self.record(e, int(t["timestamp"]))
                da, dc = key_of(int(m["debit_account"]["id"]))
                ca, cc = key_of(int(m["credit_account"]["id"]))
                msg = f"E {e}\nD {da} {dc} {-amt}\nD {ca} {cc} {amt}\n.\n".encode()
                with self.sub_lock:
                    for s in list(self.subscribers):
                        try:
                            s.sendall(msg)
                        except OSError:
                            self.subscribers.remove(s)
        except Exception as e:  # the stream is the view's only source: say so and stop
            print(f"driver: CDC consumer failed: {e}", file=sys.stderr, flush=True)
            os._exit(3)


class Handler(socketserver.StreamRequestHandler):
    def handle(self):
        led = self.server.ledger
        while True:
            line = self.rfile.readline()
            if not line:
                return
            parts = line.decode().split()
            if not parts:
                continue
            try:
                cmd = parts[0]
                if cmd == "UP":
                    v, n = led.upquery(int(parts[1]), int(parts[2]), int(parts[3]))
                    reply = f"OK {v} {n}"
                elif cmd == "WRITE":
                    e, txn, frm, to, cur, amt, day = map(int, parts[1:8])
                    led.write(e, (txn, frm, to, cur, amt, day))
                    reply = "OK"
                elif cmd == "LOAD":
                    e, n = int(parts[1]), int(parts[2])
                    rows = [tuple(map(int, self.rfile.readline().split())) for _ in range(n)]
                    led.load(e, rows)
                    reply = "OK"
                elif cmd == "CDC":
                    reply = f"OK {led.start_cdc()}"
                elif cmd == "HEAD":
                    reply = f"OK {led.head()}"
                elif cmd == "LEGS":
                    out = []
                    for t in led.all_transfers():
                        da, dc = key_of(t.debit_account_id)
                        ca, cc = key_of(t.credit_account_id)
                        out.append(f"L {da} {dc} {-t.amount}\nL {ca} {cc} {t.amount}\n")
                    self.wfile.write(("".join(out) + "END\n").encode())
                    continue
                elif cmd == "SUBSCRIBE":
                    with led.sub_lock:
                        led.subscribers.append(self.connection)
                    threading.Event().wait()  # the consumer thread writes to this socket
                    return
                else:
                    reply = f"ERR unknown command {cmd}"
            except Exception as e:
                reply = "ERR " + str(e).replace("\n", " ")
            self.wfile.write((reply + "\n").encode())


class Server(socketserver.ThreadingTCPServer):
    daemon_threads = True
    allow_reuse_address = True


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--tb", required=True)
    p.add_argument("--cluster", type=int, default=0)
    p.add_argument("--listen", required=True)
    p.add_argument("--amqp", default="127.0.0.1:5672")
    p.add_argument("--amqp-user", default="cdc")
    p.add_argument("--amqp-password-file", required=True)
    p.add_argument("--exchange", default="tb-cdc")
    p.add_argument("--queue", default="tb-cdc-view")
    args = p.parse_args()
    host, port = args.listen.rsplit(":", 1)
    srv = Server((host, int(port)), Handler)
    srv.ledger = Ledger(args)
    print(f"driver ready on {args.listen}", flush=True)
    srv.serve_forever()


if __name__ == "__main__":
    main()
