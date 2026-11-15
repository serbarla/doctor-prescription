# Divergence tracking: 12-L rate-limit proxy trust

Source specs: `specs/done/12-L-hardening.md`, objective 3 ("per-IP limits on `/api/auth/*` … returning `429`") and its criterion `objective 5`; `specs/done/09-I-public-verification.md`, objective 3 ("per-IP rate limit at the HTTP layer on `/api/verify/:code` and `/verify/:code` … in-memory keyed by IP").
Resolving spec: none — requires a new hardening spec for proxy trust and limiter state. Until then 12-L and 09-I remain complete only for direct-to-app traffic.

Severity: P2 — behind the documented TLS terminator the limits are either per-proxy (one client can exhaust the clinic's budget) or bypassable by spoofing `X-Forwarded-For`. The maps also grow without bound in a public deployment.

## What's broken

- `persona-sanus/src/verify_server.rs:97-102`: `client_ip` returns `ConnectInfo<SocketAddr>` immediately whenever it is present, and it is always present because `main.rs:110-118` serves with `into_make_service_with_connect_info`. The `forwarded_ip` fallback at lines 104-113 is therefore dead code. Behind a reverse proxy every request shares the proxy IP, so `VERIFY_RATE_LIMIT_PER_MINUTE=10` is a global limit: one abusive client makes the public verifier return `429` for everyone.
- `persona-sanus/src/auth_server.rs:244-250`: `forwarded_ip` takes `.split(',').next()` — the leftmost `X-Forwarded-For` value, which is client-controlled when the trusted proxy appends. The value is used only when the peer is loopback; a same-host Nginx/Caddy proxy is loopback, so a client can send a fresh leftmost IP on each request and bypass the 20/min auth limit. A Docker bridge proxy is non-loopback, so the code ignores XFF there and gives every client the proxy's single bucket.
- Neither module defines which peers may supply `X-Forwarded-For`. The spec says per-IP but no trust boundary is configured; direct and proxied deployments therefore behave differently without any startup error.
- Both limiters keep `HashMap<IpAddr, VecDeque<Instant>>` entries forever (`auth_server.rs:191-227`, `verify_server.rs:12-52`). Expired timestamps are popped only when that same IP returns; an empty deque remains. `/api/verify/:code` is public and unauthenticated. Scanners, spoofed leftmost values through a loopback proxy (the auth map), or ordinary long-lived traffic grow the maps without bound until restart.
- The probes prove the limit in the direct-to-app topology only (`persona-probe --flow security` and `--flow verify`). They do not prove behavior behind the TLS proxy the runbook requires, which is why this passed.

## Why each fix was fenced

- The correct client-IP rule needs an ops decision: a trusted-proxy CIDR list, a fixed hop count, or a proxy-injected header that the app overwrites. Taking the first XFF value is spoofable; taking the last value trusts a direct client. No topology is specified in 12-L, 09-I, the runbook, or compose.
- Docker assigns the app and a future proxy dynamic addresses; a fixed CIDR default would be wrong. The shipped default should remain "trust no forwarding headers", with an explicit opt-in for the documented proxy.
- The map bound needs a policy choice (cap + eviction, or periodic drain); changing it ad hoc could hide an attack rather than bound memory.
- The 2026-11-15 pass was report-only. No proxy fixture or container build was run; `cargo`/`dx` are not on this shell's PATH.

Verified 2026-11-15: 0 fix attempts — static reading of both limiters, the middleware stack, and the serving entry point; the XFF trust and map-retention paths are exact. Fenced by the missing trust-boundary decision and the spec process, not by tooling.

## How to clear it

- [ ] Add one shared client-IP helper used by auth and verify. Default: ignore `X-Forwarded-For` entirely, use the socket peer.
- [ ] Add `PERSONA_TRUSTED_PROXY_CIDRS` (comma-separated CIDRs, default empty). Only when the peer is inside a configured CIDR, parse XFF from right to left, skip trusted proxies, and return the first untrusted address. Reject malformed or empty results by falling back to the peer; never take the leftmost value by default.
- [ ] Consider `PERSONA_TRUSTED_PROXY_HOPS` instead if the deployment cannot enumerate CIDRs. Do not ship both with ambiguous precedence.
- [ ] Bound both limiter maps: remove empty deques when the window is pruned, cap entries (e.g. 10,000), and evict idle/oldest keys on insert. Alternatively use a maintained limiter crate; keep the same `429` semantics.
- [ ] Document the selected topology in `context/runbook.md` and `context/architecture.md` §14. If compose fronts the app with a proxy, give the compose network a fixed subnet and set the trusted CIDR; if not, leave the variable empty.
- [ ] Verify direct traffic: send repeated requests with `X-Forwarded-For: 198.51.100.1` to a direct app peer; the bucket must remain keyed by the real peer, so the limit still trips at `VERIFY_RATE_LIMIT_PER_MINUTE`/`AUTH_RATE_LIMIT_PER_MINUTE`.
- [ ] Verify proxied traffic with two real client addresses: each must get its own budget, and one client's `429` must not `429` the other.
- [ ] Verify spoofing: when the leftmost XFF is random and the peer is trusted, the rightmost untrusted address must still determine the bucket; changing the leftmost value must not bypass the limit.
- [ ] Verify memory: drive thousands of distinct keys through the limiter and confirm map size stays capped/empty entries are pruned.
- [ ] Run `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow security` and `python3 .blocks/persona-probe.py --base http://127.0.0.1:8080 --flow verify`, then mark 12-L objective 3/5 and 09-I objective 3 `[x]`. If 12-L and 09-I remain in `specs/done/`, reopen or cross-reference them from the remediation spec; delete this file only after the behavior is verified behind the documented proxy.
