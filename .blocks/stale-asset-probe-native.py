#!/usr/bin/env python3
# /// block
# name = "stale-asset-probe-native"
# summary = "Native E2E stale pending asset cleanup: success, failure, recovery, idempotency."
# effects = "network"
# example = ["--dir", ".", "--port", "8080", "--minio-port", "9000"]
# matches = ['cleanup_stale_assets', 'asset_cleanup_failed', 'JOBS_INTERVAL_SECONDS']
#
# [returns]
# keys = ["ok", "checks"]
# doc = "checks maps phase names to booleans: success_row_gone, success_object_gone, success_summary_deleted, success_audit, failure_row_kept, failure_audit, recovery_deleted, recovery_object_gone, idempotent_rerun, skipped_row_kept, skipped_audit; containers and temp files removed at the end"
# [params]
# dir = { type = "path", default = ".", help = "repo root containing persona-sanus" }
# port = { type = "int", default = 8080, min = 1, help = "server port" }
# minio-port = { type = "int", default = 9000, min = 1, help = "minio port" }
# jobs-interval = { type = "int", default = 5, min = 1, help = "JOBS_INTERVAL_SECONDS for the scheduler under test" }
# email = { type = "str", default = "probe@example.com", help = "account created by asset-probe" }
# password = { type = "str", default = "ProbePass123!", help = "password for the probe account" }
# name = { type = "str", default = "Probe Tester", help = "display name for the probe account" }
#
# [provenance]
# created = "2026-11-20"
# source = "specs/divergences/13-M-stale-asset-cleanup.md verify steps"
#
# [stamp]
# verified = "000000000000"
# ///
"""Drive a native server binary plus a MinIO container through stale asset cleanup phases."""
import argparse
import hashlib
import hmac
import json
import os
import shutil
import signal
import sqlite3
import struct
import subprocess
import sys
import tempfile
import time
import urllib.request
import uuid


def run(cmd, cwd, env=None, timeout=300):
    """Run a command; return (exit code, stdout text)."""
    proc = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, timeout=timeout)
    return proc.returncode, proc.stdout.decode(errors="replace")


def http_status(url, timeout=5):
    """Return the HTTP status code for a GET, 0 on connection failure."""
    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            return response.status
    except Exception:
        return 0


def wait_health(base, timeout=120):
    """Poll the server health endpoint until it answers 200."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        if http_status(f"{base}/health", timeout=3) == 200:
            return True
        time.sleep(1)
    return False


def query_db(db_path, sql, params=()):
    """Run a read query against the stopped database copy."""
    connection = sqlite3.connect(db_path)
    try:
        return connection.execute(sql, params).fetchall()
    finally:
        connection.close()


def seed_stale_asset(db_path, asset_id, object_key):
    """Clone an existing asset row as a stale pending asset; return True on success."""
    connection = sqlite3.connect(db_path)
    try:
        row = connection.execute(
            "select clinic_id, owner_type, owner_id, kind, content_type, size_bytes,"
            " sha256, created_by from assets limit 1"
        ).fetchone()
        if row is None:
            return False
        clinic_id, owner_type, owner_id, kind, content_type, size, sha, created_by = row
        stale_at = (datetime.now(timezone.utc) - timedelta(hours=2)).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )
        connection.execute(
            "insert into assets (id, clinic_id, object_key, owner_type, owner_id, kind,"
            " original_name, content_type, size_bytes, sha256, width, height, status,"
            " deleted_at, created_at, updated_at, created_by)"
            " values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', null, ?, ?, ?)",
            (
                asset_id,
                clinic_id,
                object_key,
                owner_type,
                owner_id,
                kind,
                "stale-probe.png",
                content_type,
                size,
                sha,
                None,
                None,
                stale_at,
                stale_at,
                created_by,
            ),
        )
        connection.commit()
        return True
    finally:
        connection.close()


def sign(key, message):
    """One SigV4 HMAC step."""
    return hmac.new(key, message.encode("utf-8"), hashlib.sha256).digest()


def s3_request(method, host, port, bucket, key, access, secret, body=b""):
    """Signed SigV4 request to MinIO; return the HTTP status code."""
    now = datetime.now(timezone.utc)
    amz_date = now.strftime("%Y%m%dT%H%M%SZ")
    date = now.strftime("%Y%m%d")
    payload_hash = hashlib.sha256(body).hexdigest()
    canonical_uri = "/" + bucket + "/" + key
    canonical_request = "\n".join(
        [method, canonical_uri, "", "host:%s:%d\n" % (host, port), "host", payload_hash]
    )
    scope = f"{date}/us-east-1/s3/aws4_request"
    string_to_sign = "\n".join(
        [
            "AWS4-HMAC-SHA256",
            amz_date,
            scope,
            hashlib.sha256(canonical_request.encode("utf-8")).hexdigest(),
        ]
    )
    k_date = sign(("AWS4" + secret).encode("utf-8"), date)
    k_region = sign(k_date, "us-east-1")
    k_service = sign(k_region, "s3")
    k_signing = sign(k_service, "aws4_request")
    signature = hmac.new(
        k_signing, string_to_sign.encode("utf-8"), hashlib.sha256
    ).hexdigest()
    authorization = (
        f"AWS4-HMAC-SHA256 Credential={access}/{scope},"
        f" SignedHeaders=host, Signature={signature}"
    )
    request = urllib.request.Request(
        f"http://{host}:{port}{canonical_uri}",
        data=body if method == "PUT" else None,
        method=method,
    )
    request.add_header("Authorization", authorization)
    request.add_header("x-amz-date", amz_date)
    request.add_header("x-amz-content-sha256", payload_hash)
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            return response.status
    except Exception:
        return 0


class Server:
    """One native server process with a fixed environment."""

    def __init__(self, binary, cwd, base, db_path, storage=True):
        self.base = base
        self.db_path = db_path
        env = dict(os.environ)
        env.update(
            {
                "IP": "127.0.0.1",
                "PORT": base.rsplit(":", 1)[1],
                "DATABASE_URL": f"sqlite://{db_path}",
                "PERSONA_COOKIE_SECURE": "false",
                "JOBS_INTERVAL_SECONDS": "5",
            }
        )
        if storage:
            env.update(
                {
                    "ASSETS_BUCKET": "persona-assets",
                    "ASSETS_ENDPOINT": "http://127.0.0.1:9000",
                    "ASSETS_PUBLIC_ENDPOINT": "http://127.0.0.1:9000",
                    "ASSETS_REGION": "us-east-1",
                    "ASSETS_ACCESS_KEY_ID": "persona",
                    "ASSETS_SECRET_ACCESS_KEY": "persona-secret",
                }
            )
        else:
            for name in list(env):
                if name.startswith("ASSETS_"):
                    del env[name]
        self.proc = subprocess.Popen(
            [binary], cwd=cwd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )

    def start(self):
        """Wait until the server answers health."""
        return wait_health(self.base)

    def stop(self):
        """Terminate the server and wait for it to exit."""
        self.proc.send_signal(signal.SIGTERM)
        self.proc.wait(timeout=30)


def probe(root, port, minio_port, email, password, name):
    """Run every phase and return the checks map."""
    checks = {}
    base = f"http://127.0.0.1:{port}"
    binary = os.path.join(root, "persona-sanus", "target", "debug", "persona-sanus")
    tmp = tempfile.mkdtemp(prefix="stale-asset-")
    db_path = os.path.join(tmp, "app.db")
    first_id, second_id, third_id = (f"stale-{uuid.uuid4().hex[:8]}" for _ in range(3))
    minio_name = f"stale-minio-{uuid.uuid4().hex[:6]}"
    server = None

    def object_exists(key):
        return s3_request("HEAD", "127.0.0.1", minio_port, "persona-assets", key, "persona", "persona-secret") in (200, 204)

    def put_object(key):
        return s3_request("PUT", "127.0.0.1", minio_port, "persona-assets", key, "persona", "persona-secret", b"stale-asset-probe") in (200, 204)

    try:
        code, out = run(
            ["docker", "run", "-d", "--name", minio_name, "-p", f"{minio_port}:9000",
             "-e", "MINIO_ROOT_USER=persona", "-e", "MINIO_ROOT_PASSWORD=persona-secret",
             "persona-minio:RELEASE.2025-10-15T17-29-55Z", "server", "/data"],
            root,
            timeout=120,
        )
        if code != 0:
            return {"error": "minio container failed to start", "checks": checks}
        deadline = time.time() + 90
        while time.time() < deadline:
            if http_status(f"http://127.0.0.1:{minio_port}/minio/health/live", timeout=3) == 200:
                break
            time.sleep(2)
        server = Server(binary, os.path.join(root, "persona-sanus"), base, db_path)
        if not server.start():
            return {"error": "server did not become healthy", "checks": checks}
        code, _ = run(
            [sys.executable, os.path.join(root, ".blocks", "asset-probe.py"), "--base", base,
             "--email", email, "--password", password, "--name", name],
            root,
            timeout=600,
        )
        if code != 0:
            return {"error": "asset-probe failed", "checks": checks}
        server.stop()
        existing = query_db(db_path, "select clinic_id from assets limit 1")
        if not existing:
            return {"error": "no asset row to clone", "checks": checks}
        clinic_id = existing[0][0]
        keys = {asset: f"clinics/{clinic_id}/images/{asset}" for asset in (first_id, second_id, third_id)}
        if not seed_stale_asset(db_path, first_id, keys[first_id]) or not put_object(keys[first_id]):
            return {"error": "success-phase seed failed", "checks": checks}
        server = Server(binary, os.path.join(root, "persona-sanus"), base, db_path)
        if not server.start():
            return {"error": "server did not restart", "checks": checks}
        time.sleep(15)
        server.stop()
        gone = query_db(db_path, "select count(*) from assets where id = ?", (first_id,))
        checks["success_row_gone"] = gone[0][0] == 0
        checks["success_object_gone"] = not object_exists(keys[first_id])
        summaries = query_db(db_path, "select metadata_json from audit_log where action = 'asset_cleanup' and entity_id = 'stale_pending'")
        checks["success_summary_deleted"] = any(
            (json.loads(row[0]).get("deleted", 0) >= 1) for row in summaries if row[0]
        )
        checks["success_audit"] = query_db(db_path, "select count(*) from audit_log where action = 'asset_cleanup' and entity_id = ?", (first_id,))[0][0] >= 1
        if not all(checks.values()):
            return {"error": "success phase failed", "checks": checks}
        if not seed_stale_asset(db_path, second_id, keys[second_id]) or not put_object(keys[second_id]):
            return {"error": "failure-phase seed failed", "checks": checks}
        run(["docker", "stop", minio_name], root, timeout=60)
        server = Server(binary, os.path.join(root, "persona-sanus"), base, db_path)
        if not server.start():
            return {"error": "server did not restart for failure phase", "checks": checks}
        time.sleep(15)
        server.stop()
        kept = query_db(db_path, "select status from assets where id = ?", (second_id,))
        checks["failure_row_kept"] = bool(kept) and kept[0][0] == "pending"
        checks["failure_audit"] = query_db(db_path, "select count(*) from audit_log where action = 'asset_cleanup_failed' and entity_id = ?", (second_id,))[0][0] >= 1
        if not all(checks.values()):
            return {"error": "failure phase failed", "checks": checks}
        run(["docker", "start", minio_name], root, timeout=60)
        time.sleep(5)
        server = Server(binary, os.path.join(root, "persona-sanus"), base, db_path)
        if not server.start():
            return {"error": "server did not restart for recovery", "checks": checks}
        time.sleep(15)
        server.stop()
        checks["recovery_deleted"] = query_db(db_path, "select count(*) from assets where id = ?", (second_id,))[0][0] == 0
        checks["recovery_object_gone"] = not object_exists(keys[second_id])
        if not all(checks.values()):
            return {"error": "recovery phase failed", "checks": checks}
        before = query_db(db_path, "select count(*) from assets")[0][0]
        before_audit = query_db(db_path, "select count(*) from audit_log")[0][0]
        server = Server(binary, os.path.join(root, "persona-sanus"), base, db_path)
        if not server.start():
            return {"error": "server did not restart for idempotency phase", "checks": checks}
        time.sleep(15)
        server.stop()
        latest = query_db(db_path, "select metadata_json from audit_log where action = 'asset_cleanup' and entity_id = 'stale_pending' order by at desc limit 1")
        idle = json.loads(latest[0][0]) if latest and latest[0][0] else {}
        checks["idempotent_rerun"] = (
            query_db(db_path, "select count(*) from assets")[0][0] == before
            and query_db(db_path, "select count(*) from audit_log")[0][0] > before_audit
            and idle.get("deleted", 1) == 0
            and idle.get("failed", 1) == 0
            and idle.get("skipped", 1) == 0
        )
        if not checks["idempotent_rerun"]:
            return {"error": "idempotency phase failed", "checks": checks}
        if not seed_stale_asset(db_path, third_id, keys[third_id]) or not put_object(keys[third_id]):
            return {"error": "skipped-phase seed failed", "checks": checks}
        server = Server(binary, os.path.join(root, "persona-sanus"), base, db_path, storage=False)
        if not server.start():
            return {"error": "server did not restart without storage", "checks": checks}
        time.sleep(15)
        server.stop()
        kept = query_db(db_path, "select status from assets where id = ?", (third_id,))
        checks["skipped_row_kept"] = bool(kept) and kept[0][0] == "pending"
        skipped = query_db(db_path, "select metadata_json from audit_log where action = 'asset_cleanup_skipped' and entity_id = ?", (third_id,))
        checks["skipped_audit"] = bool(skipped) and skipped[0][0] and json.loads(skipped[0][0]).get("reason") == "storage_not_configured"
        return {"checks": checks}
    finally:
        if server is not None and server.proc.poll() is None:
            server.stop()
        run(["docker", "rm", "-f", minio_name], root, timeout=60)
        shutil.rmtree(tmp, ignore_errors=True)

def main():
    """Parse arguments and run the probe; print one JSON verdict."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", default=".")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--minio-port", type=int, default=9000)
    parser.add_argument("--jobs-interval", type=int, default=5)
    parser.add_argument("--email", default="probe@example.com")
    parser.add_argument("--password", default="ProbePass123!")
    parser.add_argument("--name", default="Probe Tester")
    args = parser.parse_args()
    os.makedirs(os.path.join(os.path.abspath(args.dir), "persona-sanus", "target", "debug", "public"), exist_ok=True)
    result = probe(os.path.abspath(args.dir), args.port, args.minio_port, args.email, args.password, args.name)
    result["ok"] = all(result.get("checks", {}).values()) and "error" not in result
    print(json.dumps(result))
    return result


if __name__ == "__main__":
    verdict = main()
    sys.exit(0 if verdict.get("ok") else 1)
