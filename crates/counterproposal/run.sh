#!/usr/bin/env bash
# E14 — the minimal counterproposal.
#
# The question: can reconstructible epoch-anchored views be built with current Rust and
# SQL, and if so, what is left for a new language and a new engine to do?
#
# The method is a differential defect corpus. The same defect classes are written twice —
# once against a good-faith PostgreSQL 16 schema, once in Niles — and for each we record the
# *stage* at which the defect is caught: compile time, runtime, or never.
#
# The two sides were not aligned. The Niles directory had no `d7` and no `d10` while the SQL
# side had both, and the prose said "twelve defect classes ... written twice" over eleven
# files. Both are written now, in the form the defect takes in a language with no run-time
# rule-dropping and no wall clock, and `crates/bank-bench/tests/counterproposal.rs` counts
# the Niles side from the files so the tally in §6.10.3 cannot drift from the corpus again.
#
#   ./crates/counterproposal/run.sh
#
# Requires a running PostgreSQL 16 and a built `nilesc`.
set -uo pipefail
cd "$(dirname "$0")/../.."
DB=counterproposal
PSQL="su postgres -c"

echo "== rebuilding the counterproposal database =="
$PSQL "dropdb --if-exists $DB" >/dev/null 2>&1
$PSQL "createdb $DB" >/dev/null 2>&1
$PSQL "psql -q -d $DB -f $PWD/crates/counterproposal/sql/schema.sql" 2>&1 | grep -E "ERROR" && exit 1

echo "== loading a ledger (2,000 epochs, 4,000 postings, 50 accounts, C=16) =="
# `|| true` here would hide a load failure and leave every later part measuring an empty
# database. A load error is fatal.
if $PSQL "psql -q -d $DB -f $PWD/crates/counterproposal/sql/load.sql" 2>&1 | grep -E "ERROR"; then
  echo "the load failed; the corpus below would be measured against an empty ledger" >&2
  exit 1
fi

echo "== part 1: does the REV mechanism work in PostgreSQL? =="
$PSQL "psql -q -d $DB -f $PWD/crates/counterproposal/sql/mechanism.sql" 2>&1 | grep -vE "^$|^CONTEXT"

echo
echo "== part 2: the defect corpus, in PostgreSQL =="
$PSQL "psql -q -d $DB -f $PWD/crates/counterproposal/sql/defects.sql" 2>&1 | grep -E "^###|RESULT|CAUGHT" | sed 's/^psql[^N]*NOTICE:  //'

echo
echo "== part 3: the same corpus, in Niles =="
rejected=0; warned=0; accepted=0; total=0
for f in crates/counterproposal/niles/*.niles; do
  n=$(basename "$f" .niles)
  out=$(cargo run -q -p nilesc -- check "$f" 2>&1)
  vf=$(cargo run -q -p nilesc -- verify "$f" 2>&1 | grep -oE "IR0[0-9]+" | head -1)
  codes=$(echo "$out" | grep -oE "^(error|warning)\[NL[0-9]+\]" | sort -u | tr '\n' ' ')
  total=$((total + 1))
  if echo "$out" | grep -q "^ok:"; then
    if echo "$codes" | grep -q warning; then st="warned"; warned=$((warned + 1));
    else st="accepted"; accepted=$((accepted + 1)); fi
  else st="REJECTED"; rejected=$((rejected + 1)); fi
  printf "  %-34s %-9s %s%s\n" "$n" "$st" "$codes" "$vf"
done
echo
echo "  $total defect classes: $rejected refused at compile time, $warned warned, $accepted accepted silently"
if [ "$total" -eq 0 ]; then echo "no corpus files found" >&2; exit 1; fi
