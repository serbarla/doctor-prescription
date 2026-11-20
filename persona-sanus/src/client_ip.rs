use std::collections::VecDeque;
use std::net::IpAddr;
use std::time::Instant;

use axum::http::{HeaderMap, HeaderName};

pub const MAX_WINDOWS: usize = 10_000;

pub fn trusted_cidrs() -> &'static Vec<(IpAddr, u8)> {
    static CIDRS: std::sync::LazyLock<Vec<(IpAddr, u8)>> = std::sync::LazyLock::new(|| {
        std::env::var("PERSONA_TRUSTED_PROXY_CIDRS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() {
                    return None;
                }
                if let Some((addr, bits)) = part.split_once('/') {
                    let addr = addr.trim().parse().ok()?;
                    let bits = bits.trim().parse().ok()?;
                    let max = match addr {
                        IpAddr::V4(_) => 32,
                        IpAddr::V6(_) => 128,
                    };
                    if bits > max {
                        return None;
                    }
                    Some((addr, bits))
                } else {
                    let addr = part.parse().ok()?;
                    let max = match addr {
                        IpAddr::V4(_) => 32,
                        IpAddr::V6(_) => 128,
                    };
                    Some((addr, max))
                }
            })
            .collect()
    });
    &CIDRS
}

fn in_cidr(ip: IpAddr, network: IpAddr, bits: u8) -> bool {
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(network)) => {
            if bits == 0 {
                return true;
            }
            let mask = u32::MAX << (32 - bits);
            u32::from(ip) & mask == u32::from(network) & mask
        }
        (IpAddr::V6(ip), IpAddr::V6(network)) => {
            if bits == 0 {
                return true;
            }
            let mask = u128::MAX << (128 - bits);
            u128::from(ip) & mask == u128::from(network) & mask
        }
        _ => false,
    }
}

pub fn is_trusted(ip: IpAddr) -> bool {
    trusted_cidrs()
        .iter()
        .any(|(network, bits)| in_cidr(ip, *network, *bits))
}

pub fn client_ip(peer: IpAddr, headers: &HeaderMap) -> IpAddr {
    if !is_trusted(peer) {
        return peer;
    }
    let Some(value) = headers.get(HeaderName::from_static("x-forwarded-for")) else {
        return peer;
    };
    let Ok(value) = value.to_str() else {
        return peer;
    };
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.is_empty() || parts.iter().all(|part| part.is_empty()) {
        return peer;
    }
    let mut ips = Vec::with_capacity(parts.len());
    for part in parts {
        let Ok(ip) = part.parse::<IpAddr>() else {
            return peer;
        };
        ips.push(ip);
    }
    for ip in ips.iter().rev() {
        if !is_trusted(*ip) {
            return *ip;
        }
    }
    peer
}

pub fn admit(
    windows: &mut std::collections::HashMap<IpAddr, VecDeque<Instant>>,
    ip: IpAddr,
    now: Instant,
    cutoff: Instant,
    limit: u32,
) -> bool {
    if let Some(entry) = windows.get_mut(&ip) {
        while entry.front().is_some_and(|moment| *moment <= cutoff) {
            entry.pop_front();
        }
        if entry.is_empty() {
            windows.remove(&ip);
        } else if entry.len() >= limit as usize {
            return false;
        } else {
            entry.push_back(now);
            return true;
        }
    }
    if windows.len() >= MAX_WINDOWS {
        windows.retain(|_, deque| deque.back().is_some_and(|moment| *moment > cutoff));
        if windows.len() >= MAX_WINDOWS {
            let oldest = windows
                .iter()
                .min_by_key(|(_, deque)| deque.back().copied())
                .map(|(key, _)| *key);
            if let Some(key) = oldest {
                windows.remove(&key);
            }
        }
    }
    windows.insert(ip, VecDeque::from([now]));
    true
}
