#!/usr/bin/env python3
"""Copy generated result tables into the thesis, between markers.

Why this exists
---------------

Every number in `results/` is produced by a harness and written to a file. Every number in
the thesis used to be typed in beside it. Two copies of a measurement is two measurements,
and the second one goes stale silently: nothing fails, the table simply stops describing
the run it names.

So a block of the thesis that reproduces a generated table is delimited by markers and
filled in from the source file by this script. `build.sh` runs it before pandoc, and
`crates/bank-bench/tests/thesis_drift.rs` fails the build if a block is out of date — so
the drift is caught by the test suite rather than by a reader.

Markers, in the thesis markdown:

    <!-- BEGIN:E16-contract results/E16-wallclock.md#contract -->
    ...generated, do not edit...
    <!-- END:E16-contract -->

The fragment after `#` names a section extractor defined in EXTRACTORS below. An extractor
is a function from the results file's text to the block to insert; keeping them named and
few is deliberate, because a general "copy lines 12-19" mechanism would break the moment a
results file grew a paragraph.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def first_table(text: str) -> str:
    """The first markdown table in the file."""
    lines = text.splitlines()
    out, seen = [], False
    for ln in lines:
        if ln.startswith("|"):
            seen = True
            out.append(ln)
        elif seen:
            break
    if not out:
        raise SystemExit("no table found")
    return "\n".join(out)


def table_after(text: str, heading: str) -> str:
    """The first table following a heading line containing `heading`."""
    idx = text.find(heading)
    if idx < 0:
        raise SystemExit(f"heading {heading!r} not found")
    return first_table(text[idx:])


EXTRACTORS = {
    "contract": lambda t: first_table(t),
    "curve": lambda t: table_after(t, "### The curve"),
}

MARKER = re.compile(
    r"(?P<begin><!-- BEGIN:(?P<name>[\w.-]+) (?P<src>[^\s#]+)#(?P<kind>\w+) -->\n)"
    r"(?P<body>.*?)"
    r"(?P<end><!-- END:(?P=name) -->)",
    re.S,
)


def render(doc: str) -> str:
    def one(m: re.Match) -> str:
        src = ROOT / m.group("src")
        if not src.exists():
            raise SystemExit(f"{src} does not exist; run the harness that generates it")
        kind = m.group("kind")
        if kind not in EXTRACTORS:
            raise SystemExit(f"unknown extractor {kind!r}; known: {sorted(EXTRACTORS)}")
        body = EXTRACTORS[kind](src.read_text())
        note = f"*Generated from `{m.group('src')}`. Do not edit by hand.*"
        return f"{m.group('begin')}\n{note}\n\n{body}\n\n{m.group('end')}"

    return MARKER.sub(one, doc)


def main() -> int:
    check = "--check" in sys.argv
    stale = []
    for path in sorted((ROOT / "thesis").glob("*.md")):
        doc = path.read_text()
        if "<!-- BEGIN:" not in doc:
            continue
        new = render(doc)
        if new == doc:
            continue
        if check:
            stale.append(path.name)
        else:
            path.write_text(new)
            print(f"updated {path.name}")
    if stale:
        print("stale generated blocks: " + ", ".join(stale), file=sys.stderr)
        print("run thesis/include-results.py to refresh them", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
