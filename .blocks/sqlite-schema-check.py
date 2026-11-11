#!/usr/bin/env python3
# /// block
# name = "sqlite-schema-check"
# summary = "Run a Markdown fenced SQL block in an in-memory SQLite DB, check integrity and optionally query it."
# effects = "exec"
# example = ["--path", "context/database-design.md", "--query", "select code, amount_cents, interval_months from plans order by code"]
# matches = ['awk.*```sql', 'sqlite3 .*<', 'sqlite3 .*integrity_check']
#
# [returns]
# keys = ["ok", "path", "block", "tables", "integrity", "query_columns", "query_rows", "truncated"]
# doc = "block is the zero-based fenced-sql block index; ok is true only when integrity_check returns ok; query fields appear only with --query"
#
# [params]
# path = { type = "path", required = true, help = "Markdown file with at least one fenced ```sql block" }
# block = { type = "int", default = 0, min = 0, help = "Zero-based index of the fenced SQL block to run" }
# query = { type = "str", default = "", help = "Optional SQL to run after the schema is created" }
# limit = { type = "int", default = 10, min = 1, max = 100, help = "Maximum query rows returned" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: repeated schema validation for context/database-design.md"
#
# [stamp]
# verified = "d5b87f5dfe6f"
# ///
"""Run a fenced SQL block from Markdown in an in-memory SQLite database and print one small JSON verdict."""
import argparse
import json
import re
import sqlite3
import sys
from pathlib import Path

FENCE = re.compile(r"^```sql[ \t]*\r?\n(.*?)^```[ \t]*$", re.MULTILINE | re.DOTALL)
MAX_VALUE = 160


def value_cell(value):
    """Return a JSON-safe, length-capped cell."""
    if isinstance(value, bytes):
        value = value.hex()
    elif not isinstance(value, (str, int, float, bool, type(None))):
        value = json.dumps(value, ensure_ascii=False, default=str)
    if isinstance(value, str) and len(value) > MAX_VALUE:
        return value[:MAX_VALUE] + "..."
    return value


def check(path, block, query, limit):
    text = Path(path).read_text(encoding="utf-8")
    blocks = FENCE.findall(text)
    if not blocks:
        raise ValueError(f"no fenced ```sql block in {path}")
    if block >= len(blocks):
        raise ValueError(f"block {block} out of range; file has {len(blocks)}")
    db = sqlite3.connect(":memory:")
    try:
        db.executescript(blocks[block])
        integrity = db.execute("PRAGMA integrity_check").fetchone()[0]
        tables = db.execute(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        ).fetchone()[0]
        answer = {
            "ok": integrity == "ok",
            "path": path,
            "block": block,
            "tables": tables,
            "integrity": integrity,
        }
        if query:
            cur = db.execute(query)
            columns = [column[0] for column in cur.description or []]
            rows = cur.fetchmany(limit + 1)
            truncated = len(rows) > limit
            answer["query_columns"] = columns
            answer["query_rows"] = [[value_cell(cell) for cell in row] for row in rows[:limit]]
            answer["truncated"] = truncated
        if not answer["ok"]:
            answer["error"] = f"integrity_check: {integrity}"
        return answer
    finally:
        db.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--path", required=True, help="Markdown file with fenced SQL")
    parser.add_argument("--block", type=int, default=0, help="Zero-based fenced SQL block index")
    parser.add_argument("--query", default="", help="Optional SQL to run after schema creation")
    parser.add_argument("--limit", type=int, default=10, help="Maximum query rows (1-100)")
    args = parser.parse_args()
    if args.block < 0:
        parser.error("--block must be >= 0")
    if not 1 <= args.limit <= 100:
        parser.error("--limit must be between 1 and 100")
    return check(args.path, args.block, args.query, args.limit)


if __name__ == "__main__":
    try:
        result = main()
    except (OSError, ValueError, sqlite3.Error) as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
