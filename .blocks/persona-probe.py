#!/usr/bin/env python3
# /// block
# name = "persona-probe"
# summary = "Probe a running persona-sanus server: pages plus signup/session/signout/signin over the auth API."
# effects = "network"
# example = ["--base", "http://127.0.0.1:8080", "--email", "probe@example.com", "--password", "ProbePass123!"]
# matches = ['/api/auth/sign-up/email', '/api/auth/(sign-in|sign-out|get-session)']
#
# [returns]
# keys = ["ok", "checks", "base"]
# doc = "checks maps probe names (home, login, dashboard, signup, session, signout, session_after_signout, signin) to booleans; flow=signin checks only signin plus session"
#
# [params]
# base = { type = "str", required = true, help = "server origin, e.g. http://127.0.0.1:8080" }
# email = { type = "str", default = "probe@example.com", help = "account email for the probe flow" }
# password = { type = "str", default = "ProbePass123!", help = "account password for the probe flow" }
# name = { type = "str", default = "Probe Tester", help = "display name used by signup (full flow only)" }
# flow = { type = "enum", values = ["full", "signin"], default = "full", help = "full runs signup then signout then signin; signin only signs in (needs an existing account)" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: session verification flow"
# ///
"""Probe one running persona-sanus server and print a small JSON verdict."""
import argparse
import http.cookiejar
import json
import sys
import urllib.request
import urllib.error

FULL = ["home", "login", "dashboard", "signup", "session", "signout", "session_after_signout", "signin"]
SIGNIN = ["signin", "session"]


def http_get(base, path, jar):
    """Return (status, first 4000 bytes of body) for a GET."""
    req = urllib.request.Request(base + path)
    try:
        with urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(jar)
        ).open(req, timeout=10) as res:
            return res.status, res.read(4000)
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read(4000)
    except OSError:
        return None, b""


def http_post(base, path, jar, payload):
    """POST JSON with a same-origin Origin header; return (status, parsed body)."""
    body = json.dumps(payload).encode()
    req = urllib.request.Request(
        base + path,
        data=body,
        headers={"Content-Type": "application/json", "Origin": base},
        method="POST",
    )
    try:
        with urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(jar)
        ).open(req, timeout=10) as res:
            return res.status, json.loads(res.read() or b"{}")
    except urllib.error.HTTPError as exc:
        try:
            return exc.code, json.loads(exc.read() or b"{}")
        except ValueError:
            return exc.code, {}
    except OSError:
        return None, {}


def get_json(base, path, jar):
    """GET a JSON endpoint; return (status, parsed body or {})."""
    status, raw = http_get(base, path, jar)
    try:
        return status, json.loads(raw or b"null")
    except ValueError:
        return status, {}


def probe(base, email, password, name, flow):
    """Run the page + auth probes against base; return the verdict dict."""
    checks = {}
    jar = http.cookiejar.CookieJar()

    def signin():
        status, data = http_post(
            base, "/api/auth/sign-in/email", jar,
            {"email": email, "password": password, "rememberMe": True},
        )
        return status == 200 and bool(data.get("token"))

    if flow == "signin":
        checks["signin"] = signin()
        status, data = get_json(base, "/api/auth/get-session", jar)
        checks["session"] = status == 200 and bool(data.get("session"))
        ordered = SIGNIN
    else:
        status, body = http_get(base, "/", jar)
        checks["home"] = status == 200 and b"PersonaSanus" in body
        status, _ = http_get(base, "/login", jar)
        checks["login"] = status == 200
        status, _ = http_get(base, "/dashboard", jar)
        checks["dashboard"] = status == 200
        status, data = http_post(
            base, "/api/auth/sign-up/email", jar,
            {"name": name, "email": email, "password": password, "rememberMe": True},
        )
        checks["signup"] = status == 200 and bool(data.get("token"))
        status, data = get_json(base, "/api/auth/get-session", jar)
        checks["session"] = status == 200 and bool(data.get("session"))
        status, data = http_post(base, "/api/auth/sign-out", jar, {})
        checks["signout"] = status == 200 and data.get("success") is True
        status, data = get_json(base, "/api/auth/get-session", jar)
        checks["session_after_signout"] = status == 200 and data in (None, {})
        checks["signin"] = signin()
        ordered = FULL

    ok = all(checks.get(item) for item in ordered)
    verdict = {"ok": ok, "checks": {item: bool(checks.get(item)) for item in ordered}, "base": base}
    if not ok:
        verdict["error"] = "one or more checks failed"
    return verdict


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--email", default="probe@example.com")
    parser.add_argument("--password", default="ProbePass123!")
    parser.add_argument("--name", default="Probe Tester")
    parser.add_argument("--flow", default="full")
    args = parser.parse_args()
    return probe(args.base, args.email, args.password, args.name, args.flow)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
