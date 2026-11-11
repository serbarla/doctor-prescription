#!/usr/bin/env python3
# /// block
# name = "wait-health"
# summary = "Poll a URL until it answers with the expected HTTP status, or time out."
# effects = "network"
# example = ["--url", "http://127.0.0.1:3000/", "--expect", "404", "--timeout", "15"]
# matches = ['curl[^\n]*health', 'for .*seq.*; *curl', 'health.*curl.*--max-time']
#
# [returns]
# keys = ["ok", "url", "status", "attempts", "elapsed_s"]
# doc = "ok true when status equals expect; on timeout exits non-zero with error plus last_status"
#
# [params]
# url = { type = "str", required = true, help = "URL to poll, e.g. a local dev server health endpoint" }
# expect = { type = "int", default = 200, help = "HTTP status that counts as up" }
# timeout = { type = "int", default = 60, min = 1, help = "seconds to keep polling" }
# interval = { type = "float", default = 2, min = 0.1, help = "seconds between attempts" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: session verification polling"
# ///
"""Poll one URL until it returns the expected HTTP status; print a small JSON verdict."""
import argparse
import json
import sys
import time
import urllib.request
import urllib.error


def probe(url):
    """Return the HTTP status code for one GET, or None when unreachable."""
    try:
        with urllib.request.urlopen(url, timeout=5) as response:
            return response.status
    except urllib.error.HTTPError as exc:
        return exc.code
    except OSError:
        return None


def wait_health(url, expect, timeout, interval):
    """Poll url until its status equals expect or timeout elapses."""
    started = time.monotonic()
    attempts = 0
    last = None
    while True:
        attempts += 1
        last = probe(url)
        if last == expect:
            return {
                "ok": True,
                "url": url,
                "status": last,
                "attempts": attempts,
                "elapsed_s": round(time.monotonic() - started, 2),
            }
        if time.monotonic() - started >= timeout:
            return {
                "error": f"no status {expect} from {url} within {timeout}s",
                "url": url,
                "attempts": attempts,
                "last_status": last,
            }
        time.sleep(interval)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", required=True)
    parser.add_argument("--expect", type=int, default=200)
    parser.add_argument("--timeout", type=int, default=60)
    parser.add_argument("--interval", type=float, default=2)
    args = parser.parse_args()
    return wait_health(args.url, args.expect, args.timeout, args.interval)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
