#!/usr/bin/env bash
# Deploy Domus to /opt/usr/local/domus and install the backup timer.
set -euo pipefail

DEST_DIR="${DOMUS_DEST_DIR:-/opt/usr/local/domus}"
RUN_USER="${DOMUS_RUN_USER:-root}"
BACKUP_REPO_URL="${DOMUS_BACKUP_REPO_URL:-git@github.com:xiedeacc/domus_data.git}"
SYSTEMD_DIR="${DOMUS_SYSTEMD_DIR:-/etc/systemd/system}"
SKIP_BUILD="${DOMUS_SKIP_BUILD:-0}"
START_SERVICE="${DOMUS_START_SERVICE:-1}"
START_BACKUP_TIMER="${DOMUS_START_BACKUP_TIMER:-1}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

log() {
    echo "[domus-deploy] $*"
}

build_server() {
    if [ "$SKIP_BUILD" = "1" ]; then
        log "using existing release binary"
        return
    fi
    log "building domus-server"
    (cd "$repo_root/server" && cargo build --release --bin domus-server)
}

install_layout() {
    log "installing files to $DEST_DIR"
    mkdir -p "$DEST_DIR"/{bin,conf,data,logs}
    install -m 0755 "$repo_root/server/target/release/domus-server" "$DEST_DIR/bin/domus-server"
    install -m 0755 "$repo_root/scripts/backup_sqlite.sh" "$DEST_DIR/bin/backup_sqlite.sh"
    if [ -f "$repo_root/scripts/migrate_pg_to_sqlite.py" ]; then
        install -m 0755 "$repo_root/scripts/migrate_pg_to_sqlite.py" "$DEST_DIR/bin/migrate_pg_to_sqlite.py"
    fi

    if [ ! -f "$DEST_DIR/conf/domus.env" ]; then
        cat >"$DEST_DIR/conf/domus.env" <<EOF
DOMUS_PORT=2284
DOMUS_MEDIA_LOCATION=${DEST_DIR}/data/upload
DOMUS_DATABASE__URL=sqlite://${DEST_DIR}/data/domus.sqlite3
DOMUS_DATABASE__RUN_MIGRATIONS=false
RUST_LOG=info,sqlx=warn
EOF
        chmod 0600 "$DEST_DIR/conf/domus.env"
    fi

    if [ "$RUN_USER" != "root" ]; then
        id -u "$RUN_USER" >/dev/null 2>&1 || useradd --system --home "$DEST_DIR" --shell /usr/sbin/nologin "$RUN_USER"
        chown -R "$RUN_USER":"$RUN_USER" "$DEST_DIR"
    fi
}

write_service() {
    mkdir -p "$SYSTEMD_DIR"
    cat >"$SYSTEMD_DIR/domus.service" <<EOF
[Unit]
Description=Domus Immich-compatible server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${RUN_USER}
WorkingDirectory=${DEST_DIR}
EnvironmentFile=${DEST_DIR}/conf/domus.env
Environment=HOME=${DEST_DIR}
ExecStart=${DEST_DIR}/bin/domus-server
Restart=always
RestartSec=3
StandardOutput=append:${DEST_DIR}/logs/domus.log
StandardError=append:${DEST_DIR}/logs/domus.log
NoNewPrivileges=true
ProtectSystem=strict
ReadWritePaths=${DEST_DIR}
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictSUIDSGID=true
LockPersonality=true
UMask=0077
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
}

install_backup() {
    DOMUS_DEST_DIR="$DEST_DIR" \
    DOMUS_RUN_USER="$RUN_USER" \
    DOMUS_BACKUP_REPO_URL="$BACKUP_REPO_URL" \
    DOMUS_SYSTEMD_DIR="$SYSTEMD_DIR" \
    DOMUS_START_BACKUP_TIMER="$START_BACKUP_TIMER" \
        "$repo_root/scripts/install-backup-systemd.sh"
}

enable_and_start() {
    systemctl daemon-reload
    systemctl enable domus.service
    if [ "$START_SERVICE" = "1" ]; then
        systemctl restart domus.service
    fi
}

main() {
    build_server
    install_layout
    write_service
    install_backup
    enable_and_start
    log "deployed Domus to $DEST_DIR"
}

main "$@"
