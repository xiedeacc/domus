#!/usr/bin/env bash
set -euo pipefail

DB_PATH="${1:-/opt/usr/local/domus/data/domus.sqlite3}"
BACKUP_ROOT="${2:-/opt/usr/local/domus/data/backups}"

if [[ ! -f "${DB_PATH}" ]]; then
  echo "SQLite database not found: ${DB_PATH}" >&2
  exit 1
fi

TS="$(date -u +%Y%m%dT%H%M%SZ)"
DIR="${BACKUP_ROOT}/sqlite-${TS}"
mkdir -p "${DIR}"

sqlite3 "${DB_PATH}" ".backup '${DIR}/domus.sqlite3'"
sqlite3 "${DIR}/domus.sqlite3" "PRAGMA integrity_check;" > "${DIR}/integrity.txt"
sqlite3 "${DIR}/domus.sqlite3" <<'SQL' > "${DIR}/manifest.txt"
.mode list
select 'user_count', count(*) from "user";
select 'asset_count', count(*) from asset;
select 'album_count', count(*) from album;
select 'memory_count', count(*) from memory;
select 'asset_file_count', count(*) from asset_file;
SQL

sha256sum "${DIR}"/* > "${DIR}/SHA256SUMS"
echo "${DIR}" > "${BACKUP_ROOT}/latest-sqlite-backup.txt"
echo "${DIR}"
