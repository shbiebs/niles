# E30c corpus — schema-level omission

Fourteen cases, each a **defective** script (`d.sql`) and its **correct twin** (`c.sql`), written
for `docs/study/E30c-design.md` and committed with it, **before** the dialect's event trigger,
the dialect script `schema/dialect.sql`, or `nilescheck-sql`'s catalog pass existed. Each script
runs in one transaction after the dialect and `schema/base.sql`.

`cases.tsv` gives each case's omission class, the clause of the design's §3 predicted to refuse
the defective script (`D1`–`D3` at DDL time, `K1`–`K4` at check time), and the baseline
prediction for today's SQL+C+L and stock PostgreSQL. **The predictions are predictions**: an
outcome that differs is reported as differing, and the scripts are not edited after their
outcomes are first read.

S09 and S10 name the dialect's triggers `dialect_conserve` and `dialect_append_only`; those are the
names the dialect is to give them (design §3, D3).
