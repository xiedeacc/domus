#!/usr/bin/env bash
# Install Domus backup systemd units for /opt/usr/local/domus-style layouts.
set -euo pipefail

DEST_DIR="${DOMUS_DEST_DIR:-/opt/usr/local/domus}"
RUN_USER="${DOMUS_RUN_USER:-root}"
BACKUP_REPO_URL="${DOMUS_BACKUP_REPO_URL:-git@github.com:xiedeacc/domus_data.git}"
SYSTEMD_DIR="${DOMUS_SYSTEMD_DIR:-/etc/systemd/system}"
ENABLE_UNITS="${DOMUS_ENABLE_UNITS:-1}"
START_TIMER="${DOMUS_START_BACKUP_TIMER:-1}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

log() {
    echo "[domus-backup-install] $*"
}

install_layout() {
    mkdir -p "$DEST_DIR"/{bin,conf,data,logs,.backup-worktree}
    install -d -m 0700 "$DEST_DIR/.ssh"
    install -m 0755 "$repo_root/scripts/domus-backup.sh" "$DEST_DIR/bin/domus-backup"
    if command -v ssh-keyscan >/dev/null 2>&1; then
        touch "$DEST_DIR/.ssh/known_hosts"
        ssh-keyscan github.com 2>/dev/null >>"$DEST_DIR/.ssh/known_hosts" || true
        sort -u "$DEST_DIR/.ssh/known_hosts" -o "$DEST_DIR/.ssh/known_hosts"
        chmod 0600 "$DEST_DIR/.ssh/known_hosts"
    fi
    if [ "$RUN_USER" != "root" ]; then
        id -u "$RUN_USER" >/dev/null 2>&1 || useradd --system --home "$DEST_DIR" --shell /usr/sbin/nologin "$RUN_USER"
        chown -R "$RUN_USER":"$RUN_USER" "$DEST_DIR/data" "$DEST_DIR/logs" "$DEST_DIR/conf" "$DEST_DIR/.backup-worktree" "$DEST_DIR/.ssh"
    fi
}

write_backup_service() {
    mkdir -p "$SYSTEMD_DIR"
    cat >"$SYSTEMD_DIR/domus-backup.service" <<EOF
[Unit]
Description=Domus backup to GitHub
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
User=${RUN_USER}
WorkingDirectory=${DEST_DIR}
Environment=DOMUS_BACKUP_REPO_URL=${BACKUP_REPO_URL}
Environment=DOMUS_BACKUP_ROOT=${DEST_DIR}
Environment=DOMUS_BACKUP_WORK_DIR=${DEST_DIR}/.backup-worktree
Environment=DOMUS_BACKUP_DB=${DEST_DIR}/data/domus.sqlite3
Environment=HOME=${DEST_DIR}
Environment=GIT_SSH_COMMAND=ssh -o IdentitiesOnly=yes -o UserKnownHostsFile=${DEST_DIR}/.ssh/known_hosts -o StrictHostKeyChecking=yes
ExecStart=${DEST_DIR}/bin/domus-backup
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
EOF
}

write_backup_timer() {
    cat >"$SYSTEMD_DIR/domus-backup.timer" <<EOF
[Unit]
Description=Periodic Domus backup

[Timer]
OnBootSec=5min
OnUnitActiveSec=1h
AccuracySec=1min
Persistent=true
Unit=domus-backup.service

[Install]
WantedBy=timers.target
EOF
}

enable_units() {
    if [ "$ENABLE_UNITS" != "1" ]; then
        log "systemd enable/start skipped"
        return
    fi
    systemctl daemon-reload
    systemctl enable domus-backup.timer
    if [ "$START_TIMER" = "1" ]; then
        systemctl start domus-backup.timer
    fi
}

main() {
    install_layout
    write_backup_service
    write_backup_timer
    enable_units
    log "installed backup timer for ${DEST_DIR}; repo=${BACKUP_REPO_URL}"
}

main "$@"
