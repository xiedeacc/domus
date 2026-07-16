#!/usr/bin/env python3
"""Migrate a native Immich PostgreSQL database into Domus SQLite.

The script is intentionally dependency-light for NAS deployments: it only needs
`psql` and Python's stdlib sqlite3. It reads through COPY-to-JSON streams so the
source PostgreSQL database is never modified.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable


CORE_TABLES: dict[str, str] = {
    "user": """
        id blob primary key,
        email text not null unique,
        password text not null default '',
        name text not null default '',
        isAdmin integer not null default 0,
        avatarColor text,
        profileImagePath text not null default '',
        storageLabel text,
        oauthId text not null default '',
        quotaSizeInBytes integer,
        quotaUsageInBytes integer not null default 0,
        shouldChangePassword integer not null default 1,
        createdAt text not null,
        updatedAt text not null,
        deletedAt text,
        profileChangedAt text not null,
        status text,
        pinCode text
    """,
    "session": """
        id blob primary key,
        token blob not null,
        userId blob not null,
        deviceType text not null default '',
        deviceOS text not null default '',
        expiresAt text,
        createdAt text not null,
        updatedAt text not null,
        appVersion text,
        oauthSid text
    """,
    "api_key": """
        id blob primary key,
        name text not null,
        key blob not null,
        userId blob not null,
        permissions text not null default '[]',
        createdAt text not null,
        updatedAt text not null
    """,
    "asset": """
        id blob primary key,
        ownerId blob not null,
        libraryId blob,
        type text not null,
        originalPath text not null,
        originalFileName text not null,
        checksum blob not null,
        checksumAlgorithm text not null default 'sha1',
        visibility text not null default 'timeline',
        isFavorite integer not null default 0,
        isOffline integer not null default 0,
        isExternal integer not null default 0,
        livePhotoVideoId blob,
        stackId blob,
        duplicateId blob,
        duration integer,
        thumbhash blob,
        fileCreatedAt text not null,
        fileModifiedAt text not null,
        localDateTime text not null,
        createdAt text not null,
        updatedAt text not null,
        deletedAt text,
        width integer,
        height integer,
        isEdited integer not null default 0,
        status text not null default 'active'
    """,
    "asset_exif": """
        assetId blob primary key,
        make text,
        model text,
        exifImageWidth integer,
        exifImageHeight integer,
        fileSizeInByte integer,
        orientation text,
        dateTimeOriginal text,
        modifyDate text,
        timeZone text,
        latitude real,
        longitude real,
        city text,
        state text,
        country text,
        description text not null default '',
        fNumber real,
        focalLength real,
        iso integer,
        exposureTime text,
        lensModel text,
        projectionType text,
        rating integer,
        fps real,
        livePhotoCID text,
        profileDescription text,
        colorspace text,
        bitsPerSample integer,
        autoStackId blob
    """,
    "asset_file": """
        id blob primary key,
        assetId blob not null,
        type text not null,
        path text not null,
        createdAt text not null,
        updatedAt text not null,
        isEdited integer not null default 0,
        isProgressive integer not null default 0,
        isTransparent integer not null default 0,
        unique(assetId, type)
    """,
    "album": """
        id blob primary key,
        ownerId blob,
        albumName text not null default '',
        description text not null default '',
        albumThumbnailAssetId blob,
        isActivityEnabled integer not null default 1,
        "order" text not null default 'desc',
        createdAt text not null,
        updatedAt text not null,
        deletedAt text
    """,
    "album_asset": """
        albumId blob not null,
        assetId blob not null,
        createdAt text not null,
        updatedAt text,
        primary key(albumId, assetId)
    """,
    "album_user": """
        albumId blob not null,
        userId blob not null,
        role text not null default 'editor',
        createdAt text,
        updatedAt text,
        primary key(albumId, userId)
    """,
    "memory": """
        id blob primary key,
        ownerId blob not null,
        type text not null default 'on_this_day',
        data text not null,
        memoryAt text not null,
        isSaved integer not null default 0,
        seenAt text,
        showAt text,
        hideAt text,
        createdAt text not null,
        updatedAt text not null,
        deletedAt text
    """,
    "memory_asset": """
        memoryId blob not null,
        assetId blob not null,
        primary key(memoryId, assetId)
    """,
    "shared_link": """
        id blob primary key,
        userId blob not null,
        key blob not null,
        slug text,
        type text not null,
        albumId blob,
        description text,
        password text,
        allowUpload integer not null default 0,
        allowDownload integer not null default 1,
        showExif integer not null default 1,
        expiresAt text,
        createdAt text not null
    """,
    "shared_link_asset": """
        sharedLinkId blob not null,
        assetId blob not null,
        primary key(sharedLinkId, assetId)
    """,
    "partner": """
        sharedById blob not null,
        sharedWithId blob not null,
        inTimeline integer not null default 0,
        createdAt text not null,
        primary key(sharedById, sharedWithId)
    """,
    "system_metadata": """
        key text primary key,
        value text not null
    """,
    "tag": """
        id blob primary key,
        userId blob not null,
        value text not null,
        color text,
        parentId blob,
        createdAt text not null,
        updatedAt text not null
    """,
    "tag_asset": """
        tagId blob not null,
        assetId blob not null,
        primary key(tagId, assetId)
    """,
    "job": """
        id blob primary key,
        queue text not null,
        name text not null,
        payload text not null default '{}',
        status text not null default 'waiting',
        attempts integer not null default 0,
        maxAttempts integer not null default 3,
        error text,
        runAt text not null,
        createdAt text not null,
        updatedAt text not null
    """,
}


SELECTS: dict[str, str] = {
    "user": '''select id, email, password, name, "isAdmin", "avatarColor", "profileImagePath",
               "storageLabel", "oauthId", "quotaSizeInBytes", "quotaUsageInBytes",
               "shouldChangePassword", "createdAt", "updatedAt", "deletedAt",
               "profileChangedAt", status, "pinCode" from "user"''',
    "session": '''select id, encode(token, 'hex') as token, "userId", "deviceType", "deviceOS",
                  "expiresAt", "createdAt", "updatedAt", "appVersion", "oauthSid" from session''',
    "api_key": '''select id, name, encode(key, 'hex') as key, "userId", permissions, "createdAt", "updatedAt" from api_key''',
    "asset": '''select id, "ownerId", "libraryId", type, "originalPath", "originalFileName",
                encode(checksum, 'hex') as checksum, "checksumAlgorithm", visibility,
                "isFavorite", "isOffline", "isExternal", "livePhotoVideoId", "stackId",
                "duplicateId", duration, encode(thumbhash, 'hex') as thumbhash,
                "fileCreatedAt", "fileModifiedAt", "localDateTime", "createdAt", "updatedAt",
                "deletedAt", width, height, "isEdited", status from asset''',
    "asset_exif": '''select "assetId", make, model, "exifImageWidth", "exifImageHeight",
                    "fileSizeInByte", orientation, "dateTimeOriginal", "modifyDate", "timeZone",
                    latitude, longitude, city, state, country, description, "fNumber",
                    "focalLength", iso, "exposureTime", "lensModel", "projectionType", rating,
                    fps, "livePhotoCID", "profileDescription", colorspace, "bitsPerSample",
                    "autoStackId" from asset_exif''',
    "asset_file": '''select id, "assetId", type, path, "createdAt", "updatedAt", "isEdited",
                    "isProgressive", "isTransparent" from asset_file''',
    "album": '''select a.id, coalesce(au."userId", null) as "ownerId", a."albumName",
                a.description, a."albumThumbnailAssetId", a."isActivityEnabled", a."order",
                a."createdAt", a."updatedAt", a."deletedAt"
                from album a
                left join lateral (
                  select "userId" from album_user where "albumId" = a.id order by "createdAt" limit 1
                ) au on true''',
    "album_asset": '''select "albumId", "assetId", "createdAt", "updatedAt" from album_asset''',
    "album_user": '''select "albumId", "userId", role, "createdAt", "updatedAt" from album_user''',
    "memory": '''select id, "ownerId", type, data, "memoryAt", "isSaved", "seenAt", "showAt",
                "hideAt", "createdAt", "updatedAt", "deletedAt" from memory''',
    "memory_asset": '''select "memoriesId" as "memoryId", "assetId" from memory_asset''',
    "shared_link": '''select id, "userId", encode(key, 'hex') as key, slug, type, "albumId",
                     description, password, "allowUpload", "allowDownload", "showExif",
                     "expiresAt", "createdAt" from shared_link''',
    "shared_link_asset": '''select "sharedLinkId", "assetId" from shared_link_asset''',
    "partner": '''select "sharedById", "sharedWithId", "inTimeline", "createdAt" from partner''',
    "system_metadata": '''select key, value from system_metadata''',
}

HEX_COLUMNS = {
    ("session", "token"),
    ("api_key", "key"),
    ("asset", "checksum"),
    ("asset", "thumbhash"),
    ("shared_link", "key"),
}

UUID_COLUMNS = {
    ("user", "id"),
    ("session", "id"),
    ("session", "userId"),
    ("api_key", "id"),
    ("api_key", "userId"),
    ("asset", "id"),
    ("asset", "ownerId"),
    ("asset", "libraryId"),
    ("asset", "livePhotoVideoId"),
    ("asset", "stackId"),
    ("asset", "duplicateId"),
    ("asset_exif", "assetId"),
    ("asset_exif", "autoStackId"),
    ("asset_file", "id"),
    ("asset_file", "assetId"),
    ("album", "id"),
    ("album", "ownerId"),
    ("album", "albumThumbnailAssetId"),
    ("album_asset", "albumId"),
    ("album_asset", "assetId"),
    ("album_user", "albumId"),
    ("album_user", "userId"),
    ("memory", "id"),
    ("memory", "ownerId"),
    ("memory_asset", "memoryId"),
    ("memory_asset", "assetId"),
    ("shared_link", "id"),
    ("shared_link", "userId"),
    ("shared_link", "albumId"),
    ("shared_link_asset", "sharedLinkId"),
    ("shared_link_asset", "assetId"),
    ("partner", "sharedById"),
    ("partner", "sharedWithId"),
    ("tag", "id"),
    ("tag", "userId"),
    ("tag", "parentId"),
    ("tag_asset", "tagId"),
    ("tag_asset", "assetId"),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default="immich", help="PostgreSQL database name")
    parser.add_argument("--psql-user", default="postgres", help="OS user used to run psql")
    parser.add_argument("--output", default="/opt/usr/local/domus/data/domus.sqlite3")
    parser.add_argument("--backup-root", default="/opt/usr/local/domus/data/backups")
    return parser.parse_args()


def now_stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def run_psql_json(db: str, psql_user: str, sql: str) -> Iterable[dict]:
    json_sql = f"select row_to_json(q)::text from ({sql}) q"
    cmd = ["sudo", "-u", psql_user, "psql", "-d", db, "-At", "-P", "pager=off", "-c", json_sql]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True)
    assert proc.stdout is not None
    for line in proc.stdout:
        line = line.strip()
        if line:
            yield json.loads(line)
    code = proc.wait()
    if code != 0:
        raise subprocess.CalledProcessError(code, cmd)


def create_schema(conn: sqlite3.Connection) -> None:
    conn.execute("pragma journal_mode=wal")
    conn.execute("pragma foreign_keys=off")
    for table, columns in CORE_TABLES.items():
        conn.execute(f'drop table if exists "{table}"')
        conn.execute(f'create table "{table}" ({columns})')
    conn.execute('create index if not exists "idx_asset_owner_local" on asset(ownerId, localDateTime)')
    conn.execute('create index if not exists "idx_asset_file_asset_type" on asset_file(assetId, type)')
    conn.execute('create index if not exists "idx_session_token" on session(token)')
    conn.execute('create index if not exists "idx_api_key_key" on api_key(key)')
    conn.execute('create index if not exists "idx_job_claim" on job(queue, status, runAt)')


def normalize_value(table: str, key: str, value):
    if value is None:
        return None
    if (table, key) in UUID_COLUMNS:
        return uuid.UUID(str(value)).bytes
    if (table, key) in HEX_COLUMNS:
        return bytes.fromhex(value) if value else None
    if isinstance(value, (dict, list)):
        return json.dumps(value, separators=(",", ":"), ensure_ascii=False)
    if isinstance(value, bool):
        return 1 if value else 0
    return value


def insert_rows(conn: sqlite3.Connection, table: str, rows: Iterable[dict]) -> int:
    count = 0
    columns = None
    statement = None
    for row in rows:
        if columns is None:
            columns = list(row.keys())
            quoted = ", ".join(f'"{c}"' for c in columns)
            placeholders = ", ".join("?" for _ in columns)
            statement = f'insert into "{table}" ({quoted}) values ({placeholders})'
        values = [normalize_value(table, key, row.get(key)) for key in columns]
        conn.execute(statement, values)
        count += 1
        if count % 5000 == 0:
            conn.commit()
    conn.commit()
    return count


def backup_existing_sqlite(path: Path, backup_root: Path) -> None:
    if not path.exists():
        return
    target = backup_root / f"sqlite-before-migrate-{now_stamp()}"
    target.mkdir(parents=True, exist_ok=True)
    backup_path = target / path.name
    source = sqlite3.connect(path)
    dest = sqlite3.connect(backup_path)
    with dest:
        source.backup(dest)
    source.close()
    dest.close()
    (target / "manifest.txt").write_text("pre-migration sqlite backup\n", encoding="utf-8")


def write_manifest(conn: sqlite3.Connection, out_dir: Path) -> None:
    lines = []
    for table in ["user", "asset", "album", "memory", "asset_file"]:
        count = conn.execute(f'select count(*) from "{table}"').fetchone()[0]
        lines.append(f"{table}_count|{count}")
    result = conn.execute("pragma integrity_check").fetchone()[0]
    lines.append(f"integrity|{result}")
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "sqlite-manifest.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    output = Path(args.output)
    backup_root = Path(args.backup_root)
    output.parent.mkdir(parents=True, exist_ok=True)
    backup_root.mkdir(parents=True, exist_ok=True)
    backup_existing_sqlite(output, backup_root)

    fd, tmp_name = tempfile.mkstemp(prefix=f".{output.name}.", suffix=".tmp", dir=output.parent)
    os.close(fd)
    tmp = Path(tmp_name)
    try:
        conn = sqlite3.connect(tmp)
        create_schema(conn)
        counts = {}
        for table, sql in SELECTS.items():
            counts[table] = insert_rows(conn, table, run_psql_json(args.db, args.psql_user, sql))
            print(f"{table}: {counts[table]}", flush=True)
        conn.commit()
        write_manifest(conn, backup_root / f"sqlite-migrate-{now_stamp()}")
        conn.close()
        tmp.replace(output)
        print(output)
        return 0
    except Exception:
        tmp.unlink(missing_ok=True)
        raise


if __name__ == "__main__":
    sys.exit(main())
