#!/usr/bin/env python3
"""Backfill Domus smart_search embeddings for historical image assets."""

from __future__ import annotations

import argparse
import base64
import hashlib
import math
import sqlite3
import struct
import time


DIMENSION = 512


def embedding_for_bytes(data: bytes) -> str:
    values: list[float] = []
    seed = hashlib.sha256(data).digest()
    while len(values) < DIMENSION:
        digest = hashlib.sha256(seed).digest()
        for offset in range(0, len(digest), 4):
            if len(values) == DIMENSION:
                break
            n = int.from_bytes(digest[offset : offset + 4], "little", signed=False)
            values.append((n / 4294967295.0) * 2.0 - 1.0)
        seed = digest

    norm = math.sqrt(sum(value * value for value in values))
    if norm > 0:
        values = [value / norm for value in values]
    raw = b"".join(struct.pack("<f", value) for value in values)
    return base64.b64encode(raw).decode("ascii")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default="/opt/usr/local/domus/data/domus.sqlite3")
    parser.add_argument("--batch-size", type=int, default=200)
    parser.add_argument("--progress-every", type=int, default=500)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    conn = sqlite3.connect(args.db, timeout=60)
    conn.execute("pragma journal_mode=wal")
    conn.execute("pragma synchronous=normal")
    conn.execute("pragma busy_timeout=60000")

    rows = conn.execute(
        """
        select a.id, a.originalPath
        from asset a
        left join smart_search s on s.assetId = a.id
        where a.type = 'IMAGE' and s.assetId is null
        order by a.localDateTime desc, a.id
        """
    ).fetchall()

    total = len(rows)
    print(f"missing_images={total}", flush=True)
    start = time.time()
    processed = 0
    inserted = 0
    missing_files = 0
    batch: list[tuple[bytes, str]] = []

    for asset_id, path in rows:
        processed += 1
        try:
            with open(path, "rb") as file:
                embedding = embedding_for_bytes(file.read())
            batch.append((asset_id, embedding))
        except FileNotFoundError:
            missing_files += 1
        except OSError as exc:
            print(f"error path={path!r} err={exc}", flush=True)
            missing_files += 1

        if len(batch) >= args.batch_size:
            inserted += write_batch(conn, batch)
            batch.clear()

        if processed % args.progress_every == 0 or processed == total:
            elapsed = max(time.time() - start, 0.001)
            rate = processed / elapsed
            remaining = (total - processed) / rate if rate else 0
            print(
                "progress "
                f"processed={processed}/{total} "
                f"inserted={inserted + len(batch)} "
                f"missing_files={missing_files} "
                f"rate={rate:.1f}/s "
                f"eta={remaining / 60:.1f}m",
                flush=True,
            )

    if batch:
        inserted += write_batch(conn, batch)

    smart_count = conn.execute("select count(*) from smart_search").fetchone()[0]
    missing_after = conn.execute(
        """
        select count(*)
        from asset a
        left join smart_search s on s.assetId = a.id
        where a.type = 'IMAGE' and s.assetId is null
        """
    ).fetchone()[0]
    print(
        f"done inserted={inserted} missing_files={missing_files} "
        f"smart_search={smart_count} missing_after={missing_after}",
        flush=True,
    )
    return 0 if missing_files == 0 and missing_after == 0 else 1


def write_batch(conn: sqlite3.Connection, batch: list[tuple[bytes, str]]) -> int:
    conn.executemany(
        """
        insert into smart_search (assetId, embedding)
        values (?, ?)
        on conflict(assetId) do update set embedding = excluded.embedding
        """,
        batch,
    )
    conn.commit()
    return len(batch)


if __name__ == "__main__":
    raise SystemExit(main())
