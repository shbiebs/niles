# 1. What the manual covers, and what it does not yet

*The construction manual, chapter 1 (cycle 17, C17-13). The manual builds the system; the thesis
argues the theory. This chapter is the map: `docs/manual/coverage.csv` has one row per mechanism a
builder must understand, across the language, the engine, GBS and the front end, and names the
chapter that explains it, or says `not yet`. A row that names a chapter which does not exist fails
the build, so the map cannot claim more than the manual holds.*

## 1.1 How to read the map

Each row is `area,mechanism,chapter`. The areas are the four parts of the project:

| area | what it covers |
|---|---|
| `language` | annotated PostgreSQL SQL (the dialect), its checker, and Niles as its specification |
| `engine` | the ledger write path and the REV runtime (Nilestream, the thesis's instrument), and the comparisons that place it |
| `gbs` | the core-banking system on PostgreSQL |
| `front end` | the client API and the operator interfaces |

Most rows say `not yet`. That is the honest state: two chapters were written before this one
(the workspace, chapter 2, and the counterexamples, chapter 5), and the decision catalogue is
chapter 6. The map grows by two chapters a round (decision D17-10).

## 1.2 The map, checked

The replay of this chapter runs the test that checks the map and the catalogue:

```sh
cargo test -q -p bank-bench --test manual_catalogue
```

The rows that already have a chapter:

```sh
grep -v ',not yet$' docs/manual/coverage.csv | tail -n +2 | cut -d, -f3 | sort | uniq -c
```

```expected
      1 02-workspace.md
      2 05-counterexamples.md
      4 06-decisions.md
```
