/// Discover real network hostnames and interface names from the local system.
/// These are used purely for display flavor — no network access is performed.
use std::process::Command;

#[derive(Debug, Clone)]
pub struct LocalNetworkInfo {
    pub hostname: String,
    pub interfaces: Vec<String>,
    pub dns_servers: Vec<String>,
    pub gateway: Option<String>,
}

impl LocalNetworkInfo {
    pub fn discover() -> Self {
        Self {
            hostname: discover_hostname(),
            interfaces: discover_interfaces(),
            dns_servers: discover_dns_servers(),
            gateway: discover_gateway(),
        }
    }
}

fn discover_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".into())
}

fn discover_interfaces() -> Vec<String> {
    // Read interface names from /sys/class/net/
    std::fs::read_dir("/sys/class/net")
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().into_string().unwrap_or_default())
                .filter(|name| !name.is_empty() && name != "lo")
                .collect()
        })
        .unwrap_or_else(|_| vec!["eth0".into()])
}

fn discover_dns_servers() -> Vec<String> {
    std::fs::read_to_string("/etc/resolv.conf")
        .map(|content| {
            content
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.starts_with("nameserver") {
                        line.split_whitespace().nth(1).map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .take(3)
                .collect()
        })
        .unwrap_or_else(|_| vec!["8.8.8.8".into()])
}

fn discover_gateway() -> Option<String> {
    Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout).ok().and_then(|s| {
                s.split_whitespace()
                    .skip_while(|w| *w != "via")
                    .nth(1)
                    .map(|s| s.to_string())
            })
        })
}
