# Arm T — TigerBeetle as the ledger floor (E27, cycle 14 R2-03)

Outside both Cargo workspaces, as the round-2 order requires. Nothing here is linked into
Nilestream or GBS.

| file | what it is |
|---|---|
| `driver.py` | TigerBeetle through the **official Python client** (`tigerbeetle` 0.17.9, PyPI, Apache-2.0). It loads and writes transfers, consumes the change stream from RabbitMQ, and serves the line protocol that `rev-sidecar --ledger driver:…` uses as its ledger. The protocol is in the file's docstring. |
| `amqp.py` | A minimal AMQP 0-9-1 consumer, standard library only. It covers exchange and queue declare, bind, purge and a no-ack consume: the whole of what T needs, so that no further download is needed. |

What runs, and where it comes from (approved per file on 2026-09-28; see
`claude/cycle-14-r2-03-downloads.md` in the project):

- TigerBeetle 0.17.9, `tigerbeetle-x86_64-linux.zip`, unpacked to `/opt/arms/tigerbeetle`
  (override with `TIGERBEETLE_BIN`). One replica, Direct I/O, `--cache-grid=256MiB`.
- The Python client is in a venv at `/opt/arms/tbvenv` (override with `TB_PYTHON`).
- RabbitMQ 3.12.1 and Erlang/OTP 25.3 come from the Ubuntu 24.04 archive. The broker listens
  on loopback only. It runs as its own node, `e27t@localhost`, with its data under the
  comparator's scratch directory.

The comparator's `TArm` (`crates/comparator/src/t.rs`) starts, loads and stops all of it. The
broker password is generated on the machine, kept in an untracked mode-600 file under the
scratch directory, and never printed. `tigerbeetle amqp` takes it only as a command-line flag.
