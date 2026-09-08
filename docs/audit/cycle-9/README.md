# Cycle 9 audit briefs

Two auditors read the same two repositories **independently**, from companion briefs, and each
produces a work order. The author reconciles the two afterwards into the consolidated order that
Claude Opus executes.

| brief | auditor | how it works | territory |
|---|---|---|---|
| [`fable-audit-prompt.md`](fable-audit-prompt.md) | Claude Fable 5.1 | a cloud container with a shell, the toolchain, PostgreSQL and valgrind; reads the Mac through the desktop bridge; asks the author to run Host C scripts | the empirical half, and the re-derivation of everything cycle 8 reported |
| [`astra-audit-prompt.md`](astra-audit-prompt.md) | Astra | no shell: reads GitHub and attached files; asks the author to run named commands and paste the output | the structural half: claims versus code, the type system's obligations against the wire's refusals, the thesis text against the runtime, GBS against Loan IQ and Calypso |

The two briefs share §§1–13 nearly verbatim — the state of the trees, the access rules, the
admissibility table, what cycle 8 changed and what it found, where the budget goes, the
constraints, the open-questions ledger, and the required shape of a work order. They differ in
their opening and in §6.6, which names what each auditor can do that the other cannot.

**Both start from the preflight.** Fable runs `bash docs/audit/cycle-9/preflight.sh <niles>
<gbs>` in its container. Astra asks the author to run it on the Mac and paste the output. The
script measures its tree rather than describing it (cycle 8's T-06 made it so after it misreported
its own pin and test count), never lets rustup attempt a download (the first cycle-9 run hung on
exactly that), and now probes for the benchmark's PostgreSQL on 127.0.0.1:5433, which is what still
blocks `run6.sh` sections B and C.

**The state both briefs are written against**: Niles `5eb0bb2` (`c7/01-durable-rows` =
`c8/08-run6-fixes`), GBS `963e4d9` (`c7/00-adapter` = `c8/01-idem-window-epochs`), both on GitHub.

**On the Astra brief's wording.** The cycle-7 Astra brief was refused by that model's content
classifier before any work was done, and its territory was reassigned. The cycle-9 brief is
written in plain engineering language for that reason — integrity checks rather than adversaries,
altered records rather than forgeries, terminated processes rather than signals — with no change
to what it asks for. If it is refused again, the author should say so and the structural half goes
to a second Fable pass, as in cycle 7.

**Reconciliation.** Where the two work orders disagree, the disagreement is the finding; each
states its evidence class. The consolidated order carries the union of tasks with one cut line,
one branch stack per repository from `5eb0bb2` / `963e4d9` named `c9/*`, and the verbatim
checklist the executor reports against.
