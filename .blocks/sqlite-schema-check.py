#!/usr/bin/env python3
# /// block
# name = "sqlite-schema-check"
# summary = "Run SQL from a Markdown fence, a .sql file, or a live SQLite DB; check integrity and query it."
# effects = "exec"
# example = ["--path", "context/database-design.md", "--query", "select code, amount_cents, interval_months from plans order by code"]
# matches = ['awk.*```sql', 'sqlite3 .*<', 'sqlite3 .*integrity_check']
#
# [returns]
# keys = ["ok", "path", "db", "block", "tables", "integrity", "query_columns", "query_rows", "truncated"]
# doc = "block is the zero-based fenced-sql block index for Markdown input; path is omitted with --db; ok is true only when integrity_check returns ok; query fields appear only with --query"
#
# [params]
# path = { type = "path", default = "", help = "Markdown file with a fenced ```sql block, or a .sql file run verbatim" }
# db = { type = "path", default = "", help = "Existing SQLite database file to open and query without running a script" }
# block = { type = "int", default = 0, min = 0, help = "Zero-based index of the fenced SQL block to run" }
# query = { type = "str", default = "", help = "Optional SQL to run after the script (or against --db)" }
# limit = { type = "int", default = 10, min = 1, max = 100, help = "Maximum query rows returned" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: repeated schema validation for context/database-design.md"
#
# [stamp]
# verified = "f2e4ad69efa7"
# ///
"""Run SQL from a fenced Markdown block or a .sql file, or query a live SQLite database, and print one small JSON verdict."""
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


def script_from(path, block):
    """Return (sql, fenced_block_or_none) for a .sql file or a Markdown fenced block."""
    text = Path(path).read_text(encoding="utf-8")
    if Path(path).suffix.lower() == ".sql":
        return text, None
    blocks = FENCE.findall(text)
    if not blocks:
        raise ValueError(f"no fenced ```sql block in {path}")
    if block >= len(blocks):
        raise ValueError(f"block {block} out of range; file has {len(blocks)}")
    return blocks[block], block


def check(path, db_path, block, query, limit):
    connection = sqlite3.connect(db_path or ":memory:")
    try:
        answer = {"ok": False, "db": db_path}
        if path:
            script, fenced = script_from(path, block)
            connection.executescript(script)
            answer["path"] = path
            if fenced is not None:
                answer["block"] = fenced
        integrity = connection.execute("PRAGMA integrity_check").fetchone()[0]
        tables = connection.execute(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        ).fetchone()[0]
        answer["ok"] = integrity == "ok"
        answer["tables"] = tables
        answer["integrity"] = integrity
        if query:
            cur = connection.execute(query)
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
        connection.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--path", default="", help="Markdown file with fenced SQL, or a .sql file")
    parser.add_argument("--db", default="", help="Existing SQLite database file to query")
    parser.add_argument("--block", type=int, default=0, help="Zero-based fenced SQL block index")
    parser.add_argument("--query", default="", help="Optional SQL to run after the script")
    parser.add_argument("--limit", type=int, default=10, help="Maximum query rows (1-100)")
    args = parser.parse_args()
    if not args.path and not args.db:
        parser.error("--path or --db is required")
    if args.path and args.db:
        parser.error("--path and --db cannot be combined")
    if args.block < 0:
        parser.error("--block must be >= 0")
    if not 1 <= args.limit <= 100:
        parser.error("--limit must be between 1 and 100")
    return check(args.path, args.db, args.block, args.query, args.limit)


if __name__ == "__main__":
    try:
        result = main()
    except (OSError, ValueError, sqlite3.Error) as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
