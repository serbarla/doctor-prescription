#!/usr/bin/env python3
# /// block
# name = "asset-probe"
# summary = "Probe image library: upload, quota, formats, size cap, metadata, signed URL TTL, pin protection."
# effects = "network"
# example = ["--base", "http://127.0.0.1:8080"]
# matches = ['/api/assets', '/api/clinic/logo', '/api/clinic']
#
# [returns]
# keys = ["ok", "checks", "base"]
# doc = "checks maps probe names to booleans; verifies upload metadata (width/height/sha256), the v1 one-image quota, the 10 MiB cap, JPEG/PNG-only decoding, the 900-second signed GET URL (and that tampering with it fails), private bucket access, logo upload excluded from quota, template publish pins, and refusal to delete a pinned image"
#
# [params]
# base = { type = "str", required = true, help = "server origin, e.g. http://127.0.0.1:8080" }
# email = { type = "str", default = "probe@example.com", help = "administrator account email" }
# password = { type = "str", default = "ProbePass123!", help = "administrator account password" }
# name = { type = "str", default = "Probe Tester", help = "display name used when the account is created" }
#
# [provenance]
# created = "2026-11-12"
# source = "promoted: spec 06-F verification flow"
#
# [stamp]
# verified = "ebcc7d4b9b4e"
# ///
"""Probe the persona-sanus image library and print one JSON verdict."""
import argparse
import base64
import http.cookiejar
import json
import struct
import sys
import urllib.error
import urllib.request
import uuid

JPEG = base64.b64decode(
    "/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0a"
    "HBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/wAALCAABAAEBAREA/8QAFAABAAAAAAAA"
    "AAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q=="
)
PNG = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
)
CHECKS = [
    "signin", "admin_me", "library_list", "upload_image", "metadata",
    "quota_exceeded", "oversized_rejected", "webp_rejected", "svg_rejected",
    "signed_url_ttl", "signed_url_serves", "signed_url_tampered", "private_bucket",
    "logo_upload", "logo_excluded_from_quota", "template_publish_pins",
    "pinned_delete_refused",
]


def opener(jar):
    """Return a urllib opener that carries the session cookie jar."""
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))


def http_call(base, method, path, jar, body=None, headers=None):
    """Send one request; return (status, parsed JSON, headers dict)."""
    all_headers = {"Origin": base}
    if headers:
        all_headers.update(headers)
    req = urllib.request.Request(base + path, data=body, headers=all_headers, method=method)
    try:
        with opener(jar).open(req, timeout=60) as res:
            raw = res.read()
            try:
                parsed = json.loads(raw or b"{}")
            except ValueError:
                parsed = {}
            return res.status, parsed, dict(res.headers)
    except urllib.error.HTTPError as exc:
        raw = exc.read()
        try:
            parsed = json.loads(raw or b"{}")
        except ValueError:
            parsed = {}
        return exc.code, parsed, dict(exc.headers)
    except OSError:
        return None, {}, {}


def http_json(base, method, path, jar, payload=None):
    """Send JSON; return (status, parsed JSON, headers)."""
    body = json.dumps(payload).encode() if payload is not None else None
    headers = {"Content-Type": "application/json"} if body is not None else None
    return http_call(base, method, path, jar, body, headers)


def multipart(field, filename, content_type, data):
    """Build a one-file multipart body; return (body, content-type header)."""
    boundary = "----persona" + uuid.uuid4().hex
    body = b"--" + boundary.encode() + b"\r\n"
    body += (
        f'Content-Disposition: form-data; name="{field}"; filename="{filename}"\r\n'
    ).encode()
    body += f"Content-Type: {content_type}\r\n\r\n".encode()
    body += data + b"\r\n--" + boundary.encode() + b"--\r\n"
    return body, "multipart/form-data; boundary=" + boundary


def upload(base, path, jar, data, filename="probe.jpg", content_type="image/jpeg"):
    """Upload one file; return (status, parsed JSON)."""
    body, content_type_header = multipart("file", filename, content_type, data)
    status, parsed, _ = http_call(
        base, "POST", path, jar, body, {"Content-Type": content_type_header}
    )
    return status, parsed


def sign_in(base, email, password, name, jar):
    """Sign in, creating the administrator account when it does not exist yet."""
    status, _, _ = http_json(
        base, "POST", "/api/auth/sign-in/email", jar,
        {"email": email, "password": password},
    )
    if status == 200:
        return True
    status, _, _ = http_json(
        base, "POST", "/api/auth/sign-up/email", jar,
        {"name": name, "email": email, "password": password},
    )
    return status == 200


def get_asset_url(base, jar, asset_id):
    """Return the signed URL payload for one asset."""
    status, parsed, _ = http_json(base, "GET", f"/api/assets/{asset_id}/url", jar)
    if status != 200:
        return None
    return parsed


def document_json(image_id, logo_id):
    """Build a valid structured template document pinning both assets."""
    return json.dumps(
        {
            "schema_version": 1,
            "letterhead": {
                "logo_asset_id": logo_id,
                "doctor_details": "Dr. Probe\nMedicina general",
                "clinic_details": "Clínica Probe\nQuito",
                "colors": {"primary": "#0f6e64", "accent": "#e8b04b"},
            },
            "body": "Paciente: {{patient.name}}",
            "footer": "Firma del médico",
            "images": [{"asset_id": image_id, "caption": "Imagen de biblioteca"}],
            "medications": [],
        }
    )


def fetch_asset(url):
    """GET a presigned URL without cookies; return (status, content-type)."""
    req = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(req, timeout=30) as res:
            res.read(64)
            return res.status, res.headers.get("content-type", "")
    except urllib.error.HTTPError as exc:
        return exc.code, exc.headers.get("content-type", "")
    except OSError:
        return None, ""


def probe(base, email, password, name):
    """Run the image-library probe and return the checks map."""
    checks = {}
    jar = http.cookiejar.CookieJar()
    checks["signin"] = sign_in(base, email, password, name, jar)

    status, me, _ = http_json(base, "GET", "/api/me", jar)
    checks["admin_me"] = status == 200 and me.get("role") == "administrator"
    if not checks["signin"] or not checks["admin_me"]:
        return checks

    status, library, _ = http_json(base, "GET", "/api/assets", jar)
    checks["library_list"] = status == 200 and isinstance(library.get("assets"), list)
    if not checks["library_list"]:
        return checks

    images = [asset for asset in library["assets"] if asset.get("kind") == "image"]
    image = images[0] if images else None
    if image is None:
        status, created = upload(base, "/api/assets", jar, JPEG)
        checks["upload_image"] = status == 201
        if status != 201:
            return checks
        image = created
    else:
        checks["upload_image"] = image.get("status") == "ready"
    checks["metadata"] = (
        (image.get("width") or 0) > 0
        and (image.get("height") or 0) > 0
        and len(image.get("sha256") or "") == 64
    )
    image_id = image.get("id")

    status, rejected = upload(base, "/api/assets", jar, JPEG)
    checks["quota_exceeded"] = (
        status == 409 and rejected.get("code") == "image_quota_exceeded"
    )

    oversized = b"\xff\xd8\xff" + b"\x00" * 10485760
    status, rejected = upload(base, "/api/assets", jar, oversized)
    checks["oversized_rejected"] = (
        400 <= (status or 0) < 500 and rejected.get("code") == "image_too_large"
    )

    webp = b"RIFF" + struct.pack("<I", 4) + b"WEBP" + b"VP8 " + b"\x00" * 16
    status, rejected = upload(base, "/api/assets", jar, webp, "probe.webp", "image/webp")
    checks["webp_rejected"] = (
        400 <= (status or 0) < 500 and rejected.get("code") == "unsupported_media_type"
    )

    svg = b'<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>'
    status, rejected = upload(base, "/api/assets", jar, svg, "probe.svg", "image/svg+xml")
    checks["svg_rejected"] = (
        400 <= (status or 0) < 500 and rejected.get("code") == "unsupported_media_type"
    )

    signed = get_asset_url(base, jar, image_id) or {}
    url = signed.get("url") or ""
    checks["signed_url_ttl"] = "X-Amz-Expires=900" in url and signed.get("expiresIn") == 900
    status, content_type = fetch_asset(url) if url else (None, "")
    checks["signed_url_serves"] = status == 200 and content_type.startswith("image/")
    tampered = url.replace("X-Amz-Expires=900", "X-Amz-Expires=1")
    status, _ = fetch_asset(tampered) if tampered and tampered != url else (None, "")
    checks["signed_url_tampered"] = status in (400, 403)
    status, _ = fetch_asset(url.split("?", 1)[0]) if url else (None, "")
    checks["private_bucket"] = status == 403

    status, logo = upload(base, "/api/clinic/logo", jar, PNG, "logo.png", "image/png")
    checks["logo_upload"] = status == 201 and logo.get("kind") == "logo"
    logo_id = logo.get("id") if status == 201 else None
    status, library, _ = http_json(base, "GET", "/api/assets", jar)
    quota = library.get("quota") or {}
    logos = [asset for asset in library.get("assets", []) if asset.get("kind") == "logo"]
    if logo_id is None and logos:
        logo_id = logos[0].get("id")
    checks["logo_excluded_from_quota"] = (
        status == 200 and quota.get("used") == 1 and quota.get("limit") == 1
        and len(logos) == 1
    )

    name_suffix = uuid.uuid4().hex[:8]
    status, template, _ = http_json(
        base, "POST", "/api/templates", jar, {"name": f"Probe {name_suffix}"}
    )
    template_id = template.get("id") if status == 200 else None
    version_id = None
    if template_id:
        status, version, _ = http_json(
            base, "POST", f"/api/templates/{template_id}/versions", jar,
            {"docJson": document_json(image_id, logo_id), "theme": "{\"margin_mm\":15}"},
        )
        if status == 200:
            version_id = version.get("id")
    if template_id and version_id:
        status, _, _ = http_json(
            base, "POST", f"/api/templates/{template_id}/publish", jar,
            {"versionId": version_id},
        )
        checks["template_publish_pins"] = status == 200
        status, refused, _ = http_json(base, "DELETE", f"/api/assets/{image_id}", jar)
        checks["pinned_delete_refused"] = (
            status == 409 and refused.get("code") == "asset_in_use"
        )
    else:
        checks["template_publish_pins"] = False
        checks["pinned_delete_refused"] = False
    return checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--email", default="probe@example.com")
    parser.add_argument("--password", default="ProbePass123!")
    parser.add_argument("--name", default="Probe Tester")
    args = parser.parse_args()
    checks = probe(args.base, args.email, args.password, args.name)
    verdict = {
        "ok": all(checks.get(name) for name in CHECKS),
        "checks": {name: bool(checks.get(name)) for name in CHECKS},
        "base": args.base,
    }
    if not verdict["ok"]:
        verdict["error"] = "one or more checks failed"
    return verdict


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
