#!/usr/bin/env python3
"""Generate Appendix D's component map and public-API surface from the workspace.

Why this exists
---------------

Appendix D described an engine API in the present tense. Almost none of it existed: every
type and method it listed — `LedgerHandle::append_batch`, `ReadModel::serve`, `StorageTier` —
was absent from `crates/`, and three of the eleven crates in its component map were stubs
with no caller, two of which have since been deleted. An appendix is the last place a reader
looks before believing something is built, and this one was a design sketch wearing a
reference manual's typography.

So the component map and the API listing are *derived from the crates*. A crate that does not
exist cannot appear; a method that does not exist cannot be listed. What stays hand-written
is the prose that says what a component is *for*, keyed by crate name — because that is a
statement about intent, which no amount of reading the source produces.

The extraction is deliberately shallow: `pub fn`, `pub struct`, `pub enum` and `pub trait` at
the top level of each file, with the signature line as written. A full rustdoc dump would be
more complete and less readable, and completeness is not the property that was missing.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# What each crate is *for*. Hand-written, because a purpose is not derivable; keyed by crate
# name, so a crate with no entry here is reported rather than silently described as nothing.
ROLE = {
    "nilestream-ledger": "Epoch segments, sequencer, hash chain, durability, admission and commit rules",
    "nilestream-core": "REV runtime: resident maps, anchor indices, apply loop, upqueries, contracts",
    "nilestream-optimizer": "Plan-time mode selection and the eviction policies (the adaptive optimizer of §4.6 is specified and not built)",
    "nilestream-consensus": "A single-process, deterministic simulator for replication and cross-shard commit. No sockets, no clock",
    "niles-ir": "Typed IR: circuit types, verifier, reference interpreter, upquery paths",
    "niles-lang": "Stage-0 compiler: lexer, parser, type/effect checker, lowering; SQL surface",
    "niles-interp": "The imperative-subset interpreter `nilesc run` drives, and the ledger it posts to",
    "nilesc": "The compiler driver: `check`, `verify`, `run`",
    "nilestream": "The engine binary: sweep and serve",
    "nilestream-server": "Daemon: sessions, PostgreSQL wire surface, conformance",
    "conservation-suite": "Reference oracle and the conservation property tests",
    "bank-bench": "NilesBank generator, wall-clock harness, thesis drift tests",
    "experiments": "The E-series measurement harness",
    "proto-engine": "The research prototype the counted-work experiments run on",
    "counterproposal": "The differential defect corpus against PostgreSQL",
}

SIG = re.compile(r"^(pub (?:fn|struct|enum|trait|type|const) [^{;=]*)")


def crates() -> list[str]:
    out = []
    for p in sorted((ROOT / "crates").iterdir()):
        if (p / "Cargo.toml").exists():
            out.append(p.name)
    return out


def code_mask(text: str) -> str:
    """`text` with comments and string literals blanked out, character for character.

    Offsets are preserved, so a match found here can be used against the original. This
    exists because the extraction below has to tell an *attribute* from the same characters
    written inside prose or inside a string, and no amount of `str.find` can.
    """
    out = []
    i, n = 0, len(text)
    depth = 0  # nested `/* */`
    while i < n:
        two = text[i : i + 2]
        if depth > 0:
            if two == "/*":
                depth += 1
                out.append("  ")
                i += 2
                continue
            if two == "*/":
                depth -= 1
                out.append("  ")
                i += 2
                continue
            out.append("\n" if text[i] == "\n" else " ")
            i += 1
            continue
        if two == "/*":
            depth = 1
            out.append("  ")
            i += 2
            continue
        if two == "//":
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
            continue
        # Raw strings: r"...", r#"..."#, r##"..."##
        if text[i] == "r" and i + 1 < n and text[i + 1] in '"#':
            j = i + 1
            hashes = 0
            while j < n and text[j] == "#":
                hashes += 1
                j += 1
            if j < n and text[j] == '"':
                close = '"' + "#" * hashes
                k = text.find(close, j + 1)
                k = n if k < 0 else k + len(close)
                out.append("".join("\n" if c == "\n" else " " for c in text[i:k]))
                i = k
                continue
        if text[i] == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            out.append("".join("\n" if c == "\n" else " " for c in text[i:j]))
            i = j
            continue
        if text[i] == "'":
            # A char literal or a lifetime. Only the literal form can hide characters, and
            # it is short; a lifetime is left as itself, which is harmless.
            if i + 2 < n and (text[i + 2] == "'" or (text[i + 1] == "\\" and i + 3 < n)):
                j = text.find("'", i + 1)
                j = n if j < 0 else j + 1
                out.append(" " * (j - i))
                i = j
                continue
        out.append(text[i])
        i += 1
    return "".join(out)


def test_spans(text: str, code: str) -> list[tuple[int, int]]:
    """The `[start, end)` ranges covered by `#[cfg(test)]` items.

    **Why this is not `text.find("#[cfg(test)]")`.** It was, and it truncated each file at the
    first *occurrence of those characters anywhere* — including inside a doc comment.
    `niles-ir/src/eval.rs` explains in its module documentation why the evaluator is public
    "rather than living in a `#[cfg(test)]` block", and that sentence deleted `ZSet`,
    `eval_scalar` and every other public item in the file from Appendix D's count (A10-13).

    Two more shapes it got wrong even where the attribute was real. A test module in the
    *middle* of a file truncated everything after it, so legitimate API declared below it
    vanished; and a `#[cfg(test)]` on a non-module item swallowed the rest of the file the
    same way. Each attribute now covers exactly its own item: to the matching brace for a
    block, or to the terminating semicolon for anything else.
    """
    spans = []
    at = 0
    while True:
        at = code.find("#[cfg(test)]", at)
        if at < 0:
            return spans
        # Where does this item end? The first `{` or `;` decides which shape it is.
        brace = code.find("{", at)
        semi = code.find(";", at)
        if semi >= 0 and (brace < 0 or semi < brace):
            spans.append((at, semi + 1))
            at = semi + 1
            continue
        if brace < 0:
            spans.append((at, len(text)))
            return spans
        depth, i = 0, brace
        while i < len(code):
            if code[i] == "{":
                depth += 1
            elif code[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        end = min(i + 1, len(text))
        spans.append((at, end))
        at = end


def public_items(crate: str) -> list[tuple[str, str]]:
    """(file, signature) for every top-level public item, in file order.

    A public item inside a `#[cfg(test)]` item is not part of the crate's surface and is
    excluded; a public item that merely *follows* one is, and used not to be.
    """
    items = []
    src = ROOT / "crates" / crate / "src"
    if not src.exists():
        return items
    for f in sorted(src.rglob("*.rs")):
        text = f.read_text()
        code = code_mask(text)
        spans = test_spans(text, code)
        offset = 0
        for line in text.splitlines(keepends=True):
            start = offset
            offset += len(line)
            if not line.startswith("pub "):
                continue  # top level only: an indented `pub fn` is inside an impl or module
            if any(a <= start < b for a, b in spans):
                continue
            m = SIG.match(line.strip())
            if m:
                items.append((str(f.relative_to(ROOT)), m.group(1).strip()))
    return items


SELF_TEST_CASES = [
    (
        "a doc comment that names the attribute does not cut the file",
        """//! This module is public rather than living in a `#[cfg(test)]` block.
pub struct ZSet {}
pub fn eval_scalar() {}
""",
        ["pub struct ZSet", "pub fn eval_scalar()"],
    ),
    (
        "a test module in the middle does not delete the API below it",
        """pub fn before() {}
#[cfg(test)]
mod tests {
    pub fn helper_that_is_not_api() {}
    fn nested() { if true { } }
}
pub struct After {}
""",
        ["pub fn before()", "pub struct After"],
    ),
    (
        "a raw string containing the attribute does not cut the file",
        'pub fn scanner() {}\npub const TRAP: &str = r#"#[cfg(test)] mod tests { }"#;\npub struct AfterTheRawString {}\n',
        ["pub fn scanner()", "pub struct AfterTheRawString"],
    ),
    (
        "a `#[cfg(test)]` on a non-module item covers that item only",
        """#[cfg(test)]
pub use std::fmt::Debug as TestOnly;
pub fn still_api() {}
""",
        ["pub fn still_api()"],
    ),
]

# Names that must NOT reach the appendix: a helper declared inside a `#[cfg(test)]` module,
# and an item the attribute is applied to directly. `TRAP` is deliberately *not* here — it is
# a real public const whose value happens to contain the attribute's characters, and the
# whole point of that case is that it survives.
SELF_TEST_EXCLUDED = ["helper_that_is_not_api", "TestOnly"]


def self_test() -> int:
    """Every trap that made this script wrong, as an input it must get right.

    Run by `make check` and by the workspace's drift suite. It exists because the failure
    mode here is silent: a truncated file produces a *smaller* appendix, and an appendix that
    is missing eighteen public items looks exactly like an appendix that is correct.
    """
    import tempfile

    global ROOT
    bad = 0
    for name, source, expected in SELF_TEST_CASES:
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            src = root / "crates" / "probe" / "src"
            src.mkdir(parents=True)
            (root / "crates" / "probe" / "Cargo.toml").write_text("[package]")
            (src / "lib.rs").write_text(source)
            keep, ROOT = ROOT, root
            try:
                got = [sig for _, sig in public_items("probe")]
            finally:
                ROOT = keep
        missing = [e for e in expected if not any(g.startswith(e) for g in got)]
        leaked = [x for x in SELF_TEST_EXCLUDED if any(x in g for g in got)]
        if missing or leaked:
            bad = 1
            print("  FAILED: " + name, file=sys.stderr)
            if missing:
                print("    missing: " + repr(missing), file=sys.stderr)
            if leaked:
                print("    should not be API: " + repr(leaked), file=sys.stderr)
            print("    got: " + repr(got), file=sys.stderr)
        else:
            print("  ok: " + name, file=sys.stderr)
    if bad:
        print(
            "the API extractor does not survive the traps that made it wrong; the appendix "
            "it generates would be short by an unknown amount",
            file=sys.stderr,
        )
    return bad


def render() -> str:
    out = [
        "*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*",
        "",
        "| Crate | Role | public items |",
        "|---|---|--:|",
    ]
    missing = []
    for c in crates():
        role = ROLE.get(c)
        if role is None:
            missing.append(c)
            role = "**no role recorded** — add one to `gen-appendix-d.py`"
        out.append(f"| `{c}` | {role} | {len(public_items(c))} |")
    if missing:
        print(f"warning: no role recorded for {missing}", file=sys.stderr)
    return "\n".join(out)


def api() -> str:
    """The public surface of the three crates the thesis's chapters name by method."""
    out = [
        "*Generated from the workspace by `thesis/gen-appendix-d.py`. Do not edit by hand.*",
        "",
    ]
    for c in ["nilestream-ledger", "nilestream-core", "niles-ir"]:
        out.append(f"**`{c}`**")
        out.append("")
        out.append("```rust")
        seen = set()
        for _, sig in public_items(c):
            if sig in seen:
                continue
            seen.add(sig)
            out.append(sig)
        out.append("```")
        out.append("")
    return "\n".join(out).rstrip()


if __name__ == "__main__":
    what = sys.argv[1] if len(sys.argv) > 1 else "map"
    if what == "--self-test":
        sys.exit(self_test())
    print(render() if what == "map" else api())
