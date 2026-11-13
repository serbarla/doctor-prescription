#!/usr/bin/env python3
# /// block
# name = "persona-probe"
# summary = "Probe persona-sanus: auth, roles, patients, devices, template versions, and prescriptions."
# effects = "network"
# example = ["--base", "http://127.0.0.1:8080", "--email", "probe@example.com", "--password", "ProbePass123!"]
# matches = ['/api/auth/sign-up/email', '/api/auth/(sign-in|sign-out|get-session)', '/api/me', '/api/settings/users', '/api/patients', '/api/devices', '/api/templates', '/api/prescriptions', '/api/rx']
#
# [returns]
# keys = ["ok", "checks", "base", "patient_id", "template_id", "prescription_id", "rx_number", "code"]
# doc = "checks maps probe names to booleans; flow=signin checks signin plus session; flow=roles checks bootstrap signup, unaffiliated second signup, role assignment, server-side refusal, and the assigned role; flow=patients checks clinic-scoped CRUD, search, dedupe, administrator read-only, and soft delete; flow=devices checks registration, session linking, listing, revocation, revoked-session rejection, and re-linking on a fresh sign-in; flow=templates checks clinic-master create, doctor ownership, draft edit, publish, current version, immutable published version, canonical variables, and archive refusal; flow=rx checks number-block allocation and refusal, draft creation with ordered items, signing with a block number and verification code, idempotent replay, supersede, and void"
#
# [params]
# base = { type = "str", required = true, help = "server origin, e.g. http://127.0.0.1:8080" }
# email = { type = "str", default = "probe@example.com", help = "account email for the probe flow" }
# password = { type = "str", default = "ProbePass123!", help = "account password for the probe flow" }
# name = { type = "str", default = "Probe Tester", help = "display name used by signup (full flow only)" }
# flow = { type = "enum", values = ["full", "signin", "roles", "patients", "devices", "templates", "rx"], default = "full", help = "full runs signup then signout then signin; signin only signs in (needs an existing account); roles exercises bootstrap, membership, role assignment, and server-side refusal; patients exercises CRUD, search, dedupe, scoping, and soft delete; devices exercises registration, session linking, listing, revocation, and re-linking; templates exercises creation, draft editing, publishing, immutability, and archiving; rx exercises blocks, drafts, items, signing, replay, supersede, and void" }
# db = { type = "path", default = "", help = "SQLite file used by the server for the second-clinic fixture; defaults to DATABASE_URL or persona-sanus/app.db" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: session verification flow"
#
# [stamp]
# verified = "f2d5774c5e93"
# ///
"""Probe one running persona-sanus server and print a small JSON verdict."""
import argparse
import datetime
import http.cookiejar
import json
import os
import sqlite3
import sys
import urllib.parse
import urllib.request
import urllib.error
import uuid

FULL = ["home", "login", "dashboard", "signup", "session", "signout", "session_after_signout", "signin"]
SIGNIN = ["signin", "session"]
ROLES = [
    "signup_admin", "admin_me", "signup_member", "member_no_clinic",
    "assign_role", "member_forbidden", "member_role",
]
PATIENTS = [
    "signup_admin", "admin_me", "assign_roles", "doctor_me", "secretary_me",
    "create_patient", "search_by_name", "search_by_national_id",
    "update_patient", "read_updated", "secretary_write", "admin_read_only",
    "duplicate_national_id", "likely_duplicate_warning",
    "confirm_duplicate_creates", "cross_clinic_hidden", "soft_delete_hides",
]
DEVICES = [
    "signup_admin", "register_device", "link_session", "list_devices",
    "revoke_device", "revoked_session_rejected", "revoked_device_listed",
    "revoked_registration_rejected", "fresh_signin_relink", "idempotent_replay",
]
RX = [
    "signup_admin", "admin_me", "doctor_signup", "assign_doctor", "doctor_me",
    "register_device", "allocate_block", "duplicate_allocation_refused",
    "create_patient", "create_template", "save_template_version",
    "publish_template", "create_draft", "add_item_one", "add_item_two",
    "reorder_items", "sign_prescription", "rx_number_in_block",
    "verification_created", "block_consumed", "replay_create_idempotent",
    "replay_sign_idempotent", "supersede", "superseded_number",
    "superseded_code", "old_superseded", "old_code_superseded",
    "create_third", "sign_third", "void_third", "void_status",
    "old_code_voided",
]
TEMPLATES = [
    "signup_admin", "admin_me", "create_template", "doctor_signup",
    "assign_doctor_role", "doctor_create", "doctor_forbidden_clinic",
    "save_draft", "edit_draft", "publish_version", "immutable_refused",
    "current_version", "variables_spec", "archive_template", "archived_refused",
]
VARIABLES = [
    "patient.name", "patient.national_id", "patient.age",
    "doctor.name", "doctor.specialty", "doctor.national_id", "doctor.contact",
    "clinic.name", "clinic.address", "clinic.phone",
    "prescription.number", "prescription.date", "prescription.diagnosis",
    "medications[]",
]


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


def http_json(base, method, path, jar, payload=None):
    """Send JSON with a same-origin Origin header; return (status, parsed body)."""
    body = json.dumps(payload).encode() if payload is not None else None
    headers = {"Origin": base}
    if body is not None:
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(
        base + path,
        data=body,
        headers=headers,
        method=method,
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


def http_post(base, path, jar, payload):
    """POST JSON with a same-origin Origin header; return (status, parsed body)."""
    return http_json(base, "POST", path, jar, payload)


def get_json(base, path, jar):
    """GET a JSON endpoint; return (status, parsed dict or {}) — null sessions become {}."""
    status, raw = http_get(base, path, jar)
    try:
        parsed = json.loads(raw or b"null")
    except ValueError:
        return status, {}
    return status, parsed if isinstance(parsed, dict) else {}


def role_accounts(flow):
    """Return the deterministic admin/member emails for the roles flow."""
    return (f"probe-{flow}-admin@example.com", f"probe-{flow}-member@example.com")


def signup_or_signin(base, jar, email, password, name):
    """Sign up a fixture account; sign it in when the deterministic email exists."""
    status, data = http_post(
        base, "/api/auth/sign-up/email", jar,
        {"name": name, "email": email, "password": password, "rememberMe": True},
    )
    if status == 200 and data.get("token"):
        return True
    if status == 409:
        status, data = http_post(
            base, "/api/auth/sign-in/email", jar,
            {"email": email, "password": password, "rememberMe": True},
        )
        return status == 200 and bool(data.get("token"))
    return False


def probe_roles(base, password, flow):
    """Exercise bootstrap, membership, assignment, and server-side refusal."""
    checks = {}
    admin_email, member_email = role_accounts(flow)
    admin_name = "Probe Administrator"
    member_name = "Probe Member"
    admin_jar = http.cookiejar.CookieJar()
    member_jar = http.cookiejar.CookieJar()

    checks["signup_admin"] = signup_or_signin(
        base, admin_jar, admin_email, password, admin_name
    )
    status, admin_me = get_json(base, "/api/me", admin_jar)
    checks["admin_me"] = (
        status == 200
        and admin_me.get("role") == "administrator"
        and isinstance(admin_me.get("clinic"), dict)
    )

    checks["signup_member"] = signup_or_signin(
        base, member_jar, member_email, password, member_name
    )
    status, users = get_json(base, "/api/settings/users", admin_jar)
    member = next(
        (
            user
            for user in users.get("users", [])
            if user.get("email") == member_email
        ),
        None,
    )
    member_profile = member.get("profileId") if member else None
    if member_profile and member.get("role"):
        http_post(
            base, "/api/settings/users/remove", admin_jar,
            {"profileId": member_profile},
        )
    status, member_me = get_json(base, "/api/me", member_jar)
    checks["member_no_clinic"] = (
        status == 200
        and member_me.get("role") is None
        and member_me.get("clinic") is None
    )

    status, _ = http_post(
        base, "/api/settings/users/role", admin_jar,
        {"profileId": member_profile, "role": "secretary"},
    )
    checks["assign_role"] = status == 200

    status, _ = http_post(
        base, "/api/settings/users/role", member_jar,
        {"profileId": member_profile, "role": "doctor"},
    )
    checks["member_forbidden"] = status in (401, 403)

    status, member_me = get_json(base, "/api/me", member_jar)
    checks["member_role"] = (
        status == 200
        and member_me.get("role") == "secretary"
        and isinstance(member_me.get("clinic"), dict)
    )

    ok = all(checks.get(item) for item in ROLES)
    verdict = {
        "ok": ok,
        "checks": {item: bool(checks.get(item)) for item in ROLES},
        "base": base,
    }
    if not ok:
        verdict["error"] = "one or more role checks failed"
    return verdict


def patient_accounts(flow):
    """Return the deterministic admin/staff emails for the patients flow."""
    return {
        role: f"probe-{flow}-{role}@example.com"
        for role in ("admin", "doctor", "secretary")
    }


def now_iso():
    """Return the current UTC timestamp in the server's RFC3339 format."""
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def find_profile(base, admin_jar, email):
    """Return the profileId for an email seen by the administrator."""
    status, users = get_json(base, "/api/settings/users", admin_jar)
    if status != 200:
        return None
    return next(
        (
            user.get("profileId")
            for user in users.get("users", [])
            if user.get("email") == email
        ),
        None,
    )


def assign_role(base, admin_jar, profile_id, role):
    """Assign a clinic role as the administrator; return True on success."""
    if not profile_id:
        return False
    status, _ = http_post(
        base,
        "/api/settings/users/role",
        admin_jar,
        {"profileId": profile_id, "role": role},
    )
    return status == 200


def cleanup_patient(base, jar, national_id):
    """Soft-delete any previous fixture patient with this cédula."""
    path = "/api/patients?q=" + urllib.parse.quote(national_id)
    status, data = get_json(base, path, jar)
    if status != 200:
        return
    for patient in data.get("patients", []):
        http_json(base, "DELETE", f"/api/patients/{patient.get('id')}", jar)


def patient_ids(base, jar, query):
    """Return the ids from one search response."""
    path = "/api/patients?q=" + urllib.parse.quote(query)
    status, data = get_json(base, path, jar)
    if status != 200:
        return set()
    return {
        patient.get("id")
        for patient in data.get("patients", [])
        if patient.get("id")
    }


def resolve_db_path(explicit):
    """Resolve the server SQLite file from the flag, DATABASE_URL, or defaults."""
    if explicit:
        return os.path.abspath(explicit)
    url = os.environ.get("DATABASE_URL", "")
    if url:
        path = url
        for prefix in ("sqlite://", "sqlite:"):
            if path.startswith(prefix):
                path = path[len(prefix):]
                break
        path = path.split("?")[0]
        if path and path != ":memory:":
            return os.path.abspath(path)
    for candidate in ("persona-sanus/app.db", "app.db", "/data/app.db"):
        if os.path.isfile(candidate):
            return os.path.abspath(candidate)
    return None


def seed_foreign_patient(db_path):
    """Insert a patient in a second clinic; return its id or None."""
    if not db_path or not os.path.isfile(db_path):
        return None
    clinic_id = "probe-patients-foreign-clinic"
    patient_id = "probe-patients-foreign"
    at = now_iso()
    try:
        connection = sqlite3.connect(db_path, timeout=10)
        try:
            connection.execute("PRAGMA busy_timeout = 5000")
            connection.execute(
                "INSERT OR IGNORE INTO clinics "
                "(id, name, server_retention_until, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?)",
                (clinic_id, "Clínica de prueba externa", at, at, at),
            )
            connection.execute(
                "INSERT OR IGNORE INTO patients "
                "(id, clinic_id, full_name, national_id, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?)",
                (
                    patient_id,
                    clinic_id,
                    "Paciente de otra clínica",
                    "9999999999",
                    at,
                    at,
                ),
            )
            connection.execute(
                "UPDATE patients SET deleted_at = NULL WHERE id = ?",
                (patient_id,),
            )
            connection.commit()
            return patient_id
        finally:
            connection.close()
    except sqlite3.Error:
        return None


def probe_patients(base, password, flow, db_path):
    """Exercise patient CRUD, scoping, roles, dedupe, and soft delete."""
    checks = {}
    emails = patient_accounts(flow)
    admin_jar = http.cookiejar.CookieJar()
    doctor_jar = http.cookiejar.CookieJar()
    secretary_jar = http.cookiejar.CookieJar()

    checks["signup_admin"] = signup_or_signin(
        base, admin_jar, emails["admin"], password, "Probe Patients Admin"
    )
    status, admin_me = get_json(base, "/api/me", admin_jar)
    checks["admin_me"] = (
        status == 200
        and admin_me.get("role") == "administrator"
        and isinstance(admin_me.get("clinic"), dict)
    )

    doctor_email = emails["doctor"]
    secretary_email = emails["secretary"]
    signup_or_signin(base, doctor_jar, doctor_email, password, "Probe Patients Doctor")
    signup_or_signin(
        base, secretary_jar, secretary_email, password, "Probe Patients Secretary"
    )
    doctor_profile = find_profile(base, admin_jar, doctor_email)
    secretary_profile = find_profile(base, admin_jar, secretary_email)
    checks["assign_roles"] = assign_role(
        base, admin_jar, doctor_profile, "doctor"
    ) and assign_role(base, admin_jar, secretary_profile, "secretary")

    status, doctor_me = get_json(base, "/api/me", doctor_jar)
    checks["doctor_me"] = (
        status == 200
        and doctor_me.get("role") == "doctor"
        and isinstance(doctor_me.get("clinic"), dict)
    )
    status, secretary_me = get_json(base, "/api/me", secretary_jar)
    checks["secretary_me"] = (
        status == 200
        and secretary_me.get("role") == "secretary"
        and isinstance(secretary_me.get("clinic"), dict)
    )

    if not checks["signup_admin"] or not checks["admin_me"]:
        return {
            "ok": False,
            "checks": {item: bool(checks.get(item)) for item in PATIENTS},
            "base": base,
            "error": "administrator bootstrap failed",
        }

    national_id = "0102030405"
    cleanup_patient(base, doctor_jar, national_id)
    patient = {
        "fullName": "Ana María Pérez",
        "nationalId": national_id,
        "birthDate": "1990-02-03",
        "sex": "female",
        "phone": "0999999999",
        "notes": "Paciente de prueba",
    }
    status, created = http_post(base, "/api/patients", doctor_jar, patient)
    patient_id = created.get("id")
    checks["create_patient"] = (
        status == 200
        and created.get("fullName") == patient["fullName"]
        and bool(patient_id)
    )

    checks["search_by_name"] = patient_id in patient_ids(
        base, doctor_jar, "Ana María"
    )
    checks["search_by_national_id"] = patient_id in patient_ids(
        base, doctor_jar, national_id
    )

    updated = dict(patient, fullName="Ana María Pérez Ruiz", phone="0988888888")
    status, body = http_json(
        base, "PUT", f"/api/patients/{patient_id}", doctor_jar, updated
    )
    checks["update_patient"] = (
        status == 200 and body.get("phone") == "0988888888"
    )
    status, body = get_json(base, f"/api/patients/{patient_id}", doctor_jar)
    checks["read_updated"] = (
        status == 200 and body.get("fullName") == "Ana María Pérez Ruiz"
    )

    status, secretary_created = http_post(
        base,
        "/api/patients",
        secretary_jar,
        {"fullName": "Luis Secretaría", "nationalId": "0203040506"},
    )
    checks["secretary_write"] = status == 200 and bool(secretary_created.get("id"))
    if secretary_created.get("id"):
        http_json(
            base,
            "DELETE",
            f"/api/patients/{secretary_created['id']}",
            secretary_jar,
        )

    status, _ = http_json(
        base, "PUT", f"/api/patients/{patient_id}", admin_jar, updated
    )
    checks["admin_read_only"] = status == 403

    status, body = http_post(
        base,
        "/api/patients",
        doctor_jar,
        {"fullName": "Otra Persona", "nationalId": national_id},
    )
    checks["duplicate_national_id"] = (
        status == 409 and body.get("code") == "duplicate_national_id"
    )

    status, body = http_post(
        base,
        "/api/patients",
        doctor_jar,
        {"fullName": "  ANA   María  Pérez   Ruiz "},
    )
    checks["likely_duplicate_warning"] = (
        status == 200
        and body.get("warning") == "likely_duplicate"
        and patient_id in body.get("patientIds", [])
    )

    status, confirmed = http_post(
        base,
        "/api/patients",
        doctor_jar,
        {
            "fullName": "Ana María Pérez Ruiz",
            "confirmDuplicate": True,
        },
    )
    duplicate_id = confirmed.get("id")
    checks["confirm_duplicate_creates"] = (
        status == 200 and bool(duplicate_id) and duplicate_id != patient_id
    )

    foreign_id = seed_foreign_patient(resolve_db_path(db_path))
    if foreign_id:
        status, _ = get_json(base, f"/api/patients/{foreign_id}", doctor_jar)
        admin_status, _ = get_json(base, f"/api/patients/{foreign_id}", admin_jar)
        checks["cross_clinic_hidden"] = (
            status == 404
            and admin_status == 404
            and foreign_id not in patient_ids(base, doctor_jar, "otra clínica")
        )
    else:
        checks["cross_clinic_hidden"] = False

    status, _ = http_json(base, "DELETE", f"/api/patients/{patient_id}", doctor_jar)
    deleted_status, _ = get_json(base, f"/api/patients/{patient_id}", doctor_jar)
    checks["soft_delete_hides"] = (
        status == 200
        and deleted_status == 404
        and patient_id not in patient_ids(base, doctor_jar, national_id)
    )
    if duplicate_id:
        http_json(base, "DELETE", f"/api/patients/{duplicate_id}", doctor_jar)

    ok = all(checks.get(item) for item in PATIENTS)
    verdict = {
        "ok": ok,
        "checks": {item: bool(checks.get(item)) for item in PATIENTS},
        "base": base,
    }
    if not ok:
        verdict["error"] = "one or more patient checks failed"
    return verdict


def devices_by_id(body):
    """Return the device list keyed by id."""
    return {
        device.get("id"): device
        for device in body.get("devices", [])
        if device.get("id")
    }


def register_device(base, jar, device_id, name):
    """Register one device for the signed-in profile."""
    return http_post(
        base,
        "/api/devices/register",
        jar,
        {
            "deviceId": device_id,
            "platform": "web",
            "name": name,
            "appVersion": "0.1.0",
        },
    )


def sign_in_account(base, jar, email, password):
    """Sign the fixture account in; return True on success."""
    status, data = http_post(
        base,
        "/api/auth/sign-in/email",
        jar,
        {"email": email, "password": password, "rememberMe": True},
    )
    return status == 200 and bool(data.get("token"))


def probe_devices(base, password, flow):
    """Exercise device registration, linking, revocation, and re-linking."""
    checks = {}
    email = f"probe-{flow}-admin@example.com"
    jar = http.cookiejar.CookieJar()
    checks["signup_admin"] = signup_or_signin(
        base, jar, email, password, "Probe Devices Admin"
    )
    if not checks["signup_admin"]:
        return {
            "ok": False,
            "checks": {item: False for item in DEVICES},
            "base": base,
            "error": "administrator bootstrap failed",
        }

    run = uuid.uuid4().hex
    revoked_device = f"probe-devices-{run}-revoked"
    active_device = f"probe-devices-{run}-active"

    status, body = register_device(base, jar, revoked_device, "Probe Device Revoked")
    checks["register_device"] = (
        status == 200
        and body.get("id") == revoked_device
        and body.get("platform") == "web"
        and bool(body.get("lastSeenAt"))
    )

    status, listed = get_json(base, "/api/devices", jar)
    devices = devices_by_id(listed)
    checks["link_session"] = status == 200 and listed.get("currentDeviceId") == revoked_device
    checks["list_devices"] = (
        status == 200
        and bool(devices.get(revoked_device, {}).get("lastSeenAt"))
        and devices.get(revoked_device, {}).get("revokedAt") is None
    )

    status, _ = http_post(
        base, "/api/devices/revoke", jar, {"deviceId": revoked_device}
    )
    checks["revoke_device"] = status == 200

    status, session = get_json(base, "/api/auth/get-session", jar)
    me_status, _ = get_json(base, "/api/me", jar)
    checks["revoked_session_rejected"] = (
        status == 200 and not session.get("session") and me_status == 401
    )

    signed_in = sign_in_account(base, jar, email, password)
    status, body = register_device(base, jar, active_device, "Probe Device Active")
    list_status, listed = get_json(base, "/api/devices", jar)
    devices = devices_by_id(listed)
    revoked = devices.get(revoked_device, {})
    checks["revoked_device_listed"] = bool(revoked.get("revokedAt"))
    checks["fresh_signin_relink"] = (
        signed_in
        and status == 200
        and body.get("id") == active_device
        and list_status == 200
        and listed.get("currentDeviceId") == active_device
        and active_device in devices
    )

    status, _ = register_device(base, jar, revoked_device, "Probe Device Revoked")
    list_status, listed = get_json(base, "/api/devices", jar)
    checks["revoked_registration_rejected"] = (
        status == 403
        and list_status == 200
        and listed.get("currentDeviceId") == active_device
    )

    http_post(base, "/api/auth/sign-out", jar, {})
    signed_in = sign_in_account(base, jar, email, password)
    status, _ = register_device(base, jar, active_device, "Probe Device Active")
    list_status, listed = get_json(base, "/api/devices", jar)
    checks["idempotent_replay"] = (
        signed_in
        and status == 200
        and list_status == 200
        and listed.get("currentDeviceId") == active_device
        and sum(
            1
            for device in listed.get("devices", [])
            if device.get("id") == active_device
        )
        == 1
    )

    ok = all(checks.get(item) for item in DEVICES)
    verdict = {
        "ok": ok,
        "checks": {item: bool(checks.get(item)) for item in DEVICES},
        "base": base,
    }
    if not ok:
        verdict["error"] = "one or more device checks failed"
    return verdict


def template_document(body):
    """Return one schema-versioned template document for the probe."""
    return {
        "schema_version": 1,
        "letterhead": {
            "logo_asset_id": None,
            "doctor_details": "Dr. Prueba\nMedicina General",
            "clinic_details": "Clínica de prueba\nQuito",
            "colors": {"primary": "#0f6e64", "accent": "#e8b04b"},
        },
        "body": body,
        "footer": "Firma del médico\n{{doctor.name}}",
        "images": [],
        "medications": [
            {
                "name": "Paracetamol",
                "dose": "500 mg",
                "route": "oral",
                "frequency": "cada 8 horas",
                "duration": "3 días",
                "quantity": "9",
                "notes": "",
            }
        ],
    }


def probe_templates(base, password, flow):
    """Exercise template create, draft edit, publish, immutability, and archive."""
    checks = {}
    admin_email = f"probe-{flow}-admin@example.com"
    doctor_email = f"probe-{flow}-doctor@example.com"
    admin_jar = http.cookiejar.CookieJar()
    doctor_jar = http.cookiejar.CookieJar()
    run = uuid.uuid4().hex

    checks["signup_admin"] = signup_or_signin(
        base, admin_jar, admin_email, password, "Probe Templates Admin"
    )
    status, admin_me = get_json(base, "/api/me", admin_jar)
    checks["admin_me"] = (
        status == 200
        and admin_me.get("role") == "administrator"
        and isinstance(admin_me.get("clinic"), dict)
    )
    if not checks["signup_admin"] or not checks["admin_me"]:
        return {
            "ok": False,
            "checks": {item: bool(checks.get(item)) for item in TEMPLATES},
            "base": base,
            "error": "administrator bootstrap failed",
        }

    status, template = http_post(
        base,
        "/api/templates",
        admin_jar,
        {"name": f"Plantilla maestra {run}", "ownerType": "clinic"},
    )
    template_id = template.get("id")
    checks["create_template"] = (
        status == 200
        and bool(template_id)
        and template.get("ownerType") == "clinic"
        and template.get("status") == "draft"
    )

    checks["doctor_signup"] = signup_or_signin(
        base, doctor_jar, doctor_email, password, "Probe Templates Doctor"
    )
    doctor_profile = find_profile(base, admin_jar, doctor_email)
    checks["assign_doctor_role"] = assign_role(
        base, admin_jar, doctor_profile, "doctor"
    )
    status, doctor_template = http_post(
        base,
        "/api/templates",
        doctor_jar,
        {"name": f"Plantilla médica {run}", "ownerType": "doctor"},
    )
    checks["doctor_create"] = (
        status == 200
        and doctor_template.get("ownerType") == "doctor"
        and bool(doctor_template.get("id"))
    )
    status, _ = http_post(
        base,
        "/api/templates",
        doctor_jar,
        {"name": f"Plantilla prohibida {run}", "ownerType": "clinic"},
    )
    checks["doctor_forbidden_clinic"] = status == 403

    doc = template_document("Paciente: {{patient.name}}")
    status, version = http_post(
        base,
        f"/api/templates/{template_id}/versions",
        admin_jar,
        {"docJson": json.dumps(doc), "theme": json.dumps({"margin_mm": 15})},
    )
    version_id = version.get("id")
    checks["save_draft"] = (
        status == 200
        and bool(version_id)
        and version.get("versionNo") == 1
        and version.get("status") == "draft"
    )

    edited = template_document("Paciente: {{patient.name}} — {{doctor.name}}")
    status, version = http_json(
        base,
        "PUT",
        f"/api/templates/{template_id}/versions/{version_id}",
        admin_jar,
        {"docJson": json.dumps(edited), "theme": json.dumps({"margin_mm": 20})},
    )
    checks["edit_draft"] = (
        status == 200
        and version.get("status") == "draft"
        and "{{doctor.name}}" in version.get("docJson", "")
    )

    status, published = http_post(
        base,
        f"/api/templates/{template_id}/publish",
        admin_jar,
        {"versionId": version_id},
    )
    published_version = published.get("currentVersion") or {}
    checks["publish_version"] = (
        status == 200
        and published.get("status") == "published"
        and published.get("currentVersionId") == version_id
        and published_version.get("status") == "published"
    )

    status, _ = http_json(
        base,
        "PUT",
        f"/api/templates/{template_id}/versions/{version_id}",
        admin_jar,
        {"docJson": json.dumps(template_document("Cambio no permitido"))},
    )
    checks["immutable_refused"] = status == 409

    status, detail = get_json(base, f"/api/templates/{template_id}", admin_jar)
    checks["current_version"] = (
        status == 200 and detail.get("currentVersionId") == version_id
    )
    checks["variables_spec"] = (
        status == 200
        and detail.get("currentVersion", {}).get("variablesSpec") == VARIABLES
    )

    status, archived = http_post(
        base, f"/api/templates/{template_id}/archive", admin_jar, {}
    )
    checks["archive_template"] = (
        status == 200 and archived.get("status") == "archived"
    )
    status, _ = http_post(
        base,
        f"/api/templates/{template_id}/versions",
        admin_jar,
        {"docJson": json.dumps(template_document("Versión tras archivo"))},
    )
    checks["archived_refused"] = status == 409

    ok = all(checks.get(item) for item in TEMPLATES)
    verdict = {
        "ok": ok,
        "checks": {item: bool(checks.get(item)) for item in TEMPLATES},
        "base": base,
    }
    if not ok:
        verdict["error"] = "one or more template checks failed"
    return verdict




def probe_rx(base, password, flow):
    """Exercise number blocks, drafts, ordered items, signing, supersede, and void."""
    checks = {}
    run = uuid.uuid4().hex
    admin_email = f"probe-{flow}-admin@example.com"
    doctor_email = f"probe-{flow}-doctor@example.com"
    admin_jar = http.cookiejar.CookieJar()
    doctor_jar = http.cookiejar.CookieJar()
    verdict_extra = {
        "patient_id": None,
        "template_id": None,
        "prescription_id": None,
        "rx_number": None,
        "code": None,
    }

    checks["signup_admin"] = signup_or_signin(
        base, admin_jar, admin_email, password, "Probe Rx Admin"
    )
    status, admin_me = get_json(base, "/api/me", admin_jar)
    checks["admin_me"] = (
        status == 200
        and admin_me.get("role") == "administrator"
        and isinstance(admin_me.get("clinic"), dict)
    )
    if not checks["signup_admin"] or not checks["admin_me"]:
        return {
            "ok": False,
            "checks": {item: False for item in RX},
            "base": base,
            "error": "administrator bootstrap failed",
            **verdict_extra,
        }

    checks["doctor_signup"] = signup_or_signin(
        base, doctor_jar, doctor_email, password, "Probe Rx Doctor"
    )
    doctor_profile = find_profile(base, admin_jar, doctor_email)
    checks["assign_doctor"] = assign_role(base, admin_jar, doctor_profile, "doctor")
    status, doctor_me = get_json(base, "/api/me", doctor_jar)
    checks["doctor_me"] = (
        status == 200
        and doctor_me.get("role") == "doctor"
        and isinstance(doctor_me.get("doctor"), dict)
    )

    device_id = f"probe-rx-{run}"
    status, device = register_device(base, doctor_jar, device_id, "Probe Rx Device")
    checks["register_device"] = status == 200 and device.get("id") == device_id

    status, block = http_post(base, "/api/rx/blocks", doctor_jar, {})
    block_id = block.get("id")
    checks["allocate_block"] = (
        status == 200
        and bool(block_id)
        and isinstance(block.get("startNumber"), int)
        and isinstance(block.get("endNumber"), int)
    )
    status, _ = http_post(base, "/api/rx/blocks", doctor_jar, {})
    checks["duplicate_allocation_refused"] = status == 409

    status, patient = http_post(
        base,
        "/api/patients",
        doctor_jar,
        {
            "fullName": f"Paciente Rx {run}",
            "nationalId": f"9{run[:9]}",
        },
    )
    patient_id = patient.get("id")
    verdict_extra["patient_id"] = patient_id
    checks["create_patient"] = status == 200 and bool(patient_id)

    status, template = http_post(
        base,
        "/api/templates",
        admin_jar,
        {"name": f"Plantilla Rx {run}", "ownerType": "clinic"},
    )
    template_id = template.get("id")
    verdict_extra["template_id"] = template_id
    checks["create_template"] = status == 200 and bool(template_id)

    doc = template_document("Paciente: {{patient.name}}")
    status, version = http_post(
        base,
        f"/api/templates/{template_id}/versions",
        admin_jar,
        {"docJson": json.dumps(doc), "theme": json.dumps({"margin_mm": 15})},
    )
    version_id = version.get("id")
    checks["save_template_version"] = status == 200 and bool(version_id)
    status, published = http_post(
        base,
        f"/api/templates/{template_id}/publish",
        admin_jar,
        {"versionId": version_id},
    )
    checks["publish_template"] = (
        status == 200 and published.get("currentVersionId") == version_id
    )

    client_id = f"rx-{run}-one"
    draft_payload = {
        "patientId": patient_id,
        "templateVersionId": version_id,
        "clientId": client_id,
        "diagnosis": "Diagnóstico de prueba",
        "validity": "3 días",
        "instructions": "Tomar con alimentos.",
    }
    status, draft = http_post(base, "/api/prescriptions", doctor_jar, draft_payload)
    prescription_id = draft.get("id")
    checks["create_draft"] = (
        status == 200
        and bool(prescription_id)
        and draft.get("status") == "draft"
        and draft.get("clientId") == client_id
    )

    status, item_one = http_post(
        base,
        f"/api/prescriptions/{prescription_id}/items",
        doctor_jar,
        {
            "medName": "Paracetamol",
            "dose": "500 mg",
            "route": "oral",
            "frequency": "cada 8 horas",
            "duration": "3 días",
            "quantity": "9",
        },
    )
    item_one_id = item_one.get("id")
    checks["add_item_one"] = status == 200 and bool(item_one_id)
    status, item_two = http_post(
        base,
        f"/api/prescriptions/{prescription_id}/items",
        doctor_jar,
        {
            "medName": "Ibuprofeno",
            "dose": "400 mg",
            "route": "oral",
            "frequency": "cada 12 horas",
            "duration": "2 días",
            "quantity": "4",
        },
    )
    item_two_id = item_two.get("id")
    checks["add_item_two"] = (
        status == 200 and bool(item_two_id) and item_two_id != item_one_id
    )

    status, ordered = http_post(
        base,
        f"/api/prescriptions/{prescription_id}/items/reorder",
        doctor_jar,
        {"itemIds": [item_two_id, item_one_id]},
    )
    items = ordered.get("items", [])
    checks["reorder_items"] = (
        status == 200
        and [item.get("id") for item in items] == [item_two_id, item_one_id]
        and [item.get("position") for item in items] == [1, 2]
    )

    status, signed = http_post(
        base,
        f"/api/prescriptions/{prescription_id}/sign",
        doctor_jar,
        {"acknowledgeWarnings": True},
    )
    rx_number = signed.get("rxNumber")
    verification = signed.get("verification") or {}
    code = verification.get("code")
    verification_status = verification.get("status")
    verdict_extra["prescription_id"] = prescription_id
    verdict_extra["rx_number"] = rx_number
    verdict_extra["code"] = code
    checks["sign_prescription"] = (
        status == 200
        and signed.get("id") == prescription_id
        and signed.get("status") == "signed"
        and isinstance(rx_number, int)
        and signed.get("rxBlockId") == block_id
    )
    checks["rx_number_in_block"] = (
        isinstance(rx_number, int)
        and isinstance(block.get("startNumber"), int)
        and isinstance(block.get("endNumber"), int)
        and block["startNumber"] <= rx_number <= block["endNumber"]
    )
    checks["verification_created"] = (
        bool(code)
        and code != verification.get("codeHash")
        and verification_status == "valid"
    )

    status, block_status = get_json(base, "/api/rx/block-status", doctor_jar)
    checks["block_consumed"] = (
        status == 200
        and block_status.get("active") is True
        and block_status.get("consumed", 0) >= 1
        and block_status.get("remaining", 0)
        <= block.get("endNumber", 0) - block.get("startNumber", 0) + 1
    )

    status, replay = http_post(base, "/api/prescriptions", doctor_jar, draft_payload)
    checks["replay_create_idempotent"] = (
        status == 200
        and replay.get("id") == prescription_id
        and replay.get("rxNumber") == rx_number
    )

    status, count_before = get_json(
        base, f"/api/prescriptions?patientId={patient_id}", doctor_jar
    )
    status, replay_sign = http_post(
        base,
        f"/api/prescriptions/{prescription_id}/sign",
        doctor_jar,
        {"acknowledgeWarnings": True},
    )
    status, count_after = get_json(
        base, f"/api/prescriptions?patientId={patient_id}", doctor_jar
    )
    checks["replay_sign_idempotent"] = (
        replay_sign.get("id") == prescription_id
        and replay_sign.get("rxNumber") == rx_number
        and (replay_sign.get("verification") or {}).get("code") == code
        and len(count_before.get("prescriptions", []))
        == len(count_after.get("prescriptions", []))
    )

    supersede_client = f"rx-{run}-two"
    status, corrected = http_post(
        base,
        f"/api/prescriptions/{prescription_id}/supersede",
        doctor_jar,
        {"clientId": supersede_client, "acknowledgeWarnings": True},
    )
    corrected_id = corrected.get("id")
    corrected_number = corrected.get("rxNumber")
    corrected_code = (corrected.get("verification") or {}).get("code")
    checks["supersede"] = (
        status == 200
        and bool(corrected_id)
        and corrected_id != prescription_id
        and corrected.get("status") == "signed"
        and corrected.get("supersedesId") == prescription_id
        and (corrected.get("verification") or {}).get("status") == "valid"
    )
    checks["superseded_number"] = (
        isinstance(corrected_number, int)
        and isinstance(rx_number, int)
        and corrected_number > rx_number
    )
    checks["superseded_code"] = (
        bool(corrected_code) and corrected_code != code
    )
    status, old = get_json(base, f"/api/prescriptions/{prescription_id}", doctor_jar)
    checks["old_superseded"] = (
        status == 200
        and old.get("status") == "superseded"
        and (old.get("verification") or {}).get("status") == "superseded"
    )
    checks["old_code_superseded"] = (
        (old.get("verification") or {}).get("code") == code
    )

    status, third = http_post(
        base,
        "/api/prescriptions",
        doctor_jar,
        {
            "patientId": patient_id,
            "templateVersionId": version_id,
            "clientId": f"rx-{run}-three",
        },
    )
    third_id = third.get("id")
    checks["create_third"] = status == 200 and bool(third_id)
    http_post(
        base,
        f"/api/prescriptions/{third_id}/items",
        doctor_jar,
        {"medName": "Loratadina", "dose": "10 mg"},
    )
    status, third_signed = http_post(
        base,
        f"/api/prescriptions/{third_id}/sign",
        doctor_jar,
        {"acknowledgeWarnings": True},
    )
    third_code = (third_signed.get("verification") or {}).get("code")
    checks["sign_third"] = (
        status == 200
        and third_signed.get("status") == "signed"
        and bool(third_code)
    )

    status, voided = http_post(
        base,
        f"/api/prescriptions/{third_id}/void",
        doctor_jar,
        {"voidReason": "Error de dosificación"},
    )
    checks["void_third"] = (
        status == 200
        and voided.get("status") == "voided"
        and voided.get("voidReason") == "Error de dosificación"
    )
    status, third_after = get_json(base, f"/api/prescriptions/{third_id}", doctor_jar)
    checks["void_status"] = (
        status == 200
        and third_after.get("status") == "voided"
        and (third_after.get("verification") or {}).get("status") == "voided"
    )
    status, old_after = get_json(base, f"/api/prescriptions/{prescription_id}", doctor_jar)
    checks["old_code_voided"] = (
        (old_after.get("verification") or {}).get("status") == "superseded"
    )

    ok = all(checks.get(item) for item in RX)
    verdict = {
        "ok": ok,
        "checks": {item: bool(checks.get(item)) for item in RX},
        "base": base,
        **verdict_extra,
    }
    if not ok:
        verdict["error"] = "one or more prescription checks failed"
    return verdict


def probe(base, email, password, name, flow, db_path=""):
    """Run the page + auth probes against base; return the verdict dict."""
    if flow == "roles":
        return probe_roles(base, password, flow)
    if flow == "patients":
        return probe_patients(base, password, flow, db_path)
    if flow == "devices":
        return probe_devices(base, password, flow)
    if flow == "templates":
        return probe_templates(base, password, flow)
    if flow == "rx":
        return probe_rx(base, password, flow)

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
    parser.add_argument(
        "--flow",
        choices=["full", "signin", "roles", "patients", "devices", "templates", "rx"],
        default="full",
    )
    parser.add_argument("--db", default="")
    args = parser.parse_args()
    return probe(
        args.base, args.email, args.password, args.name, args.flow, args.db
    )


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
