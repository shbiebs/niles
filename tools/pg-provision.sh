#!/bin/sh
# Provision the PostgreSQL role and database the harness uses, without weakening
# authentication. Idempotent: safe to run on every session.
#
# What it does, and why each step is this way (cycle 14, decision D-1):
#   * role `bench`, LOGIN, with a password; the harness authenticates with SCRAM-SHA-256,
#     which is what a stock PostgreSQL 16 requires from 127.0.0.1. pg_hba.conf is never
#     edited: a gate that passed only after weakening the server tested a server nobody runs.
#   * the password is generated here, on the machine that uses it, into
#     $NILES_PGPASSFILE (default: .pg-bench-password at the workspace root, git-ignored),
#     mode 600. It is never printed. If the file exists it is reused, and the role's password
#     is reset to it, so the two cannot drift apart.
#   * membership in pg_read_all_settings, because `bench --calibrate` asks the server for
#     `data_directory` to probe fsync cost on the device the WAL is on (bench.rs). Nothing
#     else: not superuser, not CREATEDB.
#   * database `bank`, owned by `bench`.
#   The SQL goes to psql on standard input, so the password never appears in a process list.
#   One caveat, stated rather than hidden: a server configured with log_statement = ddl or
#   all writes ALTER ROLE ... PASSWORD to its log. The stock setting is `none`.
#
# Administrative access: as root (the cloud container) it runs psql as the `postgres` OS user
# over the local socket (peer authentication); otherwise it runs psql as the current user,
# which is the superuser of a Homebrew or Postgres.app cluster. Override with PSQL_ADMIN.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
pwfile=${NILES_PGPASSFILE:-$root/.pg-bench-password}
port=${PGPORT:-5432}

if [ ! -s "$pwfile" ]; then
    umask 077
    head -c 48 /dev/urandom | base64 | tr -d '/+=\n' | cut -c1-32 > "$pwfile"
fi
chmod 600 "$pwfile"
pw=$(cat "$pwfile")
case "$pw" in
    *[!A-Za-z0-9]*|'') echo "pg-provision: $pwfile holds an unexpected password format; delete it and rerun" >&2; exit 2 ;;
esac

admin() {
    if [ -n "${PSQL_ADMIN:-}" ]; then
        sh -c "$PSQL_ADMIN"
    elif [ "$(id -u)" = 0 ]; then
        su postgres -c "psql -X -q -v ON_ERROR_STOP=1 -p $port -d postgres"
    else
        psql -X -q -v ON_ERROR_STOP=1 -p "$port" -d postgres
    fi
}

admin <<SQL
SELECT 'CREATE ROLE bench LOGIN' WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'bench') \gexec
ALTER ROLE bench LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE PASSWORD '$pw';
GRANT pg_read_all_settings TO bench;
SELECT 'CREATE DATABASE bank OWNER bench' WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = 'bank') \gexec
SQL

echo "pg-provision: role bench and database bank ready on port $port; password file $pwfile"
