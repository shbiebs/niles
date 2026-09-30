# E30b′ adversarial corpus

Fourteen cases, each a **defective** program (`d.*`) and its **correct twin** (`c.*`), over the
seven classes the work order names (`docs/study/E30b-design.md` §4.2). `cases.tsv` gives each
case's class, the schema it is checked with (`r1` is E30's `corpus/schema/schema.sql`; `a01` and
`a02` are the variants in `../schema/`; `e30` is E30's Niles schema), the arguments its correct
twins are run with, and, where Niles cannot write a twin, why.

`*.sql` files use representation R1. `*.r2.sql` files, for A11 and A12 only, use R2
(`../schema/r2.sql`). `*.nl.niles` files are Niles.

Committed before any SQL+C+L addition or niles-interp addition was built (design §4.2), so no
addition is tuned to these programs. After their verdicts are first read, they are not edited.
