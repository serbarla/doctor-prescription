#!/usr/bin/env python3
# /// block
# name = "persona-probe"
# summary = "Probe persona-sanus: auth flow, roles, and patient CRUD."
# effects = "network"
# example = ["--base", "http://127.0.0.1:8080", "--email", "probe@example.com", "--password", "ProbePass123!"]
# matches = ['/api/auth/sign-up/email', '/api/auth/(sign-in|sign-out|get-session)', '/api/me', '/api/settings/users', '/api/patients']
#
# [returns]
# keys = ["ok", "checks", "base"]
# doc = "checks maps probe names to booleans; flow=signin checks signin plus session; flow=roles checks bootstrap signup, unaffiliated second signup, role assignment, server-side refusal, and the assigned role; flow=patients checks clinic-scoped CRUD, search, dedupe, administrator read-only, and soft delete"
#
# [params]
# base = { type = "str", required = true, help = "server origin, e.g. http://127.0.0.1:8080" }
# email = { type = "str", default = "probe@example.com", help = "account email for the probe flow" }
# password = { type = "str", default = "ProbePass123!", help = "account password for the probe flow" }
# name = { type = "str", default = "Probe Tester", help = "display name used by signup (full flow only)" }
# flow = { type = "enum", values = ["full", "signin", "roles", "patients"], default = "full", help = "full runs signup then signout then signin; signin only signs in (needs an existing account); roles exercises bootstrap, membership, role assignment, and server-side refusal; patients exercises CRUD, search, dedupe, scoping, and soft delete" }
# db = { type = "path", default = "", help = "SQLite file used by the server for the second-clinic fixture; defaults to DATABASE_URL or persona-sanus/app.db" }
#
# [provenance]
# created = "2026-11-11"
# source = "promoted: session verification flow"
#
# [stamp]
# verified = "8032b8c3ea0e"
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


def probe(base, email, password, name, flow, db_path=""):
    """Run the page + auth probes against base; return the verdict dict."""
    if flow == "roles":
        return probe_roles(base, password, flow)
    if flow == "patients":
        return probe_patients(base, password, flow, db_path)

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
        "--flow", choices=["full", "signin", "roles", "patients"], default="full"
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
