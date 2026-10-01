#!/bin/sh
# tools/fetch-public-calibration.sh [--download] [DATE]
#
# The universe's public calibration (docs/universe.md §5; cycle 16, C16-11), for the author to
# run on his Mac: the container's proxy blocks these sources.
#
# **By default it downloads nothing.** It lists every file it would fetch, asks each server for
# the file's size without fetching it, and prints the source and the licence as recorded here.
# Only `--download` fetches, into results/calibration/, and writes results/calibration/SOURCES.tsv
# with each file's URL, size, SHA-256 and recorded licence. A licence marked "unverified" has not
# been read by anyone on this project; read it before using the file for anything.
#
# DATE is the DTCC end-of-day file's date, YYYY_MM_DD (default: 2026_09_30).
# The SEC requires a User-Agent naming who is asking. It is read from SEC_USER_AGENT
# ("Name email"), set by whoever runs this; it is never written into this file or the results.
set -eu

download=0
if [ "${1:-}" = "--download" ]; then download=1; shift; fi
date=${1:-2026_09_30}
root=$(cd "$(dirname "$0")/.." && pwd)
out="$root/results/calibration"

# name | URL | licence as recorded (unverified unless stated)
sources() {
    cat <<EOF
dtcc-cftc-rates-$date.zip|https://kgc0418-tdw-data-0.s3.amazonaws.com/cftc/eod/CFTC_CUMULATIVE_RATES_$date.zip|DTCC public price dissemination terms of use (unverified; URL pattern from a third-party downloader, github.com/ammop/PPD, and unverified)
dtcc-cftc-forex-$date.zip|https://kgc0418-tdw-data-0.s3.amazonaws.com/cftc/eod/CFTC_CUMULATIVE_FOREX_$date.zip|DTCC public price dissemination terms of use (unverified; URL pattern unverified)
bis-triennial-2025-fx-annex.pdf|https://www.bis.org/publications/triennial-survey/rpfx25-fx-annex-2606.pdf|BIS terms of use (unverified)
EOF
}

size_of() {
    curl -sSIL --max-time 30 "$1" 2>/dev/null | tr -d '\r' | awk 'tolower($1)=="content-length:"{n=$2} END{print (n==""?"unknown":n)}'
}

if [ "$download" = 0 ]; then
    echo "Would fetch (nothing is downloaded without --download):"
    sources | while IFS='|' read -r name url lic; do
        printf '  %s\n    from    %s\n    size    %s bytes\n    licence %s\n' "$name" "$url" "$(size_of "$url")" "$lic"
    done
    echo "  sec-abs-ee: not listed. Set SEC_CIK to an ABS issuer's CIK and SEC_USER_AGENT, and the"
    echo "  issuer's ABS-EE filings are listed from data.sec.gov before anything is fetched."
    exit 0
fi

mkdir -p "$out"
printf 'file\turl\tbytes\tsha256\tlicence\n' > "$out/SOURCES.tsv"
sources | while IFS='|' read -r name url lic; do
    curl -sSfL --max-time 600 -o "$out/$name" "$url"
    bytes=$(wc -c < "$out/$name" | tr -d ' ')
    sum=$( (sha256sum "$out/$name" 2>/dev/null || shasum -a 256 "$out/$name") | cut -d' ' -f1)
    printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$url" "$bytes" "$sum" "$lic" >> "$out/SOURCES.tsv"
    echo "fetched $name ($bytes bytes)"
done

if [ -n "${SEC_CIK:-}" ]; then
    : "${SEC_USER_AGENT:?set SEC_USER_AGENT to \"Name email\"; the SEC requires it}"
    cik=$(printf '%010d' "$SEC_CIK")
    curl -sSf -A "$SEC_USER_AGENT" -o "$out/sec-submissions-$cik.json" \
        "https://data.sec.gov/submissions/CIK$cik.json"
    bytes=$(wc -c < "$out/sec-submissions-$cik.json" | tr -d ' ')
    printf '%s\t%s\t%s\t%s\t%s\n' "sec-submissions-$cik.json" "https://data.sec.gov/submissions/CIK$cik.json" \
        "$bytes" "-" "U.S. government work (unverified)" >> "$out/SOURCES.tsv"
    echo "fetched the issuer's filing index; its ABS-EE exhibits are chosen from it, by hand, next"
fi
echo "results/calibration/SOURCES.tsv written"
