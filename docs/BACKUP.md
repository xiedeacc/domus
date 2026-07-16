# Domus Deployment Backup

Domus can mirror a production deployment directory into a dedicated Git
repository, similar to the rgit backup flow.

Default layout:

```text
/opt/usr/local/domus/
  bin/
  conf/
  data/
  logs/
  .backup-worktree/  # git@github.com:xiedeacc/domus_data.git checkout
```

The backup script copies `bin/`, `conf/`, and `data/` into
`.backup-worktree`, snapshots `data/domus.sqlite3` with SQLite `VACUUM INTO`,
commits changes, pulls/rebases, and pushes to GitHub.

Install the systemd timer:

```bash
sudo DOMUS_DEST_DIR=/opt/usr/local/domus \
  DOMUS_BACKUP_REPO_URL=git@github.com:xiedeacc/domus_data.git \
  scripts/install-backup-systemd.sh
```

Run one backup manually:

```bash
sudo systemctl start domus-backup.service
sudo journalctl -u domus-backup.service -n 100 --no-pager
```

Useful environment variables:

| Variable | Default |
|---|---|
| `DOMUS_BACKUP_REPO_URL` | `git@github.com:xiedeacc/domus_data.git` |
| `DOMUS_BACKUP_BRANCH` | `master` |
| `DOMUS_BACKUP_ROOT` | parent of the script directory |
| `DOMUS_BACKUP_WORK_DIR` | `$DOMUS_BACKUP_ROOT/.backup-worktree` |
| `DOMUS_BACKUP_DB` | `$DOMUS_BACKUP_ROOT/data/domus.sqlite3` |
| `DOMUS_BACKUP_MAX_FILE_BYTES` | `52428800` |
| `DOMUS_BACKUP_SPLIT_BYTES` | `49000000` |
