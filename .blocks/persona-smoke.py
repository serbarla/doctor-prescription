#!/usr/bin/env python3
# /// block
# name = "persona-smoke"
# summary = "E2E smoke: fmt/check/clippy + release build, then full auth probe on a throwaway server."
# effects = "network"
# example = ["--dir", "persona-sanus", "--port", "8091"]
# matches = ['dx build --release --fullstack', 'cargo (check|clippy|fmt)']
#
# [returns]
# doc = "checks merges static/build stages (fmt, check, clippy, build, health) with persona-probe checks; on failure exits non-zero with error plus partial checks. Spawning the throwaway server needs a machine where a child process can bind a local port."
# keys = ["ok", "checks", "port"]
#
# [params]
# dir = { type = "path", default = "persona-sanus", help = "app directory holding Cargo.toml and the release server binary" }
# port = { type = "int", default = 8091, min = 1, max = 65535, help = "port the throwaway server listens on" }
# secret = { type = "str", help = "session secret (>=32 chars); random one generated when omitted" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: session verification flow"
# ///
"""Run the full local smoke of persona-sanus and print one JSON verdict (see checks)."""
import argparse
import json
import os
import secrets
import signal
import subprocess
import sys
import tempfile

GRACE = 5
ORDERED = [
    "fmt", "check", "clippy", "build", "health",
    "home", "login", "dashboard", "signup", "session", "signout",
    "session_after_signout", "signin",
]


def run_cmd(cmd, timeout, cwd=None):
    """Return True when the command exits 0 within timeout."""
    try:
        proc = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout
        )
        return proc.returncode == 0
    except (OSError, subprocess.TimeoutExpired):
        return False


def run_block(name, params, timeout):
    """Run one block through caveman-blocks; return (ok, parsed JSON or {})."""
    cmd = ["caveman-blocks", "run", name] + params
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except OSError:
        return False, {}
    try:
        return proc.returncode == 0, json.loads(proc.stdout.strip().splitlines()[-1])
    except ValueError:
        return proc.returncode == 0, {}


def smoke(app_dir, port, secret):
    """Run every smoke stage against a throwaway server; return the verdict dict."""
    checks = {}
    base = f"http://127.0.0.1:{port}"
    app_dir = os.path.abspath(app_dir)

    checks["fmt"] = run_cmd(["cargo", "fmt", "--check"], 120, cwd=app_dir)
    checks["check"] = run_cmd(["cargo", "check"], 600, cwd=app_dir)
    checks["clippy"] = run_cmd(["cargo", "clippy", "--", "-D", "warnings"], 600, cwd=app_dir)
    checks["build"] = run_cmd(["dx", "build", "--release", "--fullstack"], 1200, cwd=app_dir)
    if not all(checks.values()):
        return {"error": "static or build stage failed", "checks": checks, "port": port}

    binary = os.path.join(app_dir, "target", "dx", "persona-sanus", "release", "web", "server")
    if not os.path.isfile(binary):
        return {"error": f"server binary missing at {binary}", "checks": checks, "port": port}

    with tempfile.TemporaryDirectory(prefix="persona-smoke-") as tmpdir:
        env = dict(
            os.environ,
            DATABASE_URL=f"sqlite://{tmpdir}/smoke.db",
            RUSTAUTH_SECRET=secret,
            IP="127.0.0.1",
            PORT=str(port),
        )
        server = subprocess.Popen(
            [binary], cwd=app_dir, env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        try:
            _, health = run_block(
                "wait-health",
                ["--url", base + "/health", "--timeout", "60", "--interval", "1"],
                90,
            )
            checks["health"] = bool(health.get("ok"))
            if not checks["health"]:
                return {"error": "server never healthy", "checks": checks, "port": port}
            ok, probe = run_block(
                "persona-probe",
                ["--base", base, "--email", "smoke@example.com", "--password", "SmokePass123!"],
                120,
            )
            checks.update(probe.get("checks", {}))
            if not ok:
                return {
                    "error": "persona-probe failed: " + probe.get("error", "unknown"),
                    "checks": checks, "port": port,
                }
        finally:
            server.send_signal(signal.SIGTERM)
            try:
                server.wait(timeout=GRACE)
            except subprocess.TimeoutExpired:
                server.kill()

    ok = all(checks.get(name) for name in ORDERED)
    verdict = {"ok": ok, "checks": {name: bool(checks.get(name)) for name in ORDERED}, "port": port}
    if not ok:
        verdict["error"] = "one or more checks failed"
    return verdict


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", default="persona-sanus")
    parser.add_argument("--port", type=int, default=8091)
    parser.add_argument("--secret", default="")
    args = parser.parse_args()
    secret = args.secret or secrets.token_urlsafe(48)
    return smoke(args.dir, args.port, secret)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
