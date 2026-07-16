#!/usr/bin/env bash
# Mirror a Domus deployment's bin/, conf/, and recoverable data state into a
# separate Git worktree and push it to a private backup repository. Large media
# trees are intentionally excluded; they should be backed by storage snapshots.
set -euo pipefail

BACKUP_REPO_URL="${DOMUS_BACKUP_REPO_URL:-git@github.com:xiedeacc/domus_data.git}"
BACKUP_BRANCH="${DOMUS_BACKUP_BRANCH:-master}"
MAX_FILE_BYTES="${DOMUS_BACKUP_MAX_FILE_BYTES:-52428800}"  # 50 MiB commit threshold
SPLIT_BYTES="${DOMUS_BACKUP_SPLIT_BYTES:-49000000}"         # ~49 MiB chunk size

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
install_dir="${DOMUS_BACKUP_ROOT:-$(dirname "$script_dir")}"
work_dir="${DOMUS_BACKUP_WORK_DIR:-${install_dir}/.backup-worktree}"
db_file="${DOMUS_BACKUP_DB:-${install_dir}/data/domus.sqlite3}"
lock_file="${DOMUS_BACKUP_LOCK_FILE:-${install_dir}/data/.backup.lock}"

log() {
    echo "[domus-backup] $*"
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        log "required command not found: $1"
        exit 127
    fi
}

checkout_backup_branch() {
    if git -C "$work_dir" rev-parse --verify "origin/${BACKUP_BRANCH}" >/dev/null 2>&1; then
        git -C "$work_dir" checkout -B "$BACKUP_BRANCH" "origin/${BACKUP_BRANCH}"
    elif git -C "$work_dir" rev-parse --verify "$BACKUP_BRANCH" >/dev/null 2>&1; then
        git -C "$work_dir" checkout "$BACKUP_BRANCH"
    else
        git -C "$work_dir" checkout --orphan "$BACKUP_BRANCH"
    fi
}

ensure_repo() {
    if [ -d "${work_dir}/.git" ]; then
        git -C "$work_dir" remote set-url origin "$BACKUP_REPO_URL"
        git -C "$work_dir" fetch origin "$BACKUP_BRANCH" || true
        checkout_backup_branch
        git -C "$work_dir" pull --ff-only origin "$BACKUP_BRANCH" || true
        return
    fi

    if git clone --branch "$BACKUP_BRANCH" "$BACKUP_REPO_URL" "$work_dir" 2>/dev/null; then
        return
    fi

    log "clone failed (empty remote?) - initializing local repo"
    mkdir -p "$work_dir"
    git -C "$work_dir" init -b "$BACKUP_BRANCH"
    git -C "$work_dir" remote add origin "$BACKUP_REPO_URL"
}

sync_source() {
    find "$work_dir" -mindepth 1 -maxdepth 1 \
        ! -name '.git' ! -name 'bin' ! -name 'conf' ! -name 'data' \
        ! -name '.domus-empty-dirs' -exec rm -rf -- {} +
    mkdir -p "$work_dir/bin" "$work_dir/conf" "$work_dir/data"

    if [ -d "$install_dir/bin" ]; then
        rsync -a --delete \
            --exclude '*.bak-*' \
            --exclude '*.bak' \
            "$install_dir/bin/" "$work_dir/bin/"
    fi
    if [ -d "$install_dir/conf" ]; then
        rsync -a --delete "$install_dir/conf/" "$work_dir/conf/"
    fi
    if [ -d "$install_dir/data" ]; then
        rsync -a --delete \
            --exclude '/.backup.lock' \
            --exclude '/.domus.sqlite3.*.tmp*' \
            --exclude '/domus.sqlite3' \
            --exclude '/domus.sqlite3.bak' \
            --exclude '/domus.sqlite3-shm' \
            --exclude '/domus.sqlite3-wal' \
            --exclude '/backups/' \
            --exclude '/upload/' \
            --exclude '/library/' \
            --exclude '/thumbs/' \
            --exclude '/thumbs*/' \
            --exclude '/encoded-video/' \
            --exclude '/profile/' \
            "$install_dir/data/" "$work_dir/data/"
    fi
}

snapshot_sqlite() {
    local snapshot="$work_dir/data/domus.sqlite3"
    local temporary="${snapshot}.tmp"
    rm -f "$snapshot" "$temporary"

    if [ ! -f "$db_file" ]; then
        log "no sqlite db at ${db_file} - skipping snapshot"
        return
    fi

    sqlite3 "$db_file" "VACUUM INTO '$temporary'"
    mv "$temporary" "$snapshot"
    log "sqlite snapshot written: $snapshot"

    local integrity
    integrity="$(sqlite3 "$snapshot" 'PRAGMA integrity_check')"
    if [ "$integrity" != "ok" ]; then
        log "sqlite snapshot integrity check failed: $integrity"
        return 1
    fi
}

record_empty_dirs() {
    local manifest="$work_dir/.domus-empty-dirs"
    find "$work_dir" -type d -empty \
        -not -path "$work_dir/.git" \
        -not -path "$work_dir/.git/*" \
        -printf '%P\n' | LC_ALL=C sort >"$manifest"
}

split_file() {
    local file="$1"
    python3 - "$file" "$SPLIT_BYTES" <<'PYEOF'
import sys

path, chunk = sys.argv[1], int(sys.argv[2])
i = 0
with open(path, "rb") as f:
    while True:
        data = f.read(chunk)
        if not data:
            break
        with open(f"{path}.{i}", "wb") as out:
            out.write(data)
        i += 1
PYEOF
}

ignore_path() {
    local rel="$1"
    local exclude="$work_dir/.git/info/exclude"
    touch "$exclude"
    if ! grep -qxF "/$rel" "$exclude"; then
        echo "/$rel" >>"$exclude"
    fi
}

reset_generated_split_files() {
    find "$work_dir" -name '*.domus-split' -not -path "*/.git/*" | while read -r marker; do
        local base="${marker%.domus-split}"
        rm -f "$base" "$base".[0-9]* "$marker"
    done
}

split_large_files() {
    find "$work_dir" -type f -size +"$MAX_FILE_BYTES"c \
        -not -path "*/.git/*" \
        -not -name '.gitignore' \
        -not -name '*.domus-split' | while read -r file; do
        rel="${file#"$work_dir"/}"
        case "$rel" in
            *.[0-9]|*.[0-9][0-9]) continue ;;
        esac

        log "splitting large file: $rel"
        ignore_path "$rel"
        size="$(stat -c '%s' "$file")"
        digest="$(sha256sum "$file" | cut -d' ' -f1)"
        chunks="$(( (size + SPLIT_BYTES - 1) / SPLIT_BYTES ))"
        split_file "$file"
        {
            echo "original=$rel"
            echo "split_bytes=$SPLIT_BYTES"
            echo "size=$size"
            echo "sha256=$digest"
            echo "chunks=$chunks"
        } >"${file}.domus-split"
        rm -f "$file"
    done
}

commit_and_push_if_changed() {
    git -C "$work_dir" add -A
    if git -C "$work_dir" diff --cached --quiet; then
        log "no changes to back up"
        if git -C "$work_dir" rev-parse --verify HEAD >/dev/null 2>&1; then
            git -C "$work_dir" pull --ff-only origin "$BACKUP_BRANCH" || true
            git -C "$work_dir" push origin "$BACKUP_BRANCH"
        fi
        return
    fi

    if [ -z "$(git -C "$work_dir" config user.email || true)" ]; then
        git -C "$work_dir" config user.email "domus-backup@localhost"
        git -C "$work_dir" config user.name "domus backup"
    fi

    git -C "$work_dir" commit -m "Backup $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
    git -C "$work_dir" pull --rebase origin "$BACKUP_BRANCH" || true
    git -C "$work_dir" push origin "$BACKUP_BRANCH"
    log "backup pushed"
}

main() {
    require_command git
    require_command rsync
    require_command python3
    require_command sqlite3
    require_command find
    require_command stat
    require_command sha256sum
    require_command flock

    if [ ! -d "$install_dir" ]; then
        log "install dir not found: $install_dir"
        exit 1
    fi

    mkdir -p "$(dirname "$lock_file")"
    exec 9>"$lock_file"
    if ! flock -n 9; then
        log "another backup is already running"
        exit 75
    fi

    log "backing up $install_dir/{bin,conf,data} -> $BACKUP_REPO_URL ($BACKUP_BRANCH)"
    ensure_repo
    reset_generated_split_files
    sync_source
    snapshot_sqlite
    record_empty_dirs
    split_large_files
    commit_and_push_if_changed
    log "done"
}

main "$@"
