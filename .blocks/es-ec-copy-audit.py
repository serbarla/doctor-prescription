#!/usr/bin/env python3
# /// block
# name = "es-ec-copy-audit"
# summary = "Scan Rust sources for English UI literals not accept-listed as machine values."
# effects = "read"
# example = ["--src", "persona-sanus/src", "--files", "main.rs,http.rs,auth_server.rs,service.rs,api_server.rs,api.rs,forms.rs"]
# matches = ['English.*copia|copy.*audit|es-EC.*scan']
#
# [returns]
# keys = ["ui_hits", "accepted", "files", "literals"]
# doc = "ui_hits lists English literals outside the machine-value accept-list; exit 0 iff empty"
#
# [params]
# src = { type = "str", default = "persona-sanus/src", help = "Source directory to scan for .rs files" }
# files = { type = "str", default = "main.rs,http.rs,auth_server.rs,service.rs,api_server.rs,api.rs,forms.rs", help = "Comma-separated .rs filenames to scan" }
#
# [provenance]
# created = "2026-11-20"
# source = "promoted: tracker 13-M es-EC copy audit"
#
# [stamp]
# verified = "placeholder"
# ///
"""Flag English UI literals in Rust sources; machine values are accept-listed."""
import argparse
import json
import re
import sys
from pathlib import Path

ENGLISH_WORDS = {
    "the", "and", "with", "for", "from", "this", "that", "your", "you", "please",
    "must", "should", "already", "exists", "failed", "loading", "save", "delete",
    "remove", "create", "update", "search", "filter", "export", "close", "cancel",
    "confirm", "back", "next", "previous", "unknown", "invalid", "valid", "required",
    "available", "first", "only", "own", "allowed", "denied", "access", "session",
    "dashboard", "home", "settings", "history", "recent", "verify", "print",
    "download", "upload", "image", "file", "missing", "empty", "wrong", "try",
    "cannot", "unable", "contact", "connection", "network", "offline", "online",
    "month", "year", "day", "week", "user", "patient", "template",
    "prescription", "clinic", "member", "device", "profile", "account", "password",
    "email", "name", "code", "message", "request", "response", "server", "enter",
    "sign", "welcome", "fill", "redirecting", "too", "long", "short", "date",
    "birth", "sex", "gender", "phone", "address", "status", "role",
}

ACCEPTED = {
    "draft", "published", "archived", "active", "disabled", "signed", "superseded",
    "voided", "pending_sync", "failed", "canceled", "pending", "approved",
    "reversed", "female", "male", "other", "unspecified", "macos", "windows",
    "web", "administrator", "doctor", "nursing", "secretary", "America/Guayaquil",
    "persona_session", "mfaRequired", "mfaTicket", "exhausted", "not found",
    "Acción (p. ej. sensitive_read)", "Entidad (p. ej. patient)",
}

LITERAL = re.compile(r'"([^"\\]*(?:\\.[^"\\]*)*)"')
WORD = re.compile(r"[a-zA-Z]+")
PLACEHOLDER = re.compile(r"\{[^}]*\}")


def scan(path):
    """Yield (line_number, literal) for each string literal in a Rust file."""
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        for match in LITERAL.finditer(line):
            yield lineno, match.group(1)


def is_english(literal):
    """Return True when the literal contains an English word."""
    if literal in ACCEPTED:
        return False
    if not any(char.islower() for char in literal):
        return False
    if not any(char.isupper() for char in literal) and " " not in literal:
        return False
    if "/" in literal and " " not in literal:
        return False
    text = PLACEHOLDER.sub(" ", literal)
    words = {word.lower() for word in WORD.findall(text)}
    return bool(words & ENGLISH_WORDS)


def audit(src, files):
    """Scan the named Rust files and return the audit summary dict."""
    hits = []
    accepted = 0
    literals = 0
    for name in files:
        path = src / name
        if not path.exists():
            continue
        for lineno, literal in scan(path):
            literals += 1
            if is_english(literal):
                hits.append({"file": name, "line": lineno, "literal": literal[:120]})
            elif literal in ACCEPTED:
                accepted += 1
    return {
        "files": len(files),
        "literals": literals,
        "accepted": accepted,
        "ui_hits": hits[:50],
        "ui_hit_count": len(hits),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--src", default="persona-sanus/src")
    parser.add_argument(
        "--files",
        default="main.rs,http.rs,auth_server.rs,service.rs,api_server.rs,api.rs,forms.rs",
    )
    args = parser.parse_args()
    files = [name.strip() for name in args.files.split(",") if name.strip()]
    return audit(Path(args.src), files)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result["ui_hit_count"] == 0 else 1)
