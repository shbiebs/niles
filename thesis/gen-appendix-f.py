#!/usr/bin/env python3
"""Extract Appendix F.2's semantic core from `crates/conservation-suite/src/oracle.rs`.

The appendix says it presents "the one component of the system that is implemented and
passing tests today". It presented a paraphrase: the fold's signature had drifted from the
oracle's, and it declared a `Scale` type nothing in it used. Reproducing the region between
the `BEGIN:appendix-f` / `END:appendix-f` markers is what makes the sentence true, and
`crates/conservation-suite/tests/appendix_f.rs` fails if the appendix and the source differ.
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crates/conservation-suite/src/oracle.rs"


def excerpt() -> str:
    text = SRC.read_text()
    a = text.index("// BEGIN:appendix-f")
    a = text.index("\n", a) + 1
    b = text.index("    // END:appendix-f")
    lines = text[a:b].rstrip().splitlines()
    # Drop the marker's own explanatory comment block, which is about the extraction rather
    # than about the oracle.
    while lines and lines[0].strip().startswith("//") and not lines[0].strip().startswith("///"):
        lines.pop(0)
    return "\n".join(lines)


if __name__ == "__main__":
    print("*Extracted from `crates/conservation-suite/src/oracle.rs`. Do not edit by hand.*")
    print()
    print("```rust")
    print(excerpt())
    print("```")
