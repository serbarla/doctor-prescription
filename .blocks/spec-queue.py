#!/usr/bin/env python3
# /// block
# name = "spec-queue"
# summary = "Classify unchecked specs and divergence slices into a ready, blocked, or conflict dispatch queue."
# effects = "read"
# example = ["--specs-dir", "specs", "--brief"]
# matches = ['specs/divergences', 'Owned files:', 'fail-if-empty']
#
# [returns]
# keys = ["ok", "queue", "counts"]
# doc = "queue rows carry id, path, kind, status (ready|blocked|conflict), blocked_by, owned_files, gate, desktop_verify, whole_spec and unchecked; ready means no unchecked dependency and no declared owned-files overlap with another ready row"
#
# [params]
# specs-dir = { type = "path", default = "specs", help = "Specs directory holding *.md and divergences/*.md" }
# ready-only = { type = "bool", default = false, help = "Print only dispatchable ready rows in queue" }
# fail-if-empty = { type = "bool", default = false, help = "Exit non-zero when no row is dispatchable" }
# brief = { type = "bool", default = false, help = "Print id, status, and blocked_by only; keeps output under the 2 KB run cap" }
#
# [provenance]
# created = "2026-11-17"
# source = "specs/15-O-spec-automation-pipeline.md objective 2"
#
# [stamp]
# verified = "27a33363c2ac"
# ///
"""Classify spec and divergence slices into a dispatch queue and print one JSON object."""
import argparse
import fnmatch
import json
import re
import sys
from pathlib import Path

TEMPLATE = "TEMPLATE.md"
LEGACY_GATE = "python3 .blocks/persona-smoke.py --dir persona-sanus --port {port} --db-path {db_path}"
FIELD_RE = re.compile(r"^(?:-\s*)?(source specs?|source spec|depends on|owned files|gate|desktop-verify)\s*:\s*(.*)$", re.I)
BOX_RE = re.compile(r"^\s*[-*]\s*\[ \]")
RANGE_RE = re.compile(r"(\d{2})-([A-Z])\s*[\u2013\u2014]\s*(\d{2})-([A-Z])")
DEP_SPLIT_RE = re.compile(r"\s*(?:,|;|\band\b)\s*", re.I)
EXTERNAL_SPEC_RE = re.compile(r"ship signed|signed,?\s+notarized", re.I)
EXTERNAL_DIV_RE = re.compile(
    r"no apple developer id|no windows code-signing|release pipeline must hold the certificates|"
    r"signing/notarization credentials .*external",
    re.I,
)


def clean(value):
    return value.strip().strip("`").strip().strip("*").strip().rstrip(".")


def header_fields(lines):
    fields = {}
    for line in lines[:40]:
        match = FIELD_RE.match(line)
        if not match:
            continue
        key = match.group(1).lower()
        value = match.group(2).strip()
        if key in {"source spec", "source specs"}:
            fields.setdefault("source", value)
            continue
        fields[key] = value
    return fields


def objective_boxes(lines):
    boxes = []
    inside = False
    for line in lines:
        if re.match(r"^##\s+1\.\s+Objectives\b", line):
            inside = True
            continue
        if inside and line.startswith("## "):
            break
        if inside and BOX_RE.match(line):
            boxes.append(line)
    return boxes


def all_boxes(lines):
    return [line for line in lines if BOX_RE.match(line)]


def split_deps(value):
    value = value.replace("`", "")
    tokens = []
    for part in DEP_SPLIT_RE.split(value):
        part = clean(part)
        if not part or part.lower() in {"nothing", "none"}:
            continue
        match = RANGE_RE.search(part)
        if match:
            start, end = int(match.group(1)), int(match.group(3))
            if start <= end:
                tokens.extend(f"{number:02d}" for number in range(start, end + 1))
            continue
        tokens.append(part.split()[0].strip(".,;:()"))
    return tokens


def owned_files(fields):
    raw = fields.get("owned files", "")
    if not raw:
        return ["*"], True
    items = [clean(item) for item in raw.split(",")]
    return [item for item in items if item], False


def gate_value(fields):
    raw = clean(fields.get("gate", ""))
    return raw or LEGACY_GATE


def source_spec(fields):
    raw = fields.get("source", "")
    match = re.search(r"`([^`]+\.md)`", raw)
    return Path(match.group(1)).stem if match else ""


def display_path(path):
    try:
        return path.resolve().relative_to(Path.cwd().resolve()).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def build_row(path, kind):
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    fields = header_fields(lines)
    boxes = objective_boxes(lines) if kind == "spec" else all_boxes(lines)
    owned, whole_spec = owned_files(fields)
    return {
        "id": path.stem,
        "path": display_path(path),
        "kind": kind,
        "status": "ready",
        "blocked_by": [],
        "owned_files": owned,
        "gate": gate_value(fields),
        "desktop_verify": clean(fields.get("desktop-verify", "")).lower() == "yes",
        "whole_spec": whole_spec,
        "unchecked": len(boxes),
        "deps": split_deps(fields.get("depends on", "")),
        "source_spec": source_spec(fields),
        "text": text,
        "objectives": boxes,
    }


def dep_matches(dep, stem):
    if dep == stem:
        return True
    if re.fullmatch(r"\d{2}", dep):
        return stem.startswith(dep + "-")
    return stem.startswith(dep + "-")


def dependency_complete(dep, rows, done_stems, current):
    active = [row for row in rows if row["kind"] == "spec" and row is not current and dep_matches(dep, row["id"])]
    if active:
        return all(row["unchecked"] == 0 for row in active)
    return any(dep_matches(dep, stem) for stem in done_stems)


def external_divergence(row, rows):
    if not EXTERNAL_DIV_RE.search(row["text"]) or not row["source_spec"]:
        return False
    source = next((item for item in rows if item["id"] == row["source_spec"]), None)
    if source is None:
        return False
    reference = "specs/divergences/" + row["id"] + ".md"
    return reference in source["text"] and "OPEN" in source["text"]


def external_spec(row):
    if row["kind"] != "spec":
        return False
    for objective in row["objectives"]:
        if EXTERNAL_SPEC_RE.search(objective) and re.search(r"external prerequisite", objective, re.I):
            if re.search(r"credential|certificate|developer id", objective, re.I):
                return True
    return False


def patterns_overlap(left, right):
    left = left.lstrip("./")
    right = right.lstrip("./")
    if left == right:
        return True
    if left in {"*", "**"} or right in {"*", "**"}:
        return True
    if left.endswith("/") and right.startswith(left):
        return True
    if right.endswith("/") and left.startswith(right):
        return True
    if any(char in left for char in "*?[") and fnmatch.fnmatch(right, left):
        return True
    if any(char in right for char in "*?[") and fnmatch.fnmatch(left, right):
        return True
    return False


def classify(rows, done_stems):
    ready = []
    for row in rows:
        if row["unchecked"] == 0:
            row["status"] = "blocked"
            row["blocked_by"] = ["complete"]
            continue
        blockers = []
        for dep in row["deps"]:
            if not dependency_complete(dep, rows, done_stems, row):
                blockers.append(dep)
        if blockers:
            row["status"] = "blocked"
            row["blocked_by"] = blockers
            continue
        ready.append(row)

    for row in ready:
        if row["kind"] == "divergence" and external_divergence(row, rows):
            row["status"] = "blocked"
            row["blocked_by"] = ["external:signing-credentials"]

    for row in ready:
        if row["status"] == "ready" and external_spec(row):
            row["status"] = "blocked"
            row["blocked_by"] = ["external:signing-and-notarization-credentials"]

    ready = [row for row in ready if row["status"] == "ready"]
    conflicted = {}
    declared = [row for row in ready if not row["whole_spec"]]
    for index, row in enumerate(declared):
        for other in declared[index + 1:]:
            if any(patterns_overlap(a, b) for a in row["owned_files"] for b in other["owned_files"]):
                conflicted.setdefault(row["id"], set()).add(other["id"])
                conflicted.setdefault(other["id"], set()).add(row["id"])
    for row in ready:
        if row["id"] in conflicted:
            row["status"] = "conflict"
            row["blocked_by"] = sorted(conflicted[row["id"]])
    return rows


def read_rows(specs_dir):
    rows = []
    for path in sorted(specs_dir.glob("*.md")):
        if path.name == TEMPLATE:
            continue
        rows.append(build_row(path, "spec"))
    divergence_dir = specs_dir / "divergences"
    if divergence_dir.is_dir():
        for path in sorted(divergence_dir.glob("*.md")):
            rows.append(build_row(path, "divergence"))
    done_stems = []
    done_dir = specs_dir / "done"
    if done_dir.is_dir():
        done_stems = [path.stem for path in sorted(done_dir.glob("*.md"))]
    return rows, done_stems


def compact(row):
    keys = ["id", "path", "kind", "status", "blocked_by", "owned_files", "gate", "desktop_verify", "whole_spec", "unchecked"]
    return {key: row[key] for key in keys}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--specs-dir", default="specs")
    parser.add_argument("--ready-only", action="store_true")
    parser.add_argument("--fail-if-empty", action="store_true")
    parser.add_argument("--brief", action="store_true")
    args = parser.parse_args()

    specs_dir = Path(args.specs_dir)
    if not specs_dir.is_dir():
        print(json.dumps({"ok": False, "error": "specs_dir_not_found", "specs_dir": str(specs_dir)}, separators=(",", ":")))
        return 2

    rows, done_stems = read_rows(specs_dir)
    rows = classify(rows, done_stems)
    ready = [row for row in rows if row["status"] == "ready"]
    counts = {
        "total": len(rows),
        "ready": len(ready),
        "blocked": sum(1 for row in rows if row["status"] == "blocked"),
        "conflict": sum(1 for row in rows if row["status"] == "conflict"),
        "specs": sum(1 for row in rows if row["kind"] == "spec"),
        "divergences": sum(1 for row in rows if row["kind"] == "divergence"),
    }
    if args.fail_if_empty and not ready:
        print(json.dumps({"ok": False, "error": "no_ready_slices", "counts": counts}, separators=(",", ":")))
        return 1
    queue = [compact(row) for row in rows if not args.ready_only or row["status"] == "ready"]
    if args.brief:
        queue = [{key: row[key] for key in ["id", "status"]} for row in queue]
    print(json.dumps({"ok": True, "queue": queue, "counts": counts}, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    sys.exit(main())
