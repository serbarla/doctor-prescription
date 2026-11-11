#!/usr/bin/env python3
# /// block
# name = "markdown-table-check"
# summary = "Verify every Markdown table has one consistent column count per table; report the offenders."
# effects = "read"
# example = ["--path", "$FIXTURES/tables"]
# matches = ['table.*column.*count', 'fenced', 're\.split\(']
#
# [returns]
# keys = ["ok", "files", "tables", "mismatches", "truncated"]
# doc = "ok is false when any row differs from its table's column count; mismatches are file:line entries with the expected and actual counts, capped by --report; files and tables are scanned counts"
#
# [params]
# path = { type = "path", required = true, help = "Markdown file, or directory scanned recursively" }
# glob = { type = "str", default = "*.md", help = "Filename pattern when path is a directory" }
# skip = { type = "str", default = "fixtures/", help = "Substring of paths to ignore; empty disables" }
# report = { type = "int", default = 5, min = 1, help = "Maximum mismatches printed" }
# strict = { type = "bool", default = false, help = "Exit 1 when a mismatch is found" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: context/ doc table validation"
#
# [stamp]
# verified = "294eb070f125"
# ///
"""Check Markdown tables for inconsistent column counts, ignoring fenced code blocks."""
import argparse
import fnmatch
import json
import re
import sys
from pathlib import Path

CODE_SPAN = re.compile(r"`[^`]*`")


def split_row(line):
    text = line.strip()
    stripped = CODE_SPAN.sub("", text)
    if not stripped.startswith("|"):
        return None
    body = stripped[1:]
    if body.endswith("|"):
        body = body[:-1]
    cells = re.split(r"(?<!\\)\|", body)
    return [cell.strip() for cell in cells]


def is_divider(cells):
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells if cell) and any(cells)


def scan(path, pattern, skip):
    mismatches = []
    files = 0
    tables = 0
    targets = [path] if path.is_file() else sorted(p for p in path.rglob("*") if p.is_file() and fnmatch.fnmatch(p.name, pattern))
    if skip and not path.is_file():
        targets = [t for t in targets if skip not in t.relative_to(path).as_posix()]
    for target in targets:
        files += 1
        fenced = False
        expected = None
        for number, raw in enumerate(target.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            line = raw.strip()
            if line.startswith("```") or line.startswith("~~~"):
                fenced = not fenced
                expected = None
                continue
            if fenced:
                continue
            cells = split_row(raw)
            if cells is None:
                expected = None
                continue
            if expected is None:
                expected = len(cells)
                tables += 1
                continue
            if len(cells) != expected:
                mismatches.append(f"{target}:{number} cols={len(cells)} expected={expected}")
    return files, tables, mismatches


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--path", required=True)
    parser.add_argument("--glob", default="*.md")
    parser.add_argument("--skip", default="fixtures/")
    parser.add_argument("--report", type=int, default=5)
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    target = Path(args.path)
    if not target.exists():
        print(json.dumps({"error": f"path not found: {args.path}"}))
        return 2

    files, tables, mismatches = scan(target, args.glob, args.skip)
    shown = mismatches[: args.report]
    print(json.dumps({
        "ok": not mismatches,
        "files": files,
        "tables": tables,
        "mismatches": shown,
        "truncated": len(mismatches) > len(shown),
    }))
    if args.strict and mismatches:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
