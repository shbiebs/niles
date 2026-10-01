#!/usr/bin/env bash
# tools/replay.sh CHAPTER.md [WORKDIR]
#
# Replay a chapter of the manual (cycle 16, C16-12; cycle 15's C15-09): execute every fenced
# `sh` block of CHAPTER.md, in order, each in a fresh `bash -euo pipefail` started in WORKDIR
# (default: the repository root), and stop at the first block that fails. Blocks share files
# and processes, not shell variables: a chapter that needs a value in a later block writes it
# to a file, as a reader following it in two terminals would have to.
#
# * A `sh` block immediately followed by a fenced `expected` block must print exactly that
#   text (trailing whitespace ignored). This is how a chapter states its checkpoint: what the
#   reader must see, not what the author saw once.
# * `console` and `text` blocks are illustrations and are never run. A chapter that wants a
#   step run writes it as `sh`; a step the replay would have to guess is a defect of the
#   chapter, and the acceptance test of C16-12 is that there are none.
# * Before the first block it logs the host and every tool version the chapter could depend
#   on; after the last it requires the git tree to be as clean as it was at the start, so a
#   chapter cannot pass by leaving the repository in a state a reader would not have.
#
# The log is written to WORKDIR/.replay/<chapter>.log, and the exit status is the replay's.
set -euo pipefail

chapter=${1:?usage: tools/replay.sh CHAPTER.md [WORKDIR]}
root=$(cd "$(dirname "$0")/.." && pwd)
work=${2:-$root}
chapter=$(cd "$(dirname "$chapter")" && pwd)/$(basename "$chapter")
name=$(basename "$chapter" .md)
logdir="$work/.replay"
mkdir -p "$logdir"
log="$logdir/$name.log"
blocks="$logdir/$name.blocks"
rm -rf "$blocks"
mkdir -p "$blocks"

# Split the chapter into numbered blocks: N.sh for each `sh` block, N.expected for the
# `expected` block that follows it directly (blank lines between them allowed).
awk -v dir="$blocks" '
    BEGIN { n = 0; inblock = 0; kind = ""; last_sh = 0 }
    /^```/ {
        if (!inblock) {
            info = substr($0, 4); gsub(/[ \t]+$/, "", info)
            if (info == "sh") { n++; kind = "sh"; file = dir "/" sprintf("%03d", n) ".sh"; last_sh = n; inblock = 1; printf "" > file; next }
            if (info == "expected" && last_sh == n && n > 0 && pending) { kind = "expected"; file = dir "/" sprintf("%03d", n) ".expected"; inblock = 1; printf "" > file; next }
            kind = "skip"; inblock = 1; next
        } else {
            if (kind == "sh") { pending = 1 } else { pending = 0 }
            inblock = 0; close(file); next
        }
    }
    {
        if (inblock && (kind == "sh" || kind == "expected")) { print >> file }
        else if (!inblock && $0 !~ /^[ \t]*$/) { pending = 0 }
    }
' "$chapter"

{
    echo "# replay of $chapter in $work, $(date -u +%FT%TZ)"
    echo "host: $(uname -srm)"
    echo "cores: $(getconf _NPROCESSORS_ONLN 2>/dev/null || echo unknown)"
    for t in cargo rustc python3 psql git; do
        printf '%s: ' "$t"; ("$t" --version 2>/dev/null || echo absent) | head -1
    done
    echo "commit: $(git -C "$work" rev-parse HEAD 2>/dev/null || echo none)"
} | tee "$log"

before=$(git -C "$work" status --porcelain 2>/dev/null || true)

for f in "$blocks"/*.sh; do
    [ -e "$f" ] || { echo "replay: $chapter has no sh blocks" | tee -a "$log"; exit 2; }
    n=$(basename "$f" .sh)
    echo "── block $n ──" | tee -a "$log"
    sed 's/^/  $ /' "$f" >> "$log"
    out="$blocks/$n.out"
    if ! (cd "$work" && bash -euo pipefail "$f") > "$out" 2>> "$log"; then
        cat "$out" >> "$log"
        echo "replay: block $n failed; see $log" | tee -a "$log"
        exit 1
    fi
    cat "$out" >> "$log"
    if [ -e "$blocks/$n.expected" ]; then
        if ! diff <(sed 's/[ \t]*$//' "$blocks/$n.expected") <(sed 's/[ \t]*$//' "$out") >> "$log"; then
            echo "replay: block $n printed something other than its expected block" | tee -a "$log"
            exit 1
        fi
        echo "  (matches its expected block)" >> "$log"
    fi
done

after=$(git -C "$work" status --porcelain 2>/dev/null || true)
if [ "$before" != "$after" ]; then
    echo "replay: the chapter left the tree changed:" | tee -a "$log"
    diff <(echo "$before") <(echo "$after") | tee -a "$log" || true
    exit 1
fi
echo "replay: $(ls "$blocks"/*.sh | wc -l | tr -d ' ') blocks passed; tree as it was" | tee -a "$log"
