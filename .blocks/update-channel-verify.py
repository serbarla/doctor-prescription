#!/usr/bin/env python3
# /// block
# name = "update-channel-verify"
# summary = "Integration probe: compiles the real update.rs/device.rs in a throwaway crate and exercises rollout, transport, digest, and bad-signature paths (debug + release)."
# effects = "network"
# example = ["--repo", ".", "--port", "8092"]
# matches = ['cargo build', 'update']
#
# [returns]
# doc = "builds the probe crate in debug and release, serves a fake installer over loopback HTTP, and asserts: two device ids straddling the 50% bucket split exactly one/one, 0% none, 100% both, empty device refused, device id stable across calls, non-loopback HTTP refused, loopback HTTP refused in release even with the insecure flag, loopback HTTP allowed in debug with the flag, digest mismatch refused, bad signature refused. Exits non-zero with the failing check on failure."
# keys = ["ok", "checks"]
#
# [params]
# repo = { type = "path", default = ".", help = "repository root holding persona-sanus/src" }
# port = { type = "int", default = 8092, min = 1, max = 65535, help = "loopback port for the fake installer" }
#
# [stamp]
# verified = "never"
# ///
"""Verify the 13-M update-channel fixes against the real update.rs code."""
import argparse
import functools
import hashlib
import http.server
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading

PROBE_MANIFEST = """[package]
name = "update-probe"
version = "0.1.0"
edition = "2021"

[dependencies]
base64 = "0.22"
ed25519-dalek = "2"
sha2 = "0.10"
hex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "blocking"] }
uuid = { version = "1", features = ["v4"] }
"""

PROBE_MAIN = r"""
#[path = "{repo}/persona-sanus/src/device.rs"]
mod device;
#[path = "{repo}/persona-sanus/src/update.rs"]
mod update;

use base64::Engine;
use ed25519_dalek::Signer;
use update::{UpdateError, Manifest};
use sha2::Digest;

fn keypair() -> (ed25519_dalek::SigningKey, String) {
    let key = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let public = base64::engine::general_purpose::STANDARD
        .encode(key.verifying_key().to_bytes());
    (key, public)
}

fn manifest(key: &ed25519_dalek::SigningKey, version: &str, rollout: u8, url: &str, sha256: &str) -> Manifest {
    let payload = update::canonical_manifest(version, rollout, url, sha256);
    let signature = base64::engine::general_purpose::STANDARD
        .encode(key.sign(payload.as_bytes()).to_bytes());
    Manifest {
        version: version.to_owned(),
        rollout_percent: rollout,
        url: url.to_owned(),
        sha256: sha256.to_owned(),
        signature,
    }
}

fn check(name: &str, ok: bool, failures: &mut Vec<String>) {
    if ok {
        println!("{{ \"check\": \"{name}\", \"ok\": true }}");
    } else {
        println!("{{ \"check\": \"{name}\", \"ok\": false }}");
        failures.push(name.to_owned());
    }
}

fn scenario_rollout(failures: &mut Vec<String>) {
    let (key, public) = keypair();
    let mut inside = String::new();
    let mut outside = String::new();
    for i in 0..1000 {
        let id = format!("probe-device-{i}");
        if update::in_rollout(&id, 50) && inside.is_empty() {
            inside = id;
        } else if !update::in_rollout(&id, 50) && outside.is_empty() {
            outside = id;
        }
        if !inside.is_empty() && !outside.is_empty() {
            break;
        }
    }
    check("rollout-ids-found", !inside.is_empty() && !outside.is_empty(), failures);
    let digest = "a".repeat(64);
    let url = "https://updates.example.com/installer.bin";
    let m50 = manifest(&key, "9.9.9", 50, url, &digest);
    let a = update::evaluate(&m50, &public, "0.1.0", &inside);
    let b = update::evaluate(&m50, &public, "0.1.0", &outside);
    check("rollout-50-one-in-one-out", a.is_ok() && matches!(b, Err(UpdateError::NotInRollout)), failures);
    let m0 = manifest(&key, "9.9.9", 0, url, &digest);
    check("rollout-0-none", update::evaluate(&m0, &public, "0.1.0", &inside).is_err() && update::evaluate(&m0, &public, "0.1.0", &outside).is_err(), failures);
    let m100 = manifest(&key, "9.9.9", 100, url, &digest);
    check("rollout-100-both", update::evaluate(&m100, &public, "0.1.0", &inside).is_ok() && update::evaluate(&m100, &public, "0.1.0", &outside).is_ok(), failures);
    check("rollout-empty-device-refused", matches!(update::evaluate(&m100, &public, "0.1.0", ""), Err(UpdateError::EmptyDeviceId)), failures);
    let first = crate::device::device_id();
    let second = crate::device::device_id();
    check("device-id-stable", !first.is_empty() && first == second, failures);
    std::env::set_var("PERSONA_DEVICE_ID", "   ");
    let resolved = update::resolve_device_id();
    check("resolve-device-fallback", !resolved.is_empty(), failures);
    std::env::set_var("PERSONA_DEVICE_ID", &inside);
    check("resolve-device-override", update::resolve_device_id() == inside, failures);
    std::env::remove_var("PERSONA_DEVICE_ID");
    let mut tampered = manifest(&key, "9.9.9", 100, url, &digest);
    tampered.version = "9.9.8".to_owned();
    check("bad-signature-refused", matches!(update::evaluate(&tampered, &public, "0.1.0", &inside), Err(UpdateError::BadSignature)), failures);
}

fn scenario_transport(failures: &mut Vec<String>) {
    let (key, public) = keypair();
    let digest = "a".repeat(64);
    let https = manifest(&key, "9.9.9", 100, "https://updates.example.com/installer.bin", &digest);
    check("transport-https-allowed", update::evaluate(&https, &public, "0.1.0", "dev-1").is_ok(), failures);
    let remote_http = manifest(&key, "9.9.9", 100, "http://updates.example.com/installer.bin", &digest);
    check("transport-remote-http-refused", matches!(update::evaluate(&remote_http, &public, "0.1.0", "dev-1"), Err(UpdateError::InsecureTransport)), failures);
    let loopback = manifest(&key, "9.9.9", 100, "http://127.0.0.1:8092/installer.bin", &digest);
    let result = update::evaluate(&loopback, &public, "0.1.0", "dev-1");
    if cfg!(debug_assertions) {
        check("transport-loopback-debug-allowed", result.is_ok(), failures);
    } else {
        check("transport-loopback-release-refused", matches!(result, Err(UpdateError::InsecureTransport)), failures);
    }
}

fn scenario_digest(failures: &mut Vec<String>) {
    let (key, public) = keypair();
    let installer = b"fake-installer-bytes";
    let digest = hex::encode(sha2::Sha256::digest(installer));
    let url = "http://127.0.0.1:8092/installer.bin";
    let good = manifest(&key, "9.9.9", 100, url, &digest);
    check("digest-evaluate-ok", update::evaluate(&good, &public, "0.1.0", "dev-1").is_ok(), failures);
    let staged = update::stage(&good);
    let exe = std::env::current_exe().unwrap();
    let staged_path = exe.with_extension("new");
    check("digest-stage-ok", staged.is_ok() && std::fs::read(&staged_path).map(|b| b == installer).unwrap_or(false), failures);
    let wrong = manifest(&key, "9.9.9", 100, url, &"b".repeat(64));
    std::fs::remove_file(&staged_path).ok();
    let refused = update::stage(&wrong);
    check("digest-mismatch-refused", matches!(refused, Err(UpdateError::DigestMismatch)) && !staged_path.exists(), failures);
    let empty = manifest(&key, "9.9.9", 100, url, "");
    check("digest-empty-refused", matches!(update::evaluate(&empty, &public, "0.1.0", "dev-1"), Err(UpdateError::BadSignature)), failures);
}

fn main() {
    let scenario = std::env::args().nth(1).unwrap_or_default();
    let mut failures: Vec<String> = Vec::new();
    match scenario.as_str() {
        "rollout" => scenario_rollout(&mut failures),
        "transport" => scenario_transport(&mut failures),
        "digest" => scenario_digest(&mut failures),
        _ => {
            eprintln!("unknown scenario");
            std::process::exit(2);
        }
    }
    if failures.is_empty() {
        println!("{{\"ok\": true, \"scenario\": \"{scenario}\"}}");
    } else {
        println!("{{\"ok\": false, \"failures\": {failures:?}}}");
        std::process::exit(1);
    }
}
"""

def run(cmd, cwd, env=None, timeout=1800):
    proc = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
    return proc


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=".")
    parser.add_argument("--port", type=int, default=8092)
    args = parser.parse_args()
    repo = os.path.abspath(args.repo)
    checks = {}
    tmp = tempfile.mkdtemp(prefix="update-probe-")
    try:
        crate = os.path.join(tmp, "crate")
        os.makedirs(os.path.join(crate, "src"))
        with open(os.path.join(crate, "Cargo.toml"), "w") as handle:
            handle.write(PROBE_MANIFEST)
        with open(os.path.join(crate, "src", "main.rs"), "w") as handle:
            handle.write(PROBE_MAIN.replace("{repo}", repo))
        env = dict(os.environ, CARGO_TERM_COLOR="never")
        for args_build in (["cargo", "build", "--offline"], ["cargo", "build", "--offline", "--release"]):
            proc = run(args_build, crate, env)
            checks["build:" + ("release" if "--release" in args_build else "debug")] = proc.returncode == 0
            if proc.returncode != 0:
                print(json.dumps({"ok": False, "error": "probe build failed", "stderr": proc.stderr[-2000:], "checks": checks}))
                return 1
        serve_dir = os.path.join(tmp, "serve")
        os.makedirs(serve_dir)
        installer = b"fake-installer-bytes"
        with open(os.path.join(serve_dir, "installer.bin"), "wb") as handle:
            handle.write(installer)
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=serve_dir)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        flag_env = dict(env, PERSONA_UPDATE_ALLOW_INSECURE_HTTP="1")
        binaries = {
            "debug": os.path.join(crate, "target", "debug", "update-probe"),
            "release": os.path.join(crate, "target", "release", "update-probe"),
        }
        scenarios = [("debug", "rollout", env), ("debug", "transport", flag_env), ("debug", "digest", flag_env),
                     ("release", "rollout", env), ("release", "transport", flag_env)]
        for profile, scenario, run_env in scenarios:
            proc = run([binaries[profile], scenario], tmp, run_env, timeout=120)
            ok = proc.returncode == 0
            checks[f"{profile}:{scenario}"] = ok
            for line in proc.stdout.splitlines():
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if "check" in entry:
                    checks[entry["check"]] = bool(entry["ok"])
            if not ok:
                print(json.dumps({"ok": False, "error": f"scenario {profile}:{scenario} failed", "stdout": proc.stdout[-2000:], "stderr": proc.stderr[-1000:], "checks": checks}))
                return 1
        server.shutdown()
        ok = all(checks.values())
        print(json.dumps({"ok": ok, "checks": checks}))
        return 0 if ok else 1
    finally:
        if os.path.isdir(tmp):
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
