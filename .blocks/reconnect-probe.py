#!/usr/bin/env python3
# /// block
# name = "reconnect-probe"
# summary = "Desktop GUI reconnect: oldest-first outbox drain and final PDF QR."
# effects = "network"
# example = ["--port", "8123"]
# matches = ['/api/prescriptions/:id/pdf', '/api/rx/blocks', '/api/devices/register']
#
# [returns]
# keys = ["ok", "checks", "prescription_id", "rx_number", "code", "qr_payload", "gui_screenshot"]
# doc = "checks: provision (admin bootstrap, doctor assigned, device registered, block allocated); seed (session meta + 6 pending outbox rows in order); gui_window (screenshot captured while the GUI process runs); drain_synced (every outbox row synced); oldest_first (server audit signs queued number before the later one); reconciled_a/reconciled_b (detail status signed with queued numbers and verification codes); pdf_* (final PDF 200, A4, QR decodes to {base}/verify/{code}, filename present)."
#
# [params]
# port = { type = "int", default = 8123, min = 1, max = 65535, help = "port the throwaway server listens on" }
# password = { type = "str", default = "ProbePass123!", help = "password shared by the probe accounts" }
# binary = { type = "path", default = "persona-sanus/target/debug/persona-sanus", help = "desktop binary built with --no-default-features --features desktop" }
# server-binary = { type = "path", default = "persona-sanus/target/dx/persona-sanus/release/web/server", help = "release server binary built by dx build --release --fullstack" }
# data-dir = { type = "path", default = "", help = "desktop local data dir; a throwaway tempdir is used when omitted" }
# timeout = { type = "int", default = 90, min = 5, max = 600, help = "seconds to wait for the GUI drain" }
#
# [provenance]
# created = "2026-11-13"
# source = "spec 11-K divergence 1"
# ///
"""Prove the desktop GUI reconnect run: outbox drains oldest-first and the final PDF QR resolves."""
import http.cookiejar
import importlib.util
import json
import os
import re
import secrets
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.request
import uuid

GRACE = 5
A4 = (595.28, 841.89)


def load_block(name):
    """Load a sibling block module by file name and return it."""
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), name)
    spec = importlib.util.spec_from_file_location(name[:-3].replace("-", "_"), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def rfc3339(moment):
    """Format a unix timestamp as RFC3339 UTC with Z suffix."""
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(moment))


def session_token(jar):
    """Return the persona_session cookie value or empty string."""
    for cookie in jar:
        if cookie.name == "persona_session" and cookie.value:
            return cookie.value
    return ""


def wait_health(base, timeout):
    """Poll the health endpoint until it answers 200."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(base + "/health", timeout=2) as response:
                if response.status == 200:
                    return True
        except Exception:
            time.sleep(0.5)
    return False


def launch_gui(binary, base, data_dir, cwd):
    """Launch the desktop binary detached with the probe origin and data dir."""
    env = dict(
        os.environ,
        PERSONA_SERVER_ORIGIN=base,
        PERSONA_LOCAL_DATA_DIR=data_dir,
    )
    proc = subprocess.Popen(
        [binary],
        cwd=cwd,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    return proc


def stop_gui(proc):
    """SIGTERM the GUI process group without raising."""
    try:
        os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
    except (ProcessLookupError, PermissionError):
        return
    try:
        proc.wait(timeout=GRACE)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass


def outbox_statuses(db_path):
    """Return outbox rows as (operation, status) ordered by created_at."""
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute(
            "SELECT operation, status, updated_at FROM outbox ORDER BY created_at"
        ).fetchall()
    finally:
        conn.close()
    return rows


def wait_drained(db_path, timeout):
    """Poll until every outbox row is synced; return statuses list."""
    deadline = time.time() + timeout
    rows = []
    while time.time() < deadline:
        try:
            rows = outbox_statuses(db_path)
        except sqlite3.OperationalError:
            rows = []
        if rows and all(status == "synced" for _, status, _ in rows):
            return rows
        if rows and any(status == "failed" for _, status, _ in rows):
            return rows
        time.sleep(1)
    return rows


def seed_local_db(db_path, meta, rows):
    """Insert session meta and ordered pending outbox rows."""
    conn = sqlite3.connect(db_path)
    try:
        for key, value in meta.items():
            conn.execute(
                "INSERT OR REPLACE INTO sync_meta (key, value, updated_at) VALUES (?, ?, ?)",
                (key, value, rfc3339(time.time())),
            )
        base = time.time()
        for index, (operation, entity_type, entity_id, payload) in enumerate(rows):
            at = rfc3339(base + index)
            conn.execute(
                "INSERT INTO outbox (id, operation, entity_type, entity_id, payload_json,"
                " status, attempts, next_retry_at, last_error, created_at, updated_at)"
                " VALUES (?, ?, ?, ?, ?, 'pending', 0, NULL, NULL, ?, ?)",
                (
                    str(uuid.uuid4()),
                    operation,
                    entity_type,
                    entity_id,
                    json.dumps(payload),
                    at,
                    at,
                ),
            )
        conn.commit()
    finally:
        conn.close()


def audit_sign_order(db_path, first_id, second_id):
    """Return True when audit sign rows arrive in the queued order."""
    conn = sqlite3.connect(db_path)
    try:
        rows = conn.execute(
            "SELECT entity_id FROM audit_log WHERE action = 'sign_prescription'"
            " AND entity_id IN (?, ?) ORDER BY rowid",
            (first_id, second_id),
        ).fetchall()
    finally:
        conn.close()
    return [row[0] for row in rows] == [first_id, second_id]


def pdf_streams(pdf):
    """Yield decompressed FlateDecode stream bytes from a PDF."""
    import zlib

    for match in re.finditer(rb"stream\r?\n(.*?)endstream", pdf, re.S):
        try:
            yield zlib.decompress(match.group(1))
        except zlib.error:
            continue


def probe(port, password, binary, server_binary, data_dir, timeout):
    """Run the full reconnect verification; return the verdict dict."""
    checks = {}
    base = f"http://127.0.0.1:{port}"
    app_dir = os.path.abspath("persona-sanus")
    binary = os.path.abspath(binary)
    server_binary = os.path.abspath(server_binary)
    own_data_dir = None
    if not data_dir:
        own_data_dir = tempfile.TemporaryDirectory(prefix="reconnect-desktop-")
        data_dir = own_data_dir.name
    data_dir = os.path.abspath(data_dir)
    screenshot = os.path.join(data_dir, "gui.png")
    verdict_extra = {
        "prescription_id": None,
        "rx_number": None,
        "code": None,
        "qr_payload": None,
        "gui_screenshot": "",
    }
    probe_mod = load_block("persona-probe.py")
    pdf_mod = load_block("pdf-probe.py")
    tmpdir = tempfile.TemporaryDirectory(prefix="reconnect-server-")
    database_url = f"sqlite://{tmpdir.name}/server.db"
    gui = None
    server = None

    def finish(code):
        if gui is not None:
            stop_gui(gui)
        if server is not None:
            server.send_signal(signal.SIGTERM)
            try:
                server.wait(timeout=GRACE)
            except subprocess.TimeoutExpired:
                server.kill()
        if own_data_dir is not None and code == 0:
            own_data_dir.cleanup()
        tmpdir.cleanup()

    try:
        checks["binaries"] = os.path.isfile(binary) and os.path.isfile(server_binary)
        if not checks["binaries"]:
            return {"ok": False, "checks": checks, "error": "binary missing", **verdict_extra}
        env = dict(
            os.environ,
            DATABASE_URL=database_url,
            RUSTAUTH_SECRET=secrets.token_hex(24),
            IP="127.0.0.1",
            PORT=str(port),
            PUBLIC_BASE_URL=base,
        )
        server = subprocess.Popen(
            [server_binary],
            cwd=app_dir,
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        checks["health"] = wait_health(base, 60)
        if not checks["health"]:
            checks["seed"] = checks["gui_window"] = checks["drain"] = False
            finish(1)
            return {"ok": False, "checks": checks, "error": "server never healthy", **verdict_extra}

        run = uuid.uuid4().hex
        admin_email = f"probe-reconnect-admin_{run[:8]}@example.com"
        doctor_email = f"probe-reconnect-doctor_{run[:8]}@example.com"
        admin_jar = http.cookiejar.CookieJar()
        doctor_jar = http.cookiejar.CookieJar()
        checks["signup_admin"] = probe_mod.signup_or_signin(
            base, admin_jar, admin_email, password, "Probe Reconnect Admin"
        )
        checks["doctor_signup"] = probe_mod.signup_or_signin(
            base, doctor_jar, doctor_email, password, "Probe Reconnect Doctor"
        )
        doctor_profile = probe_mod.find_profile(base, admin_jar, doctor_email)
        checks["assign_doctor"] = probe_mod.assign_role(base, admin_jar, doctor_profile, "doctor")
        status, me = probe_mod.get_json(base, "/api/me", doctor_jar)
        checks["doctor_me"] = status == 200 and me.get("role") == "doctor"
        if not all(checks[key] for key in ["signup_admin", "doctor_signup", "assign_doctor", "doctor_me"]):
            checks["seed"] = checks["gui_window"] = checks["drain"] = False
            finish(1)
            return {"ok": False, "checks": checks, "error": "account provisioning failed", **verdict_extra}

        device_path = os.path.expanduser(
            "~/Library/Application Support/PersonaSanus/device_id"
        )
        if os.path.isfile(device_path):
            with open(device_path) as handle:
                device_id = handle.read().strip()
        else:
            device_id = str(uuid.uuid4())
            os.makedirs(os.path.dirname(device_path), exist_ok=True)
            with open(device_path, "w") as handle:
                handle.write(device_id)
        status, device = probe_mod.register_device(
            base, doctor_jar, device_id, "Probe Reconnect Device"
        )
        checks["register_device"] = status == 200 and device.get("id") == device_id
        status, block = probe_mod.http_post(base, "/api/rx/blocks", doctor_jar, {})
        block_id = block.get("id")
        start = block.get("startNumber")
        checks["allocate_block"] = (
            status == 200 and bool(block_id) and isinstance(start, int)
        )
        status, patient = probe_mod.http_post(
            base, "/api/patients", doctor_jar,
            {"fullName": f"Paciente Reconexión {run[:6]}", "nationalId": f"9{run[:9]}"},
        )
        patient_id = patient.get("id")
        checks["seed_patient"] = status == 200 and bool(patient_id)
        status, template = probe_mod.http_post(
            base, "/api/templates", admin_jar,
            {"name": f"Plantilla Reconexión {run[:6]}", "ownerType": "clinic"},
        )
        template_id = template.get("id")
        doc = probe_mod.template_document("Paciente: {{patient.name}}")
        status, version = probe_mod.http_post(
            base, f"/api/templates/{template_id}/versions", admin_jar,
            {"docJson": json.dumps(doc), "theme": json.dumps({"margin_mm": 15})},
        )
        version_id = version.get("id")
        status, published = probe_mod.http_post(
            base, f"/api/templates/{template_id}/publish", admin_jar,
            {"versionId": version_id},
        )
        checks["seed_template"] = (
            status == 200 and published.get("currentVersionId") == version_id
        )

        def make_draft(suffix):
            payload = {
                "patientId": patient_id,
                "templateVersionId": version_id,
                "clientId": f"gui-{run[:8]}-{suffix}",
                "diagnosis": "Diagnóstico de sincronización",
                "validity": "3 días",
                "instructions": "Tomar con alimentos.",
            }
            status, draft = probe_mod.http_post(base, "/api/prescriptions", doctor_jar, payload)
            return status, draft, payload

        status_a, draft_a, payload_a = make_draft("a")
        status_b, draft_b, payload_b = make_draft("b")
        id_a, id_b = draft_a.get("id"), draft_b.get("id")
        checks["seed_drafts"] = (
            status_a == 200 and status_b == 200 and bool(id_a) and bool(id_b)
        )
        if not checks["allocate_block"] or not checks["seed_drafts"]:
            checks["seed"] = checks["gui_window"] = checks["drain"] = False
            finish(1)
            return {"ok": False, "checks": checks, "error": "fixture seeding failed", **verdict_extra}

        token = session_token(doctor_jar)
        status, session = probe_mod.get_json(base, "/api/auth/get-session", doctor_jar)
        user_json = (session.get("user") if isinstance(session.get("user"), dict) else None) or {
            "id": (me.get("profile") or {}).get("id", doctor_email),
            "email": doctor_email,
            "name": "Probe Reconnect Doctor",
        }
        checks["session"] = status == 200 and bool(token)

        local_db = os.path.join(data_dir, "desktop-local.sqlite")
        gui = launch_gui(binary, base, data_dir, app_dir)
        deadline = time.time() + 30
        while time.time() < deadline and not os.path.isfile(local_db):
            time.sleep(0.5)
        stop_gui(gui)
        gui = None
        checks["schema_boot"] = os.path.isfile(local_db)
        if not checks["schema_boot"]:
            checks["seed"] = checks["gui_window"] = checks["drain"] = False
            finish(1)
            return {"ok": False, "checks": checks, "error": "first GUI boot created no local DB", **verdict_extra}

        item_id = str(uuid.uuid4())
        rows = [
            (
                "create_patient", "patient", f"gui-patient-{run[:8]}",
                {
                    "fullName": f"Paciente Cola {run[:6]}",
                    "nationalId": f"8{run[:9]}",
                    "birthDate": "",
                    "sex": "",
                    "phone": "",
                    "notes": "",
                    "clientId": f"gui-patient-{run[:8]}",
                },
            ),
            ("create_prescription", "prescription", id_a, payload_a),
            (
                "upsert_prescription_item", "prescription", id_a,
                {
                    "prescriptionId": id_a,
                    "itemId": item_id,
                    "medName": "Paracetamol",
                    "dose": "500 mg",
                    "route": "",
                    "frequency": "",
                    "duration": "",
                    "quantity": "",
                    "notes": "",
                },
            ),
            (
                "sign_prescription", "prescription", id_a,
                {"prescriptionId": id_a, "rxNumber": start, "rxBlockId": block_id},
            ),
            ("create_prescription", "prescription", id_b, payload_b),
            (
                "sign_prescription", "prescription", id_b,
                {"prescriptionId": id_b, "rxNumber": start + 1, "rxBlockId": block_id},
            ),
        ]
        seed_local_db(
            local_db,
            {
                "session_token": token,
                "session_user": json.dumps(user_json),
                "session_profile": json.dumps(me),
                "session_expires_at": rfc3339(time.time() + 30 * 86400),
            },
            rows,
        )
        checks["seed"] = len(outbox_statuses(local_db)) == 6

        gui = launch_gui(binary, base, data_dir, app_dir)
        drained = wait_drained(local_db, timeout)
        checks["gui_window"] = gui.poll() is None
        if checks["gui_window"]:
            shot = subprocess.run(
                ["screencapture", "-o", "-x", screenshot],
                capture_output=True, timeout=60,
            )
            checks["screenshot"] = (
                shot.returncode == 0
                and os.path.isfile(screenshot)
                and os.path.getsize(screenshot) > 10000
            )
        else:
            checks["screenshot"] = False
        checks["drain"] = bool(drained) and all(
            status == "synced" for _, status, _ in drained
        )
        checks["drain_ops"] = [op for op, _, _ in drained] == [row[0] for row in rows]
        stop_gui(gui)
        gui = None

        checks["oldest_first"] = audit_sign_order(
            os.path.join(tmpdir.name, "server.db"), id_a, id_b
        )
        status, detail_a = probe_mod.get_json(base, f"/api/prescriptions/{id_a}", doctor_jar)
        status_b2, detail_b = probe_mod.get_json(
            base, f"/api/prescriptions/{id_b}", doctor_jar
        )
        code_a = (detail_a.get("verification") or {}).get("code")
        code_b = (detail_b.get("verification") or {}).get("code")
        checks["reconciled_a"] = (
            status == 200
            and detail_a.get("status") == "signed"
            and detail_a.get("rxNumber") == start
            and bool(code_a)
        )
        checks["reconciled_b"] = (
            status_b2 == 200
            and detail_b.get("status") == "signed"
            and detail_b.get("rxNumber") == start + 1
            and bool(code_b)
        )
        verdict_extra["prescription_id"] = id_a
        verdict_extra["rx_number"] = start
        verdict_extra["code"] = code_a
        verdict_extra["gui_screenshot"] = screenshot if checks["screenshot"] else ""

        status, headers, pdf = pdf_mod.open_url(
            base, f"/api/prescriptions/{id_a}/pdf", doctor_jar
        )
        checks["pdf_ok"] = status == 200 and pdf.startswith(b"%PDF-")
        box = pdf_mod.media_box(pdf) if checks["pdf_ok"] else None
        checks["pdf_a4"] = bool(box) and all(
            abs(actual - target) <= 1 for actual, target in zip(box, A4)
        )
        checks["pdf_filename"] = bool(pdf_mod.disposition_filename(headers))
        expected = f"{base}/verify/{code_a}"
        payload = ""
        if checks["pdf_ok"]:
            for header, stream in pdf_mod.image_objects(pdf):
                payload = pdf_mod.decode_qr(header, stream, expected, 10)
                if payload:
                    break
        verdict_extra["qr_payload"] = payload
        checks["pdf_qr"] = payload == expected
        number_b16 = str(start).encode("utf-16-be")
        number_ascii = str(start).encode("ascii")
        numbers = any(
            number_b16 in stream or number_ascii in stream for stream in pdf_streams(pdf)
        ) if checks["pdf_ok"] else False
        checks["pdf_number_text"] = numbers

        gate = [
            "binaries", "health", "schema_boot", "session", "seed", "gui_window",
            "screenshot", "drain", "drain_ops", "oldest_first", "reconciled_a",
            "reconciled_b", "pdf_ok", "pdf_a4", "pdf_filename", "pdf_qr",
        ]
        ok = all(checks.get(name) for name in gate)
        finish(0 if ok else 1)
        verdict = {"ok": ok, "checks": checks, **verdict_extra}
        if not ok:
            verdict["error"] = "one or more reconnect checks failed"
        return verdict
    except Exception as error:
        finish(1)
        return {"ok": False, "checks": checks, "error": str(error), **verdict_extra}


def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8123)
    parser.add_argument("--password", default="ProbePass123!")
    parser.add_argument("--binary", default="persona-sanus/target/debug/persona-sanus")
    parser.add_argument(
        "--server-binary",
        default="persona-sanus/target/dx/persona-sanus/release/web/server",
    )
    parser.add_argument("--data-dir", default="")
    parser.add_argument("--timeout", type=int, default=90)
    args = parser.parse_args()
    result = probe(
        args.port, args.password, args.binary,
        args.server_binary, args.data_dir, args.timeout,
    )
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)


if __name__ == "__main__":
    main()
