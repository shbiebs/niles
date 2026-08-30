#!/usr/bin/env bash
# Assemble the thesis into a single .docx. Chapter order is the order the project
# description fixes; the appendices follow, then the references.
set -euo pipefail
cd "$(dirname "$0")"
FILES=(00-front-matter.md 01-introduction.md 02-background.md 03-theoretical-framework.md \
  04-novel-contributions.md 05-research-design.md 06-architecture-and-niles.md \
  07-implementation.md 08-phased-program.md 09-evaluation.md 10-related-work.md \
  11-discussion.md 12-future-work.md 13-conclusion.md \
  appendix-a.md appendix-b.md appendix-c.md appendix-d.md appendix-e.md appendix-f.md \
  appendix-g.md appendix-h.md appendix-i.md appendix-j.md appendix-k.md references.md)
pandoc "${FILES[@]}" -o Niles-Thesis.docx --toc --toc-depth=3 \
  -M title="Niles and Nilestream" \
  -M subtitle="Reconstructible Epoch-Anchored Views: a theory of versioned partially-materialized state over an immutable ledger" \
  -M author="Sergio" \
  --resource-path=.
echo "built Niles-Thesis.docx"
