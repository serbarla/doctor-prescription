#!/usr/bin/env python3
# /// block
# name = "persona-docker-verify"
# summary = "Verify compose stack: build, up, probe, recreate + re-sign-in for volume persistence, down."
# effects = "external"
# example = ["--dir", ".", "--port", "8080"]
# matches = ['docker compose (build|up|down)']
#
# [returns]
# keys = ["ok", "checks", "port"]
# doc = "checks merges compose stages (deployed, health, recreated) with persona-probe checks; on failure exits non-zero with error plus partial checks"
#
# [params]
# dir = { type = "path", default = ".", help = "compose project directory (repo root), must hold docker-compose.yml" }
# port = { type = "int", default = 8080, min = 1, max = 65535, help = "host port the stack publishes" }
# email = { type = "str", default = "verify@example.com", help = "account email for the probe flow" }
# password = { type = "str", default = "VerifyPass123!", help = "account password for the probe flow" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: session verification flow"
#
# [stamp]
# verified = "5912d0424fef"
# ///
"""Verify the persona-sanus compose stack end to end and print one JSON verdict."""
import argparse
import json
import os
import subprocess
import sys

ORDERED = [
    "deployed", "health", "home", "login", "dashboard", "signup", "session",
    "signout", "session_after_signout", "signin", "recreated", "persistent_signin",
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


def verify(project_dir, port, email, password):
    """Bring the stack up, probe it, recreate it, sign in again, tear it down."""
    checks = {}
    base = f"http://127.0.0.1:{port}"
    project_dir = os.path.abspath(project_dir)

    try:
        checks["built"] = run_cmd(["docker", "compose", "build"], 1800, cwd=project_dir)
        checks["deployed"] = run_cmd(["docker", "compose", "up", "-d"], 300, cwd=project_dir)
        if not (checks["built"] and checks["deployed"]):
            return {"error": "compose build or up failed", "checks": checks, "port": port}

        _, health = run_block(
            "wait-health",
            ["--url", base + "/health", "--timeout", "120", "--interval", "2"],
            180,
        )
        checks["health"] = bool(health.get("ok"))
        if not checks["health"]:
            return {"error": "container never healthy", "checks": checks, "port": port}

        ok, probe = run_block(
            "persona-probe",
            ["--base", base, "--email", email, "--password", password],
            120,
        )
        checks.update(probe.get("checks", {}))
        if not ok:
            return {
                "error": "persona-probe failed: " + probe.get("error", "unknown"),
                "checks": checks, "port": port,
            }

        checks["recreated"] = run_cmd(
            ["docker", "compose", "up", "-d", "--force-recreate"], 300, cwd=project_dir
        )
        if not checks["recreated"]:
            return {"error": "recreate failed", "checks": checks, "port": port}
        _, health = run_block(
            "wait-health",
            ["--url", base + "/health", "--timeout", "120", "--interval", "2"],
            180,
        )
        checks["health_after_recreate"] = bool(health.get("ok"))
        if not checks["health_after_recreate"]:
            return {"error": "container never healthy after recreate", "checks": checks, "port": port}
        ok, probe = run_block(
            "persona-probe",
            ["--base", base, "--email", email, "--password", password, "--flow", "signin"],
            120,
        )
        checks["persistent_signin"] = bool(probe.get("checks", {}).get("signin"))
        checks.update({f"signin_{k}": v for k, v in probe.get("checks", {}).items()})
        if not ok:
            return {
                "error": "post-recreate signin failed: " + probe.get("error", "unknown"),
                "checks": checks, "port": port,
            }
    finally:
        run_cmd(["docker", "compose", "down"], 120, cwd=project_dir)

    wanted = ORDERED + ["health_after_recreate"]
    ok = all(checks.get(name) for name in wanted)
    verdict = {
        "ok": ok,
        "checks": {name: bool(checks.get(name)) for name in wanted},
        "port": port,
    }
    if not ok:
        verdict["error"] = "one or more checks failed"
    return verdict


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", default=".")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--email", default="verify@example.com")
    parser.add_argument("--password", default="VerifyPass123!")
    args = parser.parse_args()
    return verify(args.dir, args.port, args.email, args.password)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
