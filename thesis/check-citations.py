#!/usr/bin/env python3
"""Check that every author-year citation in the thesis resolves to a reference entry.

Why this exists
---------------

`references.md` is a numbered IEEE bibliography. The body cites author-year —
`[Jung et al., POPL '18]`, `[Green et al., PODS '07]` — and nothing connected the two, so a
reader had no route from a claim to its source and no way to notice that five works cited in
the body had no entry at all. One entry had been corrupted by an automated fix: a test that
rewrites bare hypothesis identifiers had turned F1 Lightning into "H-F1 Lightning".

The body's style is not changed. Rewriting a hundred sites into numbers is a large mechanical
edit whose failure mode — a citation pointing at the wrong paper — is worse than the problem
it fixes. What this does instead is *verify the correspondence*: for every author-year
citation, an entry whose author list contains that surname and whose year matches must exist.
A citation that resolves is one a reader can follow.

Run: `python3 thesis/check-citations.py`. Exit 0 means every citation resolves.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
THESIS = ROOT / "thesis"

# Bracketed text that is not a citation: markdown link syntax, type names in prose, and the
# IEEE "[Online]" marker inside the bibliography itself.
NOT_A_CITATION = {
    "Online",
    "Row",
    "Key",
    "Val",
    "sketch",
    "stated",
    "withdrawn",
    "static conservation",
    "currency",
    "authorization",
    "contract",
}

CITE = re.compile(r"\[([A-Z][^\]\[]{2,90})\]")
# A year is either four digits, or two preceded by an apostrophe (`POPL '18`). Two bare
# digits are not a year: `[Budiu et al., Thm 2.20]` is a pointer to a theorem.
YEAR = re.compile(r"\b((?:19|20)\d{2})\b|'(\d{2})\b")
SURNAME = re.compile(r"^([A-Z][A-Za-zÀ-ÿ'’\-]+)")


def four_digit(y: str) -> str:
    if len(y) == 4:
        return y
    n = int(y)
    return f"20{y}" if n < 60 else f"19{y}"


def entries() -> list[str]:
    text = (THESIS / "references.md").read_text()
    out, cur = [], ""
    for line in text.splitlines():
        if line.startswith("["):
            if cur:
                out.append(cur)
            cur = line
        elif cur and line.strip():
            cur += " " + line.strip()
        elif cur:
            out.append(cur)
            cur = ""
    if cur:
        out.append(cur)
    return out


def main() -> int:
    refs = entries()
    unresolved = []
    seen = set()
    for path in sorted(THESIS.glob("*.md")):
        if path.name == "references.md":
            continue
        for raw in CITE.findall(path.read_text()):
            cite = raw.strip()
            if cite in NOT_A_CITATION or cite in seen:
                continue
            m = SURNAME.match(cite)
            ys = [a or b for a, b in YEAR.findall(cite)]
            if not m or not ys:
                # Not of the form "Surname ... year": a prose bracket, not a citation.
                continue
            seen.add(cite)
            surname = m.group(1)
            # An all-capitals leading token is a venue, not an author: `[TOPLAS 1990]` and
            # `[PVLDB 2015]` are pointers to a paper named elsewhere in the sentence.
            if surname.isupper():
                continue
            years = {four_digit(y) for y in ys}
            # A one-year tolerance, because a conference's name and its proceedings'
            # publication year legitimately differ: CRYPTO '91 was published in 1992, and
            # PVLDB volume 8 is the 2015 issue of a 2014 paper. Requiring exact agreement
            # would report correct citations as broken, which teaches a reader to ignore
            # this check.
            wanted = set()
            for y in years:
                n = int(y)
                wanted |= {str(n - 1), str(n), str(n + 1)}
            hit = any(surname in r and any(y in r for y in wanted) for r in refs)
            if not hit:
                unresolved.append((path.name, cite, surname, sorted(years)))

    if unresolved:
        print(f"{len(unresolved)} citation(s) resolve to no entry in references.md:\n")
        for f, cite, surname, years in unresolved:
            print(f"  {f}: [{cite}]  (looking for {surname} + {'/'.join(years)})")
        print(
            "\nEither the entry is missing, or the body's spelling of the author or year "
            "does not match it. A citation a reader cannot follow is decoration."
        )
        return 1
    print(f"{len(seen)} author-year citations, all resolved against {len(refs)} entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
