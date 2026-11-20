#!/usr/bin/env python3
# /// block
# name = "backup-verify"
# summary = "Verify backup file safety in docker compose: happy path, permissions, age-absent guard, stale sweep."
# effects = "external"
# example = ["--dir", ".", "--port", "8080"]
# matches = ['BACKUP_AGE_RECIPIENT', 'app-.*\\.db\\.age']
#
# [returns]
# keys = ["ok", "checks"]
# doc = "ok true when every backup safety check passes; on failure exits non-zero with error plus partial checks"
#
# [params]
# dir = { type = "path", default = ".", help = "compose project directory (repo root), must hold docker-compose.yml" }
# port = { type = "int", default = 8080, min = 1, max = 65535, help = "host port the stack publishes" }
#
# [provenance]
# created = "2026-11-19"
# source = "specs/divergences/13-M-backup-file-safety.md how-to-clear-it"
#
# [stamp]
# verified = "0db4104c8536"
# ///
"""Verify backup file safety; print one small JSON verdict."""
import argparse
import glob
import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time


def run(cmd, cwd, env=None, timeout=120):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    proc = subprocess.run(cmd, cwd=cwd, env=full_env, capture_output=True, text=True, timeout=timeout)
    return proc.returncode, proc.stdout, proc.stderr


def run_bytes(cmd, cwd, env=None, timeout=120):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    proc = subprocess.run(cmd, cwd=cwd, env=full_env, capture_output=True, timeout=timeout)
    return proc.returncode, proc.stdout, proc.stderr.decode("utf-8", "replace")


def compose(cwd, *args, env=None, timeout=180):
    return run(["docker", "compose", *args], cwd, env=env, timeout=timeout)


def compose_bytes(cwd, *args, env=None, timeout=180):
    return run_bytes(["docker", "compose", *args], cwd, env=env, timeout=timeout)


def dump_db(cwd, env):
    code, out, err = compose_bytes(cwd, "exec", "-T", "persona-sanus", "cat", "/data/app.db", env=env, timeout=60)
    if code != 0:
        return None
    tmpdir = tempfile.mkdtemp()
    tmp = os.path.join(tmpdir, "app.db")
    with open(tmp, "wb") as fh:
        fh.write(out)
    for suffix in ("-wal", "-shm"):
        code, out, err = compose_bytes(cwd, "exec", "-T", "persona-sanus", "cat", "/data/app.db" + suffix, env=env, timeout=60)
        if code == 0 and out:
            with open(tmp + suffix, "wb") as fh:
                fh.write(out)
    return tmpdir


def audit_latest(cwd, env):
    tmpdir = dump_db(cwd, env)
    if tmpdir is None:
        return ""
    try:
        conn = sqlite3.connect(os.path.join(tmpdir, "app.db"))
        try:
            row = conn.execute("SELECT metadata_json FROM audit_log WHERE action='backup_daily' ORDER BY rowid DESC LIMIT 1").fetchone()
            return row[0] if row else ""
        finally:
            conn.close()
    except Exception:
        return ""
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def audit_all(cwd, env):
    tmpdir = dump_db(cwd, env)
    if tmpdir is None:
        return []
    try:
        conn = sqlite3.connect(os.path.join(tmpdir, "app.db"))
        try:
            rows = conn.execute("SELECT metadata_json FROM audit_log WHERE action='backup_daily' ORDER BY rowid DESC LIMIT 20").fetchall()
            return [row[0] for row in rows]
        finally:
            conn.close()
    except Exception:
        return []
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def wait_audit(cwd, env, needle, timeout=30):
    started = time.monotonic()
    while time.monotonic() - started < timeout:
        audits = audit_all(cwd, env)
        if any(needle.lower() in entry.lower() for entry in audits):
            return audits
        time.sleep(2)
    return audit_all(cwd, env)


def decrypt_ok(cwd, env, backups):
    files = sorted(glob.glob(os.path.join(backups, "app-*.db.age")))
    if not files:
        return False
    inside = "/backups/" + os.path.basename(files[0])
    code, out, err = compose_bytes(cwd, "exec", "-T", "persona-sanus", "age", "-d", "-i", "/backups/age-key.txt", inside, env=env, timeout=60)
    if code != 0:
        return False
    if not out.startswith(b"SQLite format 3"):
        return False
    with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as tf:
        tf.write(out)
        tmp = tf.name
    try:
        conn = sqlite3.connect(tmp)
        try:
            row = conn.execute("PRAGMA integrity_check").fetchone()
            return bool(row and row[0] == "ok")
        finally:
            conn.close()
    except Exception:
        return False
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


def wait_for(backups, timeout=120):
    started = time.monotonic()
    while time.monotonic() - started < timeout:
        if glob.glob(os.path.join(backups, "app-*.db.age")):
            return True
        time.sleep(2)
    return False


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def plain_left(backups):
    left = []
    for pattern in ("*", ".*"):
        for path in glob.glob(os.path.join(backups, pattern)):
            if os.path.isfile(path) and (path.endswith(".db") or path.endswith(".db.tmp")):
                left.append(path)
    return sorted(set(left))


def verify(project_dir, port):
    checks = {}
    backups = os.path.join(project_dir, "backups")
    base_env = {"PERSONA_BACKUP_DIR": "./backups", "BACKUP_INTERVAL_SECONDS": "3"}
    if os.path.isdir(backups):
        shutil.rmtree(backups)
    os.makedirs(backups, exist_ok=True)
    os.chmod(backups, 0o755)
    try:
        compose(project_dir, "down", "-v", timeout=120)
        code, out, err = compose(project_dir, "build", timeout=1800)
        checks["built"] = code == 0
        if not checks["built"]:
            return {"error": "compose build failed: " + (out + err)[-500:], "checks": checks}
        code, out, err = compose(project_dir, "up", "-d", env=base_env, timeout=300)
        checks["deployed"] = code == 0
        if not checks["deployed"]:
            return {"error": "compose up failed", "checks": checks}
        time.sleep(8)
        code, out, err = compose(project_dir, "exec", "-T", "persona-sanus", "age-keygen", "-o", "/backups/age-key.txt", env=base_env, timeout=60)
        checks["keygen"] = code == 0
        pub = ""
        key_path = os.path.join(backups, "age-key.txt")
        if os.path.exists(key_path):
            for line in open(key_path):
                if "public key:" in line:
                    pub = line.split("public key:")[1].strip()
        checks["recipient"] = bool(pub.startswith("age1"))
        if not checks["recipient"]:
            return {"error": "no age recipient generated", "checks": checks}
        env = dict(base_env)
        env["BACKUP_AGE_RECIPIENT"] = pub
        compose(project_dir, "up", "-d", "--force-recreate", env=env, timeout=300)
        checks["happy_age_created"] = wait_for(backups, 120)
        checks["dir_mode_0700"] = (os.stat(backups).st_mode & 0o777) == 0o700
        age_files = glob.glob(os.path.join(backups, "app-*.db.age"))
        checks["file_mode_0600"] = bool(age_files) and (os.stat(age_files[0]).st_mode & 0o777) == 0o600
        checks["happy_decrypts"] = decrypt_ok(project_dir, env, backups)
        checks["happy_no_plain"] = plain_left(backups) == []
        first_hash = sha(age_files[0]) if age_files else ""
        atomic_ok = True
        started = time.monotonic()
        while time.monotonic() - started < 12:
            if not decrypt_ok(project_dir, env, backups):
                atomic_ok = False
                break
            time.sleep(2)
        checks["atomic_rename_always_valid"] = atomic_ok
        second_ok = False
        started = time.monotonic()
        while time.monotonic() - started < 30:
            age_files = glob.glob(os.path.join(backups, "app-*.db.age"))
            if age_files and sha(age_files[0]) != first_hash and decrypt_ok(project_dir, env, backups):
                second_ok = True
                break
            time.sleep(2)
        checks["second_backup_decrypts"] = second_ok
        code, out, err = compose(project_dir, "exec", "-T", "persona-sanus", "mv", "/usr/bin/age", "/usr/bin/age.bak", env=env, timeout=60)
        checks["age_hidden"] = code == 0
        compose(project_dir, "restart", "persona-sanus", env=env, timeout=120)
        time.sleep(15)
        checks["age_absent_no_plain"] = plain_left(backups) == []
        audit = audit_latest(project_dir, env)
        checks["age_absent_audits_error"] = "error" in audit
        checks["age_absent_audit"] = audit[:200]
        compose(project_dir, "exec", "-T", "persona-sanus", "mv", "/usr/bin/age.bak", "/usr/bin/age", env=env, timeout=60)
        compose(project_dir, "restart", "persona-sanus", env=env, timeout=120)
        time.sleep(10)
        checks["age_restored_decrypts"] = decrypt_ok(project_dir, env, backups)
        with open(os.path.join(backups, "stale.db"), "w") as fh:
            fh.write("plaintext-marker")
        with open(os.path.join(backups, ".app-stale.db.tmp"), "w") as fh:
            fh.write("plaintext-marker")
        audits_before = audit_all(project_dir, env)
        stale_before = sum(1 for entry in audits_before if "stale" in entry.lower())
        compose(project_dir, "restart", "persona-sanus", env=env, timeout=120)
        audits = wait_audit(project_dir, env, "stale", timeout=30)
        stale_after = sum(1 for entry in audits if "stale" in entry.lower())
        checks["stale_swept"] = plain_left(backups) == []
        stale_audit = next((entry for entry in audits if "stale" in entry.lower()), "")
        checks["stale_audits_actionable"] = "stale" in stale_audit.lower() and "rerun" in stale_audit.lower()
        checks["stale_audit"] = stale_audit[:200]
        checks["stale_first_run_failed"] = stale_after > stale_before
        second_ok = False
        started = time.monotonic()
        while time.monotonic() - started < 30:
            audits = audit_all(project_dir, env)
            stale_idx = next((i for i, entry in enumerate(audits) if "stale" in entry.lower()), -1)
            if stale_idx > 0 and any('"encrypted":true' in entry for entry in audits[:stale_idx]) and decrypt_ok(project_dir, env, backups):
                second_ok = True
                break
            time.sleep(2)
        checks["stale_second_run_succeeds"] = second_ok
        checks["final_no_plain"] = plain_left(backups) == []
        required = ["built", "deployed", "keygen", "recipient", "happy_age_created", "dir_mode_0700", "file_mode_0600", "happy_decrypts", "happy_no_plain", "atomic_rename_always_valid", "second_backup_decrypts", "age_hidden", "age_absent_no_plain", "age_absent_audits_error", "age_restored_decrypts", "stale_swept", "stale_audits_actionable", "stale_first_run_failed", "stale_second_run_succeeds", "final_no_plain"]
        ok = all(checks.get(k) for k in required)
        if not ok:
            return {"error": "backup safety checks failed", "checks": checks}
        return {"ok": True, "checks": checks}
    finally:
        try:
            compose(project_dir, "down", timeout=120)
        except Exception:
            pass
        if os.path.isdir(backups):
            shutil.rmtree(backups, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", default=".")
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    return verify(os.path.abspath(args.dir), args.port)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
