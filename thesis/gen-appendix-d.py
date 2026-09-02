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


def public_items(crate: str) -> list[tuple[str, str]]:
    """(file, signature) for every top-level public item, in file order."""
    items = []
    src = ROOT / "crates" / crate / "src"
    if not src.exists():
        return items
    for f in sorted(src.rglob("*.rs")):
        text = f.read_text()
        # Everything before the first `#[cfg(test)]`: a public item inside a test module is
        # not part of the crate's surface.
        cut = text.find("#[cfg(test)]")
        if cut >= 0:
            text = text[:cut]
        for line in text.splitlines():
            if not line.startswith("pub "):
                continue  # top level only: an indented `pub fn` is inside an impl or module
            m = SIG.match(line.strip())
            if m:
                items.append((str(f.relative_to(ROOT)), m.group(1).strip()))
    return items


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
    print(render() if what == "map" else api())
