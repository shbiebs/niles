# The PostgreSQL wire protocol, over a real socket

Transcript from a raw wire client against `nilestreamd`, reproducing with:

```sh
cargo run -p nilestream-server --bin nilestreamd -- --port 5433
psql -h 127.0.0.1 -p 5433 -U anyone bank
```

The adoption argument of §6.9 — that a database nobody can connect to with the tools they
already have is a database nobody adopts — is an argument until something answers a socket.

```
startup replies: R S S S S S K Z          # auth ok, 5 parameters, key data, ready

>>> select nilestream_frontier
   columns: ['frontier', 'session_anchor']
   row:     ['1', '1']

>>> select acct, sum(amt) from postings where acct = 7 group by acct
   columns: ['key', 'value', 'anchor']
   row:     ['7', '700', '1']

>>> select acct, sum(amt) from postings where acct in (1, 2, 99999) group by acct
   row:     ['1', '100', '1']
   row:     ['2', '200', '1']
   row:     ['99999', None, '1']

>>> select acct, sum(amt) from postings group by acct
   notice:  no key predicate: this engine serves a partially materialized view per key,
            so an unkeyed scan would defeat the mechanism being measured.

>>> select nope from no_such_table
   ERROR:   [NL0500] this query has no lowering

>>> begin
   notice:  this session's transaction state is advisory: a sealed ledger epoch has no
            rollback, and reads are served at a session anchor rather than from a
            snapshot held open
```

## The four lines that carry design decisions

**`row: ['7', '700', '1']` — the anchor is a column.** Every answer carries the epoch it is
true at, on the wire, as data rather than as a footnote. "The same question, asked twice,
answered consistently" is a checkable statement only if the client can see which moment
each answer belongs to.

**`row: ['99999', None, '1']` — a missing key is NULL, not zero.** The absence lattice's
central distinction survives to the client. A view with no entry for a key is not a view
whose balance is zero, and a wire format that conflated them would undo the property the
whole engine maintains. (There are three absences in this system — SQL null, the empty
string, and an evicted hole — and the codec keeps all three apart.)

**`ERROR: [NL0500]` — the compiler's own diagnostic code reaches the client.** There is no
compatibility layer with its own execution path: the client's SQL is parsed as Niles's SQL
surface, lowered to the same IR, verified by the same verifier, and served from the same
runtime. Mapping a conservation or rung error onto PostgreSQL's `42601 syntax_error` would
tell the user the one thing that is certainly false about their program.

**The `begin` notice — the server says what it does not provide.** A sealed ledger epoch has
no rollback. A server that accepted `BEGIN` silently would be letting a client believe in a
transaction boundary that does not exist.

## What is not built, and why each

* **The extended query protocol** (`Parse`/`Bind`/`Execute`) is refused with `0A000
  feature_not_supported` and a stated reason: a prepared statement must be cached against
  the epoch it was planned at, because a plan valid at one visibility frontier need not be
  valid at another. That design question is open, and shipping a version that ignored it
  would be worse than not shipping one.
* **TLS and real authentication.** `SSLRequest` is answered with an explicit refusal rather
  than silence, because a client that gets no answer hangs.
* **The binary format.** Text only; binary is an unmeasured optimisation here.
* **The MySQL wire protocol.** Specified in §7.3, not built.

Every unimplemented feature returns a named error. A server that silently ignored a `Bind`
would return results for a query the client did not ask for.
