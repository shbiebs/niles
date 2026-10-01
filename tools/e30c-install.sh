#!/bin/sh
# Install the dialect's event triggers for E30c (docs/study/E30c-design.md §5): a database
# `dialect` owned by the provisioned role `bench`, with corpus/e30c/schema/dialect.sql run in it
# by the cluster's superuser (event triggers are superuser-only). Idempotent: the database is
# dropped and re-created each time, so the triggers installed are the file's.
#
# Run after tools/pg-provision.sh. As root (the container) the superuser is reached through
# `su postgres`; elsewhere set PSQL_ADMIN to a command that runs psql as a superuser, as
# tools/pg-provision.sh documents.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
port=${PGPORT:-5432}
sql="$root/crates/syntax-study/corpus/e30c/schema/dialect.sql"

# Run a file of SQL as the superuser, in database $1.
admin() {
    if [ -n "${PSQL_ADMIN:-}" ]; then
        sh -c "$PSQL_ADMIN -d $1 -f $2"
    elif [ "$(id -u)" = 0 ]; then
        su postgres -c "psql -X -q -v ON_ERROR_STOP=1 -p $port -d $1 -f $2"
    else
        psql -X -q -v ON_ERROR_STOP=1 -p "$port" -d "$1" -f "$2"
    fi
}

tmp=$(mktemp -d)
chmod 755 "$tmp"
printf 'drop database if exists dialect;\ncreate database dialect owner bench;\n' > "$tmp/db.sql"
cp "$sql" "$tmp/dialect.sql"
chmod 644 "$tmp/db.sql" "$tmp/dialect.sql"
admin postgres "$tmp/db.sql"
admin dialect "$tmp/dialect.sql"
rm -rf "$tmp"
echo "e30c-install: database dialect ready on port $port, event triggers installed from $sql"
