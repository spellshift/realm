use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{Context, Result};
use async_recursion::async_recursion;
use eldritch_core::Value;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::{Duration, sleep};

use crate::std::StdPivotLibrary;

macro_rules! scanf {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {{
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}

const TCP: &str = "tcp";
const UDP: &str = "udp";
const OPEN: &str = "open";
const CLOSED: &str = "closed";
const TIMEOUT: &str = "timeout";

// ... helper functions omitted for brevity, they are same as original ...
// I need to include them or it won't compile. I'll copy them back.

fn int_to_string(ip_int: u32) -> Result<String> {
    let mut ip_vec: Vec<String> = vec![
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    ];

    let mut i = 0;
    while i < 4 {
        ip_vec[i] = ((ip_int >> (i * 8)) as u8).to_string();
        i += 1;
    }
    ip_vec.reverse();
    Ok(ip_vec.join("."))
}

fn vec_to_int(ip_vec: Vec<u32>) -> Result<u32> {
    let mut res: u32 = 0;

    for (i, val) in ip_vec.iter().enumerate() {
        if i != 0 {
            res <<= 8;
        }
        res += *val;
    }
    Ok(res)
}

fn get_network_and_broadcast(target_cidr: String) -> Result<(Vec<u32>, Vec<u32>)> {
    let tmpvec: Vec<&str> = target_cidr.split('/').collect();
    let host = tmpvec.first().context("Index 0 not found")?.to_string();
    let bits: u32 = tmpvec.get(1).context("Index 1 not found")?.parse::<u8>()? as u32;

    let mut addr: Vec<u64> = vec![0, 0, 0, 0];
    let mut mask: Vec<u64> = vec![0, 0, 0, 0];
    let mut bcas: Vec<u32> = vec![0, 0, 0, 0];
    let mut netw: Vec<u32> = vec![0, 0, 0, 0];

    let cidr: u64 = bits as u64;

    let (octet_one, octet_two, octet_three, octet_four) = scanf!(host, ".", u64, u64, u64, u64);
    addr[3] = octet_four.context(format!("Failed to extract fourth octet {host}"))?;
    addr[2] = octet_three.context(format!("Failed to extract third octet {host}"))?;
    addr[1] = octet_two.context(format!("Failed to extract second octet {host}"))?;
    addr[0] = octet_one.context(format!("Failed to extract first octet {host}"))?;

    let v: Vec<u64> = vec![24, 16, 8, 0];
    for (i, val) in v.iter().enumerate() {
        mask[i] = ((4294967295u64) << (32u64 - cidr) >> val) & 255u64;
    }

    let v2: Vec<usize> = vec![0, 1, 2, 3];
    for (i, val) in v2.iter().enumerate() {
        bcas[*val] = ((addr[i] & mask[i]) | (255 ^ mask[i])) as u32;
    }

    for (i, val) in v2.iter().enumerate() {
        netw[*val] = (addr[i] & mask[i]) as u32;
    }

    Ok((netw, bcas))
}

fn parse_cidr(target_cidrs: Vec<String>) -> Result<Vec<String>> {
    let mut result: Vec<String> = vec![];
    for cidr in target_cidrs {
        let (netw, bcas): (Vec<u32>, Vec<u32>) = get_network_and_broadcast(cidr)?;
        let mut host_u32: u32 = vec_to_int(netw)?;
        let broadcast_u32: u32 = vec_to_int(bcas)?;

        if host_u32 == broadcast_u32 {
            result.push(int_to_string(host_u32)?);
        }

        if host_u32 == (broadcast_u32 - 1) {
            host_u32 += 1;
            result.push(int_to_string(host_u32)?);
        }

        while host_u32 < (broadcast_u32 - 1) {
            host_u32 += 1;
            let host_ip_address = int_to_string(host_u32)?;
            if !result.contains(&host_ip_address) {
                result.push(host_ip_address);
            }
        }
    }

    Ok(result)
}

async fn tcp_connect_scan_socket(
    target_host: String,
    target_port: i32,
) -> Result<(String, i32, String, String)> {
    match TcpStream::connect(format!("{}:{}", target_host.clone(), target_port.clone())).await {
        Ok(_) => Ok((target_host, target_port, TCP.to_string(), OPEN.to_string())),
        Err(err) => match err.to_string().as_str() {
            "Connection refused (os error 111)" if cfg!(target_os = "linux") => Ok((
                target_host,
                target_port,
                TCP.to_string(),
                CLOSED.to_string(),
            )),
            "No connection could be made because the target machine actively refused it. (os error 10061)"
                if cfg!(target_os = "windows") =>
            {
                Ok((
                    target_host,
                    target_port,
                    TCP.to_string(),
                    CLOSED.to_string(),
                ))
            }
            "Connection refused (os error 61)" if cfg!(target_os = "macos") => Ok((
                target_host,
                target_port,
                TCP.to_string(),
                CLOSED.to_string(),
            )),
            "Connection reset by peer (os error 54)" if cfg!(target_os = "macos") => Ok((
                target_host,
                target_port,
                TCP.to_string(),
                CLOSED.to_string(),
            )),
            "Host is unreachable (os error 113)" if cfg!(target_os = "linux") => Ok((
                target_host,
                target_port,
                TCP.to_string(),
                CLOSED.to_string(),
            )),
            _ => Err(anyhow::anyhow!(
                "Unexpeceted error occured during scan:\n{err}"
            )),
        },
    }
}

async fn udp_scan_socket(
    target_host: String,
    target_port: i32,
) -> Result<(String, i32, String, String)> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let _bytes_sent = sock
        .send_to(
            "hello".as_bytes(),
            format!("{}:{}", target_host.clone(), target_port.clone()),
        )
        .await?;

    let mut response_buffer = [0; 1024];
    match sock.recv_from(&mut response_buffer).await {
        Ok((_bytes_copied, _addr)) => {
            Ok((target_host, target_port, UDP.to_string(), OPEN.to_string()))
        }
        Err(err) => {
            match err.to_string().as_str() {
                "An existing connection was forcibly closed by the remote host. (os error 10054)"
                    if cfg!(target_os = "windows") =>
                {
                    Ok((
                        target_host,
                        target_port,
                        UDP.to_string(),
                        CLOSED.to_string(),
                    ))
                }
                "Connection reset by peer (os error 54)" if cfg!(target_os = "macos") => Ok((
                    target_host,
                    target_port,
                    TCP.to_string(),
                    CLOSED.to_string(),
                )),
                "Host is unreachable (os error 113)" if cfg!(target_os = "linux") => Ok((
                    target_host,
                    target_port,
                    TCP.to_string(),
                    CLOSED.to_string(),
                )),
                _ => Err(anyhow::anyhow!(
                    "Unexpeceted error occured during scan:\n{err}"
                )),
            }
        }
    }
}

async fn handle_scan(
    target_host: String,
    port: i32,
    protocol: String,
) -> Result<(String, i32, String, String)> {
    let result: (String, i32, String, String);
    match protocol.as_str() {
        UDP => {
            match udp_scan_socket(target_host.clone(), port).await {
                Ok(res) => result = res,
                Err(err) => {
                    let err_str = err.to_string();
                    match err_str.as_str() {
                        "Address already in use (os error 98)" if cfg!(target_os = "linux") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        "Too many open files (os error 24)" if cfg!(target_os = "macos") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        "An operation on a socket could not be performed because the system lacked sufficient buffer space or because a queue was full. (os error 10055)"
                            if cfg!(target_os = "windows") =>
                        {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        _ => {
                            return Err(anyhow::anyhow!(format!(
                                "{}:\n---\n{}\n---\n",
                                "Unexpected error", err_str
                            )));
                        }
                    }
                }
            }
        }
        TCP => {
            match tcp_connect_scan_socket(target_host.clone(), port).await {
                Ok(res) => result = res,
                Err(err) => {
                    let err_str = err.to_string();
                    match err_str.as_str() {
                        "Too many open files (os error 24)" if cfg!(target_os = "linux") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        "Too many open files (os error 24)" if cfg!(target_os = "macos") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        "An attempt was made to access a socket in a way forbidden by its access permissions. (os error 10013)"
                            if cfg!(target_os = "windows") =>
                        {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        "An operation on a socket could not be performed because the system lacked sufficient buffer space or because a queue was full. (os error 10055)"
                            if cfg!(target_os = "windows") =>
                        {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        }
                        _ => {
                            return Err(anyhow::anyhow!(format!(
                                "{}:\n---\n{}\n---\n",
                                "Unexpected error", err_str
                            )));
                        }
                    }
                }
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "protocol not supported. Use 'udp' or 'tcp'."
            ));
        }
    }
    Ok(result)
}

#[async_recursion]
async fn handle_port_scan_timeout(
    target: String,
    port: i32,
    protocol: String,
    timeout: i32,
) -> Result<(String, i32, String, String)> {
    let timeout_duration = Duration::from_secs(timeout as u64);
    let scan = handle_scan(target.clone(), port, protocol.clone());

    match tokio::time::timeout(timeout_duration, scan).await {
        Ok(res) => {
            match res {
                Ok(scan_res) => return Ok(scan_res),
                Err(scan_err) => match scan_err.to_string().as_str() {
                    "Low resources try again" => {
                        sleep(Duration::from_secs(3)).await;
                        return handle_port_scan_timeout(target, port, protocol, timeout).await;
                    }
                    _ => {
                        return Err(scan_err);
                    }
                },
            }
        }
        Err(_timer_elapsed) => {
            return Ok((target.clone(), port, protocol.clone(), TIMEOUT.to_string()));
        }
    }
}

async fn handle_port_scan(
    target_cidrs: Vec<String>,
    ports: Vec<i32>,
    protocol: String,
    timeout: i32,
    fd_limit: usize,
) -> Result<Vec<(String, i32, String, String)>> {
    let semaphore = Arc::new(Semaphore::new(fd_limit));
    let mut all_scan_futures: Vec<_> = vec![];
    for target in parse_cidr(target_cidrs)? {
        for port in &ports {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let target_clone = target.clone();
            let protocol_clone = protocol.clone();
            let port_val = *port;

            let scan_with_timeout = async move {
                let _permit = permit;
                handle_port_scan_timeout(target_clone, port_val, protocol_clone, timeout).await
            };
            all_scan_futures.push(task::spawn(scan_with_timeout));
        }
    }

    let mut result: Vec<(String, i32, String, String)> = vec![];
    for task in all_scan_futures {
        match task.await? {
            Ok(res) => {
                result.push(res);
            }
            Err(err) => return Err(anyhow::anyhow!("Async task await failed:\n{err}")),
        };
    }
    Ok(result)
}

pub fn run(
    lib: &StdPivotLibrary,
    target_cidrs: Vec<String>,
    ports: Vec<i64>,
    protocol: String,
    timeout: i64,
    fd_limit: Option<i64>,
) -> Result<Vec<BTreeMap<String, Value>>, String> {
    if protocol != TCP && protocol != UDP {
        return Err("Unsupported protocol. Use 'tcp' or 'udp'.".to_string());
    }

    let (tx, rx) = std::sync::mpsc::channel();

    let target_cidrs_clone = target_cidrs.clone();
    let ports_i32: Vec<i32> = ports.iter().map(|&p| p as i32).collect();
    let protocol_clone = protocol.clone();
    let limit = fd_limit.unwrap_or(64) as usize;
    let timeout_i32 = timeout as i32;

    let fut = async move {
        let res = handle_port_scan(
            target_cidrs_clone,
            ports_i32,
            protocol_clone,
            timeout_i32,
            limit,
        ).await;

        let _ = tx.send(res);
    };

    lib.agent
        .spawn_subtask(lib.task_id, "port_scan".to_string(), alloc::boxed::Box::pin(fut))
        .map_err(|e| e.to_string())?;

    let response = rx.recv().map_err(|e| format!("Failed to receive result: {}", e))?;

    match response {
        Ok(results) => {
            let mut final_res = Vec::new();
            for row in results {
                let mut tmp_res = BTreeMap::new();
                tmp_res.insert("ip".into(), Value::String(row.0));
                tmp_res.insert("port".into(), Value::Int(row.1 as i64));
                tmp_res.insert("protocol".into(), Value::String(row.2));
                tmp_res.insert("status".into(), Value::String(row.3));

                final_res.push(tmp_res);
            }

            Ok(final_res)
        }
        Err(err) => Err(format!("The port_scan command failed: {:?}", err)),
    }
}
