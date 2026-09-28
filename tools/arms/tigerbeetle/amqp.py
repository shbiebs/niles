"""A minimal AMQP 0-9-1 client: enough to declare an exchange and a queue, bind them, and
consume from the queue. Standard library only.

Written for arm T of E27 (cycle 14, R2-03) because the only AMQP client for Python on PyPI
that the round would otherwise need is a download, and this is the whole of what T needs:
connection negotiation with SASL PLAIN, one channel, `exchange.declare`, `queue.declare`,
`queue.bind`, `basic.consume` with no-ack, and the deliveries. Framing and method ids are
from the AMQP 0-9-1 specification (sections 2.3 and 4.2) and RabbitMQ's errata; anything
else the server sends is an error, not something skipped.
"""

import socket
import struct

FRAME_METHOD, FRAME_HEADER, FRAME_BODY, FRAME_HEARTBEAT = 1, 2, 3, 8
FRAME_END = 0xCE


class AmqpError(Exception):
    pass


def _shortstr(s):
    b = s.encode()
    return struct.pack("B", len(b)) + b


def _longstr(b):
    return struct.pack(">I", len(b)) + b


def _table(d):
    body = b""
    for k, v in d.items():
        body += _shortstr(k)
        if isinstance(v, bool):
            body += b"t" + struct.pack("B", int(v))
        else:
            body += b"S" + _longstr(str(v).encode())
    return _longstr(body)


class _Reader:
    def __init__(self, b):
        self.b, self.i = b, 0

    def take(self, n):
        if self.i + n > len(self.b):
            raise AmqpError("truncated method frame")
        s = self.b[self.i:self.i + n]
        self.i += n
        return s

    def u8(self):
        return self.take(1)[0]

    def u16(self):
        return struct.unpack(">H", self.take(2))[0]

    def u32(self):
        return struct.unpack(">I", self.take(4))[0]

    def u64(self):
        return struct.unpack(">Q", self.take(8))[0]

    def shortstr(self):
        return self.take(self.u8()).decode()

    def longstr(self):
        return self.take(self.u32())


class Connection:
    def __init__(self, host, port, user, password, vhost="/"):
        self.sock = socket.create_connection((host, port))
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self.buf = b""
        self.sock.sendall(b"AMQP\x00\x00\x09\x01")
        # connection.start (10, 10) -> start-ok with PLAIN
        self._expect(0, 10, 10)
        response = b"\x00" + user.encode() + b"\x00" + password.encode()
        self._method(0, 10, 11, _table({"product": "niles-e27-arm-t"}) + _shortstr("PLAIN")
                     + _longstr(response) + _shortstr("en_US"))
        # connection.tune (10, 30) -> tune-ok, no heartbeats (a local, short-lived link)
        r = self._expect(0, 10, 30)
        channel_max, frame_max = r.u16(), r.u32()
        self.frame_max = frame_max or 131072
        self._method(0, 10, 31, struct.pack(">HIH", channel_max, self.frame_max, 0))
        # connection.open (10, 40) -> open-ok (10, 41)
        self._method(0, 10, 40, _shortstr(vhost) + _shortstr("") + b"\x00")
        self._expect(0, 10, 41)
        # channel.open (20, 10) -> open-ok (20, 11)
        self._method(1, 20, 10, _shortstr(""))
        self._expect(1, 20, 11)

    # -- framing ------------------------------------------------------------------------
    def _send_frame(self, kind, channel, payload):
        self.sock.sendall(struct.pack(">BHI", kind, channel, len(payload)) + payload
                          + bytes([FRAME_END]))

    def _method(self, channel, cls, mth, args):
        self._send_frame(FRAME_METHOD, channel, struct.pack(">HH", cls, mth) + args)

    def _recv_exact(self, n):
        while len(self.buf) < n:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise AmqpError("connection closed by the broker")
            self.buf += chunk
        out, self.buf = self.buf[:n], self.buf[n:]
        return out

    def _frame(self):
        kind, channel, size = struct.unpack(">BHI", self._recv_exact(7))
        payload = self._recv_exact(size)
        if self._recv_exact(1)[0] != FRAME_END:
            raise AmqpError("bad frame end")
        return kind, channel, payload

    def _expect(self, channel, cls, mth):
        while True:
            kind, ch, payload = self._frame()
            if kind == FRAME_HEARTBEAT:
                continue
            if kind != FRAME_METHOD:
                raise AmqpError(f"expected method {cls}.{mth}, got frame type {kind}")
            r = _Reader(payload)
            c, m = r.u16(), r.u16()
            if (c, m) in ((10, 50), (20, 40)):
                code, text = r.u16(), r.shortstr()
                raise AmqpError(f"broker closed the {'connection' if c == 10 else 'channel'}: {code} {text}")
            if (ch, c, m) != (channel, cls, mth):
                raise AmqpError(f"expected {cls}.{mth} on {channel}, got {c}.{m} on {ch}")
            return r

    # -- the four operations T needs ------------------------------------------------------
    def exchange_declare(self, name, kind="fanout"):
        self._method(1, 40, 10, struct.pack(">H", 0) + _shortstr(name) + _shortstr(kind)
                     + b"\x02" + _table({}))  # durable
        self._expect(1, 40, 11)

    def queue_declare(self, name):
        self._method(1, 50, 10, struct.pack(">H", 0) + _shortstr(name) + b"\x02" + _table({}))
        self._expect(1, 50, 11)

    def queue_bind(self, queue, exchange, key=""):
        self._method(1, 50, 20, struct.pack(">H", 0) + _shortstr(queue) + _shortstr(exchange)
                     + _shortstr(key) + b"\x00" + _table({}))
        self._expect(1, 50, 21)

    def queue_purge(self, name):
        self._method(1, 50, 30, struct.pack(">H", 0) + _shortstr(name) + b"\x00")
        self._expect(1, 50, 31)

    def consume(self, queue):
        """Start a no-ack consumer and yield message bodies (bytes) as they arrive."""
        self._method(1, 60, 20, struct.pack(">H", 0) + _shortstr(queue) + _shortstr("")
                     + b"\x02" + _table({}))  # no-ack
        self._expect(1, 60, 21)
        while True:
            kind, ch, payload = self._frame()
            if kind == FRAME_HEARTBEAT:
                continue
            r = _Reader(payload)
            if kind != FRAME_METHOD:
                raise AmqpError(f"unexpected frame type {kind} outside a delivery")
            c, m = r.u16(), r.u16()
            if (c, m) != (60, 60):
                if (c, m) in ((10, 50), (20, 40)):
                    code, text = r.u16(), r.shortstr()
                    raise AmqpError(f"broker closed: {code} {text}")
                raise AmqpError(f"unexpected method {c}.{m} while consuming")
            # basic.deliver, then a content header, then body frames until body_size.
            kind, ch, payload = self._frame()
            if kind != FRAME_HEADER:
                raise AmqpError("delivery without a content header")
            h = _Reader(payload)
            h.u16(); h.u16()
            size = h.u64()
            body = b""
            while len(body) < size:
                kind, ch, payload = self._frame()
                if kind != FRAME_BODY:
                    raise AmqpError("short delivery body")
                body += payload
            yield body

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass
