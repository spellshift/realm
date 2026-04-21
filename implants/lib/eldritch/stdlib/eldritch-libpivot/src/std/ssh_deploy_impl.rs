use crate::std::Session;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::{Result, anyhow};
use eldritch_core::Value;
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};

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

fn resolve_payload_dst(payload_dst: Option<&str>) -> String {
    if let Some(dst) = payload_dst {
        return dst.to_string();
    }
    "/tmp/payload".to_string()
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

/// Outcome of a single `(ip, principal)` credential attempt against a host.
///
/// Each attempted credential produces one `DeployOutcome`. A `status` of
/// `"success"` means the SSH handshake, authentication, optional payload
/// copy, and command execution all completed (and `stdout`/`stderr` carry the
/// remote command's output). A `status` of `"failed"` means one of those
/// steps failed; `error` carries a human-readable description of the
/// failure, including (for negotiation errors) the server's advertised
/// algorithm list obtained via a raw KEXINIT probe.
#[derive(Debug, Clone)]
struct DeployOutcome {
    principal: String,
    status: &'static str,
    stdout: String,
    stderr: String,
    error: String,
}

const DEFAULT_TIMEOUT_SECS: u64 = 5;
const DEFAULT_RETRIES: u32 = 0;
const PROBE_TIMEOUT_SECS: u64 = 5;
/// Default maximum number of concurrent SSH workers when the caller does not
/// specify `workers`. Chosen to be high enough to hide per-host network
/// latency on small fleets while remaining conservative with respect to
/// sockets and memory on a beacon.
const DEFAULT_WORKERS: usize = 10;

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

/// Resolve the caller-supplied `workers` value, applying the default when
/// unset. Negative or zero values are rejected. The returned value is the
/// user's *requested* worker ceiling; the actual pool size is further
/// clamped to `num_targets` by [`ssh_deploy`] so that we never spawn idle
/// workers ("1 worker per target ip up to the limit of workers").
fn resolve_workers(workers: Option<i64>) -> Result<usize> {
    match workers {
        None => Ok(DEFAULT_WORKERS),
        Some(w) if w <= 0 => Err(anyhow!("workers must be a positive integer, got {w}")),
        Some(w) => Ok(w as usize),
    }
}

/// Advertised algorithm name-lists from a remote SSH server's KEXINIT packet.
///
/// The fields map to the lists defined by RFC 4253 § 7.1 and are populated by
/// [`probe_server_kexinit`]. `languages_*` and similar fields that we do not
/// currently surface in error messages are intentionally omitted.
#[derive(Debug, Clone, Default)]
struct ServerAlgos {
    kex: Vec<String>,
    server_host_key: Vec<String>,
    cipher_c2s: Vec<String>,
    cipher_s2c: Vec<String>,
    mac_c2s: Vec<String>,
    mac_s2c: Vec<String>,
}

/// Read a single `uint32`-prefixed, comma-separated name-list from an SSH
/// binary packet payload at offset `i`, advancing `i` past the list.
fn read_namelist(payload: &[u8], i: &mut usize) -> Result<Vec<String>> {
    let len_end = i
        .checked_add(4)
        .ok_or_else(|| anyhow!("name-list offset overflow"))?;
    if payload.len() < len_end {
        return Err(anyhow!("truncated name-list length"));
    }
    let len = u32::from_be_bytes([
        payload[*i],
        payload[*i + 1],
        payload[*i + 2],
        payload[*i + 3],
    ]) as usize;
    *i = len_end;
    let data_end = i
        .checked_add(len)
        .ok_or_else(|| anyhow!("name-list length overflow"))?;
    if payload.len() < data_end {
        return Err(anyhow!("truncated name-list"));
    }
    let s = core::str::from_utf8(&payload[*i..data_end])
        .map_err(|e| anyhow!("invalid name-list utf8: {e}"))?;
    *i = data_end;
    if s.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(s.split(',').map(|a| a.to_string()).collect())
    }
}

/// Parse a KEXINIT payload (RFC 4253 § 7.1) into the subset of algorithm
/// name-lists we surface in error messages. Expects the full payload
/// including the `SSH_MSG_KEXINIT` (20) message code and 16-byte cookie.
fn parse_kexinit(payload: &[u8]) -> Result<ServerAlgos> {
    if payload.len() < 17 || payload[0] != 20 {
        return Err(anyhow!(
            "expected SSH_MSG_KEXINIT (20), got code {}",
            payload.first().copied().unwrap_or(0)
        ));
    }
    // Skip msg_code(1) + cookie(16).
    let mut i = 17;
    let kex = read_namelist(payload, &mut i)?;
    let server_host_key = read_namelist(payload, &mut i)?;
    let cipher_c2s = read_namelist(payload, &mut i)?;
    let cipher_s2c = read_namelist(payload, &mut i)?;
    let mac_c2s = read_namelist(payload, &mut i)?;
    let mac_s2c = read_namelist(payload, &mut i)?;
    // Remaining fields (compression, languages, first_kex_follows, reserved)
    // are not needed by error reporting.
    Ok(ServerAlgos {
        kex,
        server_host_key,
        cipher_c2s,
        cipher_s2c,
        mac_c2s,
        mac_s2c,
    })
}

/// Connect to an SSH server, exchange the protocol version banner, and read
/// the first binary packet (expected to be `SSH_MSG_KEXINIT`) so we can
/// report the algorithms the server advertises. Used to enrich error
/// messages when russh aborts the handshake due to missing common algorithms.
///
/// This is best-effort: any failure is surfaced to the caller so it can fall
/// back to the plain error message.
async fn probe_server_kexinit(target: &str) -> Result<ServerAlgos> {
    let stream = tokio::time::timeout(
        std::time::Duration::from_secs(PROBE_TIMEOUT_SECS),
        TcpStream::connect(target),
    )
    .await
    .map_err(|_| anyhow!("timed out connecting to {target}"))??;
    let (mut rd, mut wr) = stream.into_split();

    // Read server version line. SSH-2.0 banners end with CRLF; servers MAY
    // send preface lines before the banner per RFC 4253 § 4.2.
    let mut got_banner = false;
    let mut preface_bytes = 0usize;
    for _ in 0..64 {
        let mut line = Vec::new();
        let mut one = [0u8; 1];
        loop {
            let n = rd.read(&mut one).await?;
            if n == 0 {
                return Err(anyhow!("connection closed before SSH banner"));
            }
            if one[0] == b'\n' {
                break;
            }
            if line.len() > 1024 {
                return Err(anyhow!("SSH banner line too long"));
            }
            line.push(one[0]);
        }
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        if line.starts_with(b"SSH-") {
            got_banner = true;
            break;
        }
        preface_bytes += line.len() + 1;
        if preface_bytes > 8192 {
            return Err(anyhow!("excessive preface data before SSH banner"));
        }
    }
    if !got_banner {
        return Err(anyhow!("did not receive SSH banner"));
    }

    // Send our banner so the server progresses to KEXINIT.
    wr.write_all(b"SSH-2.0-realm-probe\r\n").await?;

    // Read the first binary packet: uint32 packet_length, byte padding_length,
    // payload, padding, [mac]. No MAC is in play before key exchange
    // completes, so we stop after the padding.
    let mut header = [0u8; 5];
    rd.read_exact(&mut header).await?;
    let packet_len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let padding_len = header[4] as usize;
    if packet_len == 0 || packet_len > 65535 || padding_len + 1 > packet_len {
        return Err(anyhow!("invalid SSH packet framing"));
    }
    let payload_len = packet_len - padding_len - 1;
    let mut payload = vec![0u8; payload_len];
    rd.read_exact(&mut payload).await?;
    let mut padding = vec![0u8; padding_len];
    rd.read_exact(&mut padding).await?;

    parse_kexinit(&payload)
}

/// Join a name-list for display in error messages, falling back to a
/// placeholder when the server advertised none.
fn fmt_algos(algos: &[String]) -> String {
    if algos.is_empty() {
        "<none advertised>".to_string()
    } else {
        algos.join(", ")
    }
}

/// Convert an error returned by [`Session::connect`] into a human readable
/// message. When russh rejects the handshake because the client and server
/// share no common algorithm we probe the remote KEXINIT to include the
/// server's advertised list.
async fn describe_connect_error(err: &anyhow::Error, target: &str, principal: &str) -> String {
    if let Some(russh_err) = err.downcast_ref::<russh::Error>() {
        match russh_err {
            russh::Error::NoCommonKexAlgo => {
                return match probe_server_kexinit(target).await {
                    Ok(a) => format!(
                        "no common key exchange algorithm with {target}; server advertised kex_algorithms: [{}]",
                        fmt_algos(&a.kex)
                    ),
                    Err(pe) => format!(
                        "no common key exchange algorithm with {target} (probe for server algorithms failed: {pe})"
                    ),
                };
            }
            russh::Error::NoCommonKeyAlgo => {
                return match probe_server_kexinit(target).await {
                    Ok(a) => format!(
                        "no common host key algorithm with {target}; server advertised server_host_key_algorithms: [{}]",
                        fmt_algos(&a.server_host_key)
                    ),
                    Err(pe) => format!(
                        "no common host key algorithm with {target} (probe for server algorithms failed: {pe})"
                    ),
                };
            }
            russh::Error::NoCommonCipher => {
                return match probe_server_kexinit(target).await {
                    Ok(a) => format!(
                        "no common cipher with {target}; server advertised encryption_algorithms client->server: [{}], server->client: [{}]",
                        fmt_algos(&a.cipher_c2s),
                        fmt_algos(&a.cipher_s2c)
                    ),
                    Err(pe) => format!(
                        "no common cipher with {target} (probe for server algorithms failed: {pe})"
                    ),
                };
            }
            russh::Error::NoCommonMac => {
                return match probe_server_kexinit(target).await {
                    Ok(a) => format!(
                        "no common MAC algorithm with {target}; server advertised mac_algorithms client->server: [{}], server->client: [{}]",
                        fmt_algos(&a.mac_c2s),
                        fmt_algos(&a.mac_s2c)
                    ),
                    Err(pe) => format!(
                        "no common MAC algorithm with {target} (probe for server algorithms failed: {pe})"
                    ),
                };
            }
            russh::Error::Disconnect => {
                return format!(
                    "connection to {target} was closed by the remote server during SSH handshake/authentication as '{principal}'; this typically indicates the credentials were rejected or a server-side authentication limit was reached"
                );
            }
            russh::Error::HUP => {
                return format!(
                    "connection to {target} was closed by the remote server (HUP) during SSH handshake as '{principal}'"
                );
            }
            russh::Error::ConnectionTimeout => {
                return format!(
                    "SSH connection to {target} timed out during handshake as '{principal}'"
                );
            }
            russh::Error::IO(io_err) => {
                return format!("I/O error connecting to {target} as '{principal}': {io_err}");
            }
            _ => {}
        }
    }
    // Fall back to anyhow's message. Session::connect's `anyhow!` paths (e.g.
    // "password authentication rejected for user@host") already carry enough
    // context.
    format!("authentication failed for '{principal}' at {target}: {err}")
}

/// Convert an error returned by [`Session::call`] (post-auth command
/// execution) into a human readable message, unwrapping russh's terse
/// `Disconnected` variant into something actionable.
fn describe_exec_error(err: &anyhow::Error, target: &str, principal: &str) -> String {
    if let Some(russh_err) = err.downcast_ref::<russh::Error>() {
        if matches!(russh_err, russh::Error::Disconnect) {
            return format!(
                "command execution on {target} as '{principal}' failed: server closed the connection (Disconnected); some SSH servers report authentication success and then immediately drop the session when credentials are actually rejected"
            );
        }
        if matches!(russh_err, russh::Error::HUP) {
            return format!(
                "command execution on {target} as '{principal}' failed: remote side hung up (HUP)"
            );
        }
    }
    format!("command execution on {target} as '{principal}' failed: {err}")
}

#[allow(clippy::too_many_arguments)]
async fn attempt_deploy_with_credential(
    target: &str,
    cred: &Credential,
    cmd: &str,
    privesc_cmd: Option<&str>,
    payload: Option<&[u8]>,
    payload_dst: Option<&str>,
    timeout_secs: u64,
) -> DeployOutcome {
    let connect = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        Session::connect(
            cred.principal.clone(),
            Some(cred.password.clone()),
            None,
            None,
            target.to_string(),
        ),
    )
    .await;

    let mut ssh = match connect {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            return DeployOutcome {
                principal: cred.principal.clone(),
                status: "failed",
                stdout: String::new(),
                stderr: String::new(),
                error: describe_connect_error(&e, target, &cred.principal).await,
            };
        }
        Err(_) => {
            return DeployOutcome {
                principal: cred.principal.clone(),
                status: "failed",
                stdout: String::new(),
                stderr: String::new(),
                error: format!(
                    "connection to {target} as '{}' timed out after {timeout_secs}s",
                    cred.principal
                ),
            };
        }
    };

    // Optional payload copy.
    if let Some(bytes) = payload {
        let dst = resolve_payload_dst(payload_dst);
        if let Err(e) = ssh.copy_bytes(bytes, &dst).await {
            let _ = ssh.close().await;
            return DeployOutcome {
                principal: cred.principal.clone(),
                status: "failed",
                stdout: String::new(),
                stderr: String::new(),
                error: format!(
                    "failed to copy payload to {target}:{dst} as '{}': {e}",
                    cred.principal
                ),
            };
        }
        // Best-effort chmod so the payload is executable. Shell-quote the
        // destination to avoid metacharacter expansion by the remote shell.
        let quoted_dst = shell_quote(&dst);
        let _ = ssh.call(&format!("chmod +x {quoted_dst}")).await;
    }

    // Determine if we are root; if not and privesc is provided, run it first.
    let mut effective_cmd = cmd.to_string();
    if let Some(privesc) = privesc_cmd {
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

    match result {
        Ok(run) => DeployOutcome {
            principal: cred.principal.clone(),
            status: "success",
            stdout: run.output().unwrap_or_default(),
            stderr: run.error().unwrap_or_default(),
            error: String::new(),
        },
        Err(e) => DeployOutcome {
            principal: cred.principal.clone(),
            status: "failed",
            stdout: String::new(),
            stderr: String::new(),
            error: describe_exec_error(&e, target, &cred.principal),
        },
    }
}

/// Attempt every credential against a single target, retrying each
/// credential up to `retries` additional times on failure. Returns one
/// [`DeployOutcome`] per credential attempted (not per credential x retry):
/// only the final attempt's outcome is recorded. Iteration stops at the
/// first successful credential.
#[allow(clippy::too_many_arguments)]
async fn handle_deploy_host(
    target: String,
    credentials: Vec<Credential>,
    cmd: String,
    privesc_cmd: Option<String>,
    payload: Option<Vec<u8>>,
    payload_dst: Option<String>,
    timeout_secs: u64,
    retries: u32,
) -> Vec<DeployOutcome> {
    let attempts = retries.saturating_add(1);
    let mut outcomes: Vec<DeployOutcome> = Vec::new();

    for cred in &credentials {
        let mut final_outcome: Option<DeployOutcome> = None;
        for _ in 0..attempts {
            let outcome = attempt_deploy_with_credential(
                &target,
                cred,
                &cmd,
                privesc_cmd.as_deref(),
                payload.as_deref(),
                payload_dst.as_deref(),
                timeout_secs,
            )
            .await;
            let succeeded = outcome.status == "success";
            final_outcome = Some(outcome);
            if succeeded {
                break;
            }
        }
        let outcome = final_outcome.expect("attempts >= 1");
        let succeeded = outcome.status == "success";
        outcomes.push(outcome);
        if succeeded {
            // Stop trying further credentials for this host.
            break;
        }
    }

    outcomes
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
    runtime: &tokio::runtime::Handle,
    ips: Vec<String>,
    credentials: Vec<BTreeMap<String, Value>>,
    cmd: String,
    privesc_cmd: Option<String>,
    payload: Option<Vec<u8>>,
    payload_dst: Option<String>,
    timeout: Option<i64>,
    retries: Option<i64>,
    workers: Option<i64>,
) -> Result<Vec<BTreeMap<String, Value>>> {
    let creds = parse_credentials(credentials)?;
    let targets = expand_targets(ips)?;
    let timeout_secs = resolve_timeout_secs(timeout)?;
    let retry_count = resolve_retries(retries)?;
    let workers_limit = resolve_workers(workers)?;

    // 1 worker per target, up to the caller's `workers` ceiling. There is no
    // benefit to spawning more workers than there is work to do. We rely on
    // the caller-provided runtime (which is assumed to be multi-threaded) to
    // actually execute those workers in parallel across cores.
    let pool_size = workers_limit.min(targets.len()).max(1);

    let results = runtime.block_on(run_worker_pool(
        targets.clone(),
        creds,
        cmd,
        privesc_cmd,
        payload,
        payload_dst,
        timeout_secs,
        retry_count,
        pool_size,
    ));

    // Flatten per-target outcomes into the user-facing row format while
    // preserving the caller's input ordering of targets.
    let mut rows: Vec<BTreeMap<String, Value>> = Vec::new();
    for (target, outcomes) in targets.iter().zip(results.into_iter()) {
        for outcome in outcomes {
            rows.push(make_result(
                target,
                outcome.status,
                &outcome.principal,
                &outcome.stdout,
                &outcome.stderr,
                &outcome.error,
            ));
        }
    }
    Ok(rows)
}

/// A single unit of work dispatched to the worker pool: the target we are
/// deploying to, and the index into the caller-provided target list so the
/// coordinator can reassemble per-host outcomes in input order.
struct Job {
    index: usize,
    target: String,
}

/// Run the worker pool and return one `Vec<DeployOutcome>` per input target,
/// positioned at the same index as the target.
///
/// Workers follow a classic "share by communicating" pattern:
///   * The coordinator sends every [`Job`] onto a job channel and then
///     closes its sending end.
///   * `pool_size` worker tasks pull from the shared receiver (serialized by
///     a `tokio::sync::Mutex` around the `mpsc::Receiver`) until the channel
///     is drained.
///   * Each worker sends its `(index, outcomes)` pair onto a results
///     channel; the coordinator collects them and drops them into the
///     correct output slot.
///
/// No mutable state is shared between workers other than the short-lived
/// lock used to pull the next job from the receiver; all outcome data flows
/// through channels.
#[allow(clippy::too_many_arguments)]
async fn run_worker_pool(
    targets: Vec<String>,
    credentials: Vec<Credential>,
    cmd: String,
    privesc_cmd: Option<String>,
    payload: Option<Vec<u8>>,
    payload_dst: Option<String>,
    timeout_secs: u64,
    retries: u32,
    pool_size: usize,
) -> Vec<Vec<DeployOutcome>> {
    let total = targets.len();

    // Per-host outcomes, indexed by input position. Populated by the
    // coordinator as workers report back, so no locking is required.
    let mut outcomes_by_index: Vec<Vec<DeployOutcome>> = (0..total).map(|_| Vec::new()).collect();

    if total == 0 {
        return outcomes_by_index;
    }

    // Bounded job channel: capacity = total so the coordinator can hand off
    // every job without blocking even if no worker has yet picked any up.
    let (job_tx, job_rx) = mpsc::channel::<Job>(total);
    let job_rx = Arc::new(Mutex::new(job_rx));

    // Results channel carries (index, outcomes) so the coordinator can place
    // each worker's output into the correct slot regardless of completion
    // order.
    let (result_tx, mut result_rx) = mpsc::channel::<(usize, Vec<DeployOutcome>)>(total);

    // Immutable per-job inputs are wrapped in `Arc` so each worker can share
    // a read-only view without deep-cloning the full vectors (credentials,
    // payload bytes) on every job.
    let credentials = Arc::new(credentials);
    let cmd = Arc::new(cmd);
    let privesc_cmd = Arc::new(privesc_cmd);
    let payload = Arc::new(payload);
    let payload_dst = Arc::new(payload_dst);

    for _ in 0..pool_size {
        let rx = Arc::clone(&job_rx);
        let tx = result_tx.clone();
        let credentials = Arc::clone(&credentials);
        let cmd = Arc::clone(&cmd);
        let privesc_cmd = Arc::clone(&privesc_cmd);
        let payload = Arc::clone(&payload);
        let payload_dst = Arc::clone(&payload_dst);
        tokio::spawn(async move {
            loop {
                // Pop the next job. The lock is held only for the duration
                // of `recv` and released before the long-running SSH work.
                let job = {
                    let mut guard = rx.lock().await;
                    guard.recv().await
                };
                let Some(job) = job else {
                    // Channel closed and drained: no more work.
                    break;
                };
                let outcomes = handle_deploy_host(
                    job.target,
                    (*credentials).clone(),
                    (*cmd).clone(),
                    (*privesc_cmd).clone(),
                    (*payload).clone(),
                    (*payload_dst).clone(),
                    timeout_secs,
                    retries,
                )
                .await;
                if tx.send((job.index, outcomes)).await.is_err() {
                    // Coordinator went away; nothing more to do.
                    break;
                }
            }
        });
    }

    // Drop our result sender so the receiver sees `None` once every worker
    // has exited.
    drop(result_tx);

    // Dispatch all jobs. We clone target strings into each `Job` rather than
    // draining `targets`, because the caller needs the original ordering to
    // build the final row list.
    for (index, target) in targets.iter().enumerate() {
        // `send` only fails if every receiver has dropped, which would only
        // happen if every worker panicked. Treat that as a no-op; any
        // missing slots will simply have no rows in the final output.
        if job_tx
            .send(Job {
                index,
                target: target.clone(),
            })
            .await
            .is_err()
        {
            break;
        }
    }
    // Closing the job channel lets workers observe end-of-work and exit.
    drop(job_tx);

    // Collect results. Workers have already filled each slot exactly once.
    while let Some((index, outcomes)) = result_rx.recv().await {
        outcomes_by_index[index] = outcomes;
    }

    outcomes_by_index
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

    /// A multi-threaded tokio runtime shared across tests. Built once via
    /// `OnceLock` so we are never constructing a fresh runtime inside the
    /// `ssh_deploy` path under test.
    fn test_runtime_handle() -> tokio::runtime::Handle {
        use std::sync::OnceLock;
        static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
        RT.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("build test runtime")
        })
        .handle()
        .clone()
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
        assert_eq!(resolve_payload_dst(None), "/tmp/payload".to_string());
    }

    #[test]
    fn test_resolve_payload_dst_override() {
        assert_eq!(
            resolve_payload_dst(Some("/var/tmp/agent")),
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
            &test_runtime_handle(),
            vec![],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(res.is_err());

        let res = ssh_deploy(
            &test_runtime_handle(),
            vec!["127.0.0.1".into()],
            vec![],
            "echo hi".into(),
            None,
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
            &test_runtime_handle(),
            vec!["127.0.0.1".into()],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            Some(0),
            None,
            None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_ssh_deploy_invalid_retries() {
        let res = ssh_deploy(
            &test_runtime_handle(),
            vec!["127.0.0.1".into()],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            None,
            Some(-1),
            None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_workers_defaults() {
        assert_eq!(resolve_workers(None).unwrap(), DEFAULT_WORKERS);
        assert_eq!(resolve_workers(Some(1)).unwrap(), 1);
        assert_eq!(resolve_workers(Some(64)).unwrap(), 64);
    }

    #[test]
    fn test_resolve_workers_invalid() {
        assert!(resolve_workers(Some(0)).is_err());
        assert!(resolve_workers(Some(-1)).is_err());
    }

    #[test]
    fn test_ssh_deploy_invalid_workers() {
        let res = ssh_deploy(
            &test_runtime_handle(),
            vec!["127.0.0.1".into()],
            vec![cred("root", "pw")],
            "echo hi".into(),
            None,
            None,
            None,
            None,
            None,
            Some(0),
        );
        assert!(res.is_err());
    }

    /// Drive the worker pool against many targets with a tiny per-attempt
    /// timeout so every target "fails fast". This exercises the
    /// share-by-communicating plumbing (job channel, results channel,
    /// index-preserving collation) end-to-end without needing a real SSH
    /// server, and verifies that the pool returns one row per target in
    /// input order.
    #[test]
    fn test_ssh_deploy_worker_pool_preserves_order() {
        // Use RFC 5737 TEST-NET-1 addresses so connection attempts fail
        // immediately (no listener) rather than hanging on routable hosts.
        let ips: Vec<String> = (1..=20).map(|i| format!("192.0.2.{i}:22")).collect();
        let res = ssh_deploy(
            &test_runtime_handle(),
            ips.clone(),
            vec![cred("root", "pw")],
            "true".into(),
            None,
            None,
            None,
            Some(1), // 1s per connection timeout
            None,
            Some(8), // 8 workers; pool_size = min(8, 20) = 8
        )
        .expect("ssh_deploy should return per-host failures, not error out");
        assert_eq!(res.len(), ips.len());
        for (row, expected_ip) in res.iter().zip(ips.iter()) {
            assert_eq!(
                row.get("ip"),
                Some(&Value::String(expected_ip.clone())),
                "row out of order or missing: {row:?}"
            );
            assert_eq!(
                row.get("status"),
                Some(&Value::String("failed".to_string())),
                "row: {row:?}"
            );
        }
    }

    /// Manual/local integration test: run
    ///   `go run ./tests/e2e/utils/sshecho -p 2223`
    /// from the repo root, then execute
    ///   `cargo test -p eldritch-libpivot ssh_deploy_against_sshecho -- --ignored --nocapture`
    /// to verify that `pivot.ssh_deploy` can authenticate against the
    /// default-mode sshecho server and execute a command.
    #[test]
    #[ignore]
    fn ssh_deploy_against_sshecho() {
        let res = ssh_deploy(
            &test_runtime_handle(),
            vec!["127.0.0.1:2223".into()],
            vec![cred("root", "changeme")],
            "whoami".into(),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("ssh_deploy call failed");
        assert_eq!(res.len(), 1);
        let row = &res[0];
        assert_eq!(
            row.get("status"),
            Some(&Value::String("success".to_string())),
            "ssh_deploy row: {row:?}"
        );
        assert_eq!(
            row.get("stdout"),
            Some(&Value::String("root\n".to_string())),
            "ssh_deploy row: {row:?}"
        );
    }

    /// Manual/local integration test: run
    ///   `go run ./tests/e2e/utils/sshecho -p 2224 -u alice -pass secret`
    /// from the repo root, then execute
    ///   `cargo test -p eldritch-libpivot ssh_deploy_against_sshecho_bad_password \
    ///       -- --ignored --nocapture`
    /// to verify that a rejected password results in a clear
    /// authentication error (rather than the previous
    /// "Channel send error" symptom caused by silently accepting a
    /// failed auth handshake).
    #[test]
    #[ignore]
    fn ssh_deploy_against_sshecho_bad_password() {
        let res = ssh_deploy(
            &test_runtime_handle(),
            vec!["127.0.0.1:2224".into()],
            vec![cred("alice", "wrong")],
            "whoami".into(),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("ssh_deploy call should return per-host result, not panic");
        assert_eq!(res.len(), 1);
        let row = &res[0];
        assert_eq!(
            row.get("status"),
            Some(&Value::String("failed".to_string())),
            "ssh_deploy row: {row:?}"
        );
        // The principal that was attempted must be recorded on the failed row
        // so operators can tell which credential was rejected.
        assert_eq!(
            row.get("principal"),
            Some(&Value::String("alice".to_string())),
            "ssh_deploy row: {row:?}"
        );
        let error = match row.get("error") {
            Some(Value::String(s)) => s.clone(),
            other => panic!("unexpected error value: {other:?}"),
        };
        assert!(
            error.contains("auth"),
            "expected auth error, got: {error:?}"
        );
        assert!(
            !error.contains("Channel send error"),
            "should no longer surface as 'Channel send error', got: {error:?}"
        );
    }

    /// Build a minimal SSH_MSG_KEXINIT payload for round-trip testing of
    /// [`parse_kexinit`]. The payload layout follows RFC 4253 § 7.1:
    ///   byte SSH_MSG_KEXINIT (20)
    ///   byte[16] cookie
    ///   name-list kex_algorithms
    ///   name-list server_host_key_algorithms
    ///   name-list encryption_algorithms_client_to_server
    ///   name-list encryption_algorithms_server_to_client
    ///   name-list mac_algorithms_client_to_server
    ///   name-list mac_algorithms_server_to_client
    ///   name-list compression_algorithms_client_to_server
    ///   name-list compression_algorithms_server_to_client
    ///   name-list languages_client_to_server
    ///   name-list languages_server_to_client
    ///   boolean first_kex_packet_follows
    ///   uint32 reserved
    fn build_kexinit(
        kex: &str,
        host_key: &str,
        cipher_c2s: &str,
        cipher_s2c: &str,
        mac_c2s: &str,
        mac_s2c: &str,
    ) -> Vec<u8> {
        let mut p = Vec::new();
        p.push(20u8);
        p.extend_from_slice(&[0u8; 16]); // cookie
        let push_list = |p: &mut Vec<u8>, s: &str| {
            let b = s.as_bytes();
            p.extend_from_slice(&(b.len() as u32).to_be_bytes());
            p.extend_from_slice(b);
        };
        push_list(&mut p, kex);
        push_list(&mut p, host_key);
        push_list(&mut p, cipher_c2s);
        push_list(&mut p, cipher_s2c);
        push_list(&mut p, mac_c2s);
        push_list(&mut p, mac_s2c);
        push_list(&mut p, "none"); // compression c2s
        push_list(&mut p, "none"); // compression s2c
        push_list(&mut p, ""); // languages c2s
        push_list(&mut p, ""); // languages s2c
        p.push(0u8); // first_kex_packet_follows
        p.extend_from_slice(&[0u8; 4]); // reserved
        p
    }

    #[test]
    fn test_parse_kexinit_roundtrip() {
        let payload = build_kexinit(
            "curve25519-sha256,diffie-hellman-group14-sha256",
            "ssh-ed25519,rsa-sha2-256",
            "chacha20-poly1305@openssh.com,aes256-ctr",
            "chacha20-poly1305@openssh.com",
            "hmac-sha2-256",
            "hmac-sha2-512",
        );
        let algos = parse_kexinit(&payload).unwrap();
        assert_eq!(
            algos.kex,
            vec![
                "curve25519-sha256".to_string(),
                "diffie-hellman-group14-sha256".to_string()
            ]
        );
        assert_eq!(
            algos.server_host_key,
            vec!["ssh-ed25519".to_string(), "rsa-sha2-256".to_string()]
        );
        assert_eq!(
            algos.cipher_c2s,
            vec![
                "chacha20-poly1305@openssh.com".to_string(),
                "aes256-ctr".to_string()
            ]
        );
        assert_eq!(
            algos.cipher_s2c,
            vec!["chacha20-poly1305@openssh.com".to_string()]
        );
        assert_eq!(algos.mac_c2s, vec!["hmac-sha2-256".to_string()]);
        assert_eq!(algos.mac_s2c, vec!["hmac-sha2-512".to_string()]);
    }

    #[test]
    fn test_parse_kexinit_empty_list() {
        let payload = build_kexinit("", "ssh-ed25519", "aes256-ctr", "aes256-ctr", "", "");
        let algos = parse_kexinit(&payload).unwrap();
        assert!(algos.kex.is_empty());
        assert!(algos.mac_c2s.is_empty());
        assert!(algos.mac_s2c.is_empty());
        assert_eq!(algos.server_host_key, vec!["ssh-ed25519".to_string()]);
    }

    #[test]
    fn test_parse_kexinit_wrong_message_code() {
        // Anything other than 20 (SSH_MSG_KEXINIT) must be rejected so we
        // surface a probe failure instead of bogus algorithm lists.
        let mut payload = build_kexinit("a", "b", "c", "d", "e", "f");
        payload[0] = 21;
        assert!(parse_kexinit(&payload).is_err());
    }

    #[test]
    fn test_parse_kexinit_truncated() {
        // Only the message code and a partial cookie - parser must error
        // rather than panic on out-of-bounds indexing.
        let payload = vec![20u8, 0, 0, 0];
        assert!(parse_kexinit(&payload).is_err());
    }

    #[test]
    fn test_fmt_algos() {
        assert_eq!(fmt_algos(&[]), "<none advertised>");
        assert_eq!(
            fmt_algos(&["ssh-ed25519".to_string(), "rsa-sha2-256".to_string()]),
            "ssh-ed25519, rsa-sha2-256"
        );
    }
}
