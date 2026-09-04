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


def parse_status(text: str) -> list[dict]:
    """Read `thesis/status.toml`.

    A five-key subset of TOML, parsed in twenty lines rather than by taking a dependency.
    The thesis build already refuses to depend on anything it does not need, and a parser
    for `[[claim]]` blocks of `key = "value"` is smaller than the argument for adding one.
    """
    claims, cur = [], None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line == "[[claim]]":
            cur = {}
            claims.append(cur)
            continue
        if cur is None or "=" not in line:
            continue
        k, _, v = line.partition("=")
        v = v.strip()
        if v.startswith('"') and v.endswith('"'):
            v = v[1:-1]
        cur[k.strip()] = v
    for c in claims:
        for k in ("id", "claim", "status", "where", "instrument"):
            if k not in c:
                raise SystemExit(f"status.toml: claim {c.get('id', '?')} has no `{k}`")
        if c["status"] != "measured" and c["status"] != "proved" and not c["instrument"]:
            raise SystemExit(
                f"status.toml: {c['id']} is `{c['status']}` and names no missing instrument. "
                "A hypothesis with no runner is a hypothesis with no status."
            )
    return claims


def status_table(text: str) -> str:
    """One row per claim: what it is, where it stands, and what is missing."""
    rows = ["| Claim | Status | Reported in | What is missing |", "|---|---|---|---|"]
    for c in parse_status(text):
        missing = c["instrument"] or "—"
        rows.append(
            f"| **{c['id']}** — {c['claim']} | **{c['status']}** | {c['where']} | {missing} |"
        )
    return "\n".join(rows)


def status_summary(text: str) -> str:
    """The one-paragraph statement, counted from the same file the table is."""
    claims = parse_status(text)
    n = {}
    for c in claims:
        n[c["status"]] = n.get(c["status"], 0) + 1
    hyp = [c for c in claims if c["id"].startswith("H-")]
    unrun = [c["id"] for c in hyp if c["status"] == "not measured"]
    parts = ", ".join(f"{v} {k}" for k, v in sorted(n.items()))
    return (
        f"Of the {len(claims)} claims this thesis makes — {len(hyp)} hypotheses and "
        f"{len(claims) - len(hyp)} contributions — {parts}. "
        f"**{len(unrun)} hypotheses have no runner at all** "
        f"({', '.join(unrun)}), and for each of them the table in §1.9 names the instrument "
        f"that does not exist rather than the result that is pending. "
        "Measurements have been taken and some of them refuted the claim they were testing; "
        "the sentence this replaces read \"no measurements have been taken yet\" and stood "
        "while §9.1 opened with \"measurements that were actually taken\"."
    )


def status_row(text: str) -> str:
    """The verification table's last row: the aggregate, in the table's own shape.

    Rendered with its own header, because it was not. The block emitted a single `|`-row into
    §3.15 with nothing above it, so a bare row appeared under the paragraph introducing it and
    a reader saw three unlabelled cells. It is the *last row of a table*, and a one-row table
    still needs to say what its columns are.

    The vocabulary grew this cycle: `specified` for a design nothing implements, and `argued`
    for a claim no instrument will settle. Both used to be filed under "not measured", which
    promised a measurement that was never coming. Anything the tally does not recognise is
    counted into `other` and named, so a new status cannot silently vanish from the total.
    """
    claims = parse_status(text)
    proved = sum(1 for c in claims if c["status"] == "proved")
    measured = sum(1 for c in claims if c["status"] in ("measured", "partly measured"))
    unrun = sum(1 for c in claims if c["status"] == "not measured")
    refuted = sum(1 for c in claims if c["status"] == "refuted")
    specified = sum(1 for c in claims if c["status"] == "specified")
    argued = sum(1 for c in claims if c["status"] == "argued")
    known = {"proved", "measured", "partly measured", "not measured", "refuted",
             "specified", "argued"}
    other = [c["id"] for c in claims if c["status"] not in known]
    tally = (
        f"{proved} proved, {measured} measured or partly measured, {specified} specified, "
        f"{argued} argued, {refuted} refuted, {unrun} not measured"
    )
    if other:
        tally += f", {len(other)} with an unrecognised status ({', '.join(other)})"
    total = len(claims)
    return (
        "| Result | Status |\n"
        "|---|---|\n"
        f"| **All {total} claims** | {tally}. Source: `thesis/status.toml`, rendered here "
        "and into §1.9 |"
    )


def keyword_column(text: str, heading: str) -> str:
    """The `Keyword` column of the table under `heading`, as one backticked line.

    Appendix B.3 carries three word lists that were maintained by hand beside a registry that
    already generates `docs/keywords.md`. Two copies of a vocabulary is two vocabularies, and
    a word added to the lexer and not to the appendix is a word the normative grammar does not
    have.
    """
    idx = text.find(heading)
    if idx < 0:
        raise SystemExit(f"heading {heading!r} not found in the keyword reference")
    words = []
    for line in text[idx:].splitlines()[1:]:
        if line.startswith("## "):
            break
        if not line.startswith("| `"):
            continue
        w = line.split("|")[1].strip().strip("`")
        if w:
            words.append(w)
    if not words:
        raise SystemExit(f"no keywords under {heading!r}")
    return " ".join(f"`{w}`" for w in words)


def reserved_list(text: str) -> str:
    """The normative reserved list, with its count, from the generated reference."""
    idx = text.find("## The normative reserved list")
    if idx < 0:
        raise SystemExit("no reserved list in the keyword reference")
    body = text[idx:]
    count = body.splitlines()[2].strip()
    start = body.find("```")
    end = body.find("```", start + 3)
    if start < 0 or end < 0:
        raise SystemExit("the reserved list is not a fenced block")
    words = body[start + 3 : end].strip()
    return f"{count}\n\n```\n{words}\n```"


def policy_table(text: str) -> str:
    """Table 9.8, computed from `results/e6_policies.csv`.

    The table was typed beside the run that produced it and drifted in six cells: the
    medians it printed for `random` and `cost_aware` were not the medians of the file, and
    the prose read 31% and 4.1% where the file says 28.6% and 2.8%. Computing it here means
    the number in the thesis is the number in the CSV or the build fails.

    Median of an even-length column is not defined here because every column has five seeds;
    if that ever changes, the lower of the two middle values is taken, which is stated so
    that a reader can reproduce the arithmetic.
    """
    rows = [ln.split(",") for ln in text.strip().splitlines()[1:] if ln.strip()]
    by: dict[tuple[str, str], list[tuple[float, float, float]]] = {}
    for r in rows:
        policy, st = r[0], r[1]
        by.setdefault((st, policy), []).append((float(r[3]), float(r[4]), float(r[5])))

    def med(xs: list[float]) -> float:
        xs = sorted(xs)
        return xs[(len(xs) - 1) // 2]

    label = {"random": "random", "lru": "LRU", "cost_aware": "cost-aware"}
    out = [
        "| service_time | policy | misses (median) | base rows read (median) | aggregate delay (median) |",
        "|---|---|---|---|---|",
    ]
    for st in sorted({k[0] for k in by}, key=int):
        for policy in ("random", "lru", "cost_aware"):
            vals = by.get((st, policy))
            if not vals:
                continue
            m = med([v[0] for v in vals])
            rr = med([v[1] for v in vals])
            d = med([v[2] for v in vals])
            delay = "—" if st == "0" else f"{d:,.0f}"
            out.append(f"| {st} | {label[policy]} | {m:,.0f} | {rr:,.0f} | {delay} |")
    return "\n".join(out)


def policy_deltas(text: str) -> str:
    """The two comparisons the prose of §9.4.2 makes, as one generated line."""
    rows = [ln.split(",") for ln in text.strip().splitlines()[1:] if ln.strip()]
    by: dict[tuple[str, str], list[tuple[float, float, float]]] = {}
    for r in rows:
        by.setdefault((r[1], r[0]), []).append((float(r[3]), float(r[4]), float(r[5])))

    def med(xs: list[float]) -> float:
        xs = sorted(xs)
        return xs[(len(xs) - 1) // 2]

    lru_rows = med([v[1] for v in by[("0", "lru")]])
    ca_rows = med([v[1] for v in by[("0", "cost_aware")]])
    lru_delay = med([v[2] for v in by[("4", "lru")]])
    ca_delay = med([v[2] for v in by[("4", "cost_aware")]])
    rows_pc = (lru_rows - ca_rows) / lru_rows * 100
    delay_pc = (lru_delay - ca_delay) / lru_delay * 100
    return (
        f"Cost-aware against LRU, from `results/e6_policies.csv`: "
        f"**{rows_pc:.1f}% fewer base rows** at service_time 0 "
        f"({lru_rows:,.0f} → {ca_rows:,.0f}), and "
        f"**{delay_pc:.1f}% less aggregate delay** at service_time 4 "
        f"({lru_delay:,.0f} → {ca_delay:,.0f})."
    )


EXTRACTORS = {
    "contract": lambda t: first_table(t),
    "kwsql": lambda t: keyword_column(t, "## SQL-derived keywords"),
    "kwrust": lambda t: keyword_column(t, "## Rust-derived keywords"),
    "kwnovel": lambda t: keyword_column(t, "## Novel Niles keywords"),
    "kwreserved": reserved_list,
    "curve": lambda t: table_after(t, "### The curve"),
    # E23's two slope tables and its verdict table, by the headings the harness writes.
    "basemedians": lambda t: table_after(t, "## Along the base"),
    "baseslopes": lambda t: table_after(t, "### The slopes, per row of base"),
    "outputmedians": lambda t: table_after(t, "## Along the answer"),
    "outputslopes": lambda t: table_after(t, "### The slopes, per row of answer"),
    "asymptotic": lambda t: table_after(t, "## The two asymptotic contract rows"),
    "table": lambda t: table_after(t, "### The table"),
    "statustable": status_table,
    "statussummary": status_summary,
    "statusrow": status_row,
    "verbatim": lambda t: t.strip(),
    "policytable": policy_table,
    "policydeltas": policy_deltas,
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
