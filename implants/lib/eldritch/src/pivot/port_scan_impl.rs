use super::super::insert_dict_kv;
use anyhow::{Context, Result};
use async_recursion::async_recursion;
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::dict::Dict;
use starlark::values::{Heap, Value};
use std::net::Ipv4Addr;
use tokio::net::{TcpStream, UdpSocket};
use tokio::task;
use tokio::time::{sleep, Duration};

macro_rules! scanf {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {{
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}

const TCP: &str = "tcp";
const UDP: &str = "upd";
const OPEN: &str = "open";
const CLOSED: &str = "closed";
const TIMEOUT: &str = "timeout";

// Convert a u32 IP representation into a string.
// Eg. 4294967295 -> "255.255.255.255"
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
        i = i + 1;
    }
    ip_vec.reverse();
    Ok(ip_vec.join("."))
}

// Transform a vector of u32 to u32 representation of IP.
// Eg. [255, 225, 255, 255] -> 4294967295
fn vec_to_int(ip_vec: Vec<u32>) -> Result<u32> {
    let mut res: u32 = 0;

    for (i, val) in ip_vec.iter().enumerate() {
        if i != 0 {
            res = res << 8;
        }
        res = res + val.clone();
    }
    Ok(res)
}

// Calculate the network and broadcast addresses given a CIDR string.
fn get_network_and_broadcast(target_cidr: String) -> Result<(Vec<u32>, Vec<u32>)> {
    // Transcribed from this python version: https://gist.github.com/vndmtrx/dc412e4d8481053ddef85c678f3323a6.
    // Ty! @vndmtrx.

    // Split on / to get host and cidr bits.
    let tmpvec: Vec<&str> = target_cidr.split("/").collect();
    let host = tmpvec.get(0).context("Index 0 not found")?.to_string();
    let bits: u32 = tmpvec
        .get(1)
        .context("Index 1 not found")?
        .parse::<u8>()?
        .try_into()?;

    // Define our vector representations.
    let mut addr: Vec<u64> = vec![0, 0, 0, 0];
    let mut mask: Vec<u64> = vec![0, 0, 0, 0];
    let mut bcas: Vec<u32> = vec![0, 0, 0, 0];
    let mut netw: Vec<u32> = vec![0, 0, 0, 0];

    let cidr: u64 = bits.try_into()?;

    let (octet_one, octet_two, octet_three, octet_four) = scanf!(host, ".", u64, u64, u64, u64);
    addr[3] = octet_four.context(format!("Failed to extract fourth octet {}", host))?;
    addr[2] = octet_three.context(format!("Failed to extract third octet {}", host))?;
    addr[1] = octet_two.context(format!("Failed to extract second octet {}", host))?;
    addr[0] = octet_one.context(format!("Failed to extract first octet {}", host))?;

    // Calculate netmask store as vector.
    let v: Vec<u64> = vec![24, 16, 8, 0];
    for (i, val) in v.iter().enumerate() {
        mask[i] = ((4294967295u64) << (32u64 - cidr) >> val) & 255u64;
    }

    // Calculate broadcast store as vector.
    let v2: Vec<usize> = vec![0, 1, 2, 3];
    for (i, val) in v2.iter().enumerate() {
        bcas[val.clone()] = ((addr[i] & mask[i]) | (255 ^ mask[i])) as u32;
    }

    // Calculate network address as vector.
    for (i, val) in v2.iter().enumerate() {
        netw[val.clone()] = (addr[i] & mask[i]) as u32;
    }

    // Return network address and broadcast address
    // we'll use these two to define or scan space.
    Ok((netw, bcas))
}

// Take a CIDR (192.168.1.1/24) and return a vector of the IPs possible within that CIDR.
fn parse_cidr(target_cidrs: Vec<String>) -> Result<Vec<String>> {
    let mut result: Vec<String> = vec![];
    for cidr in target_cidrs {
        let (netw, bcas): (Vec<u32>, Vec<u32>) = get_network_and_broadcast(cidr)?;
        let mut host_u32: u32 = vec_to_int(netw)?;
        let broadcast_u32: u32 = vec_to_int(bcas)?;

        // Handle /32 edge
        if host_u32 == broadcast_u32 {
            result.push(int_to_string(host_u32)?);
        }

        // Handle weird /31 cidr edge case
        if host_u32 == (broadcast_u32 - 1) {
            host_u32 = host_u32 + 1;
            result.push(int_to_string(host_u32)?);
        }

        // broadcast_u32-1 will not add the broadcast address for the net. Eg. 255.
        while host_u32 < (broadcast_u32 - 1) {
            // Skip network address Eg. 10.10.0.0
            host_u32 = host_u32 + 1;
            let host_ip_address = int_to_string(host_u32)?;
            if !result.contains(&host_ip_address) {
                result.push(host_ip_address);
            }
        }
    }

    Ok(result)
}

// Performs a TCP Connect scan. Connect to the remote port. If an error is thrown we know the port is closed.
// If this function timesout the port is filtered or host does not exist.
async fn tcp_connect_scan_socket(
    target_host: String,
    target_port: i32,
) -> Result<(String, i32, String, String)> {
    match TcpStream::connect(format!("{}:{}", target_host.clone(), target_port.clone())).await {
        Ok(_) => Ok((target_host, target_port, TCP.to_string(), OPEN.to_string())),
        Err(err) => {
            match err.to_string().as_str() {
                "Connection refused (os error 111)" if cfg!(target_os = "linux") => {
                    return Ok((target_host, target_port, TCP.to_string(), CLOSED.to_string()));
                },
                "No connection could be made because the target machine actively refused it. (os error 10061)" if cfg!(target_os = "windows") => {
                    return Ok((target_host, target_port, TCP.to_string(), CLOSED.to_string()));
                },
                "Connection refused (os error 61)" if cfg!(target_os = "macos") => {
                    return Ok((target_host, target_port, TCP.to_string(), CLOSED.to_string()));
                },
                "Connection reset by peer (os error 54)" if cfg!(target_os = "macos") => {
                    return Ok((target_host, target_port, TCP.to_string(), CLOSED.to_string()));
                },
                _ => {
                    return Err(anyhow::anyhow!("Unexpeceted error occured during scan:\n{}", err));
                },

            }
        },
    }
}

// Connect to a UDP port send the string hello and see if any data is sent back.
// If data is recieved port is open.
async fn udp_scan_socket(
    target_host: String,
    target_port: i32,
) -> Result<(String, i32, String, String)> {
    // Let the OS set our bind port.
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    // Send bytes to remote host.
    let _bytes_sent = sock
        .send_to(
            "hello".as_bytes(),
            format!("{}:{}", target_host.clone(), target_port.clone()),
        )
        .await?;

    // Recieve any response from remote host.
    let mut response_buffer = [0; 1024];
    // Handle the outcome of our recv.
    match sock.recv_from(&mut response_buffer).await {
        // If okay and we recieved bytes then we connected and the port  is open.
        Ok((bytes_copied, _addr)) => {
            if bytes_copied > 0 {
                return Ok((target_host, target_port, UDP.to_string(), OPEN.to_string()));
            } else {
                return Ok((target_host, target_port, UDP.to_string(), OPEN.to_string()));
            }
        }
        Err(err) => {
            match String::from(format!("{}", err.to_string())).as_str() {
                // Windows throws a weird error when scanning on localhost.
                // Considering the port closed.
                "An existing connection was forcibly closed by the remote host. (os error 10054)" if cfg!(target_os = "windows") => {
                    return Ok((target_host, target_port, UDP.to_string(), CLOSED.to_string()));
                },
                "Connection reset by peer (os error 54)" if cfg!(target_os = "macos") => {
                    return Ok((target_host, target_port, TCP.to_string(), CLOSED.to_string()));
                },
                _ => {
                    return Err(anyhow::anyhow!("Unexpeceted error occured during scan:\n{}", err));
                },
            }
        }
    }
    // UDP sockets are hard to coax.
    // If UDP doesn't respond to our hello message recv_from will hang and get timedout by `handle_port_scan_timeout`.
}

async fn handle_scan(
    target_host: String,
    port: i32,
    protocol: String,
) -> Result<(String, i32, String, String)> {
    let result: (String, i32, String, String);
    match protocol.as_str() {
        UDP => {
            match udp_scan_socket(target_host.clone(), port.clone()).await {
                Ok(res) => result = res,
                Err(err) => {
                    let err_str = String::from(format!("{}", err.to_string()));
                    match err_str.as_str() {
                        // If OS runs out source ports of raise a common error to `handle_port_scan_timeout`
                        // So a sleep can run and the port/host retried.
                        "Address already in use (os error 98)" if cfg!(target_os = "linux") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        "Too many open files (os error 24)" if cfg!(target_os = "macos") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        "An operation on a socket could not be performed because the system lacked sufficient buffer space or because a queue was full. (os error 10055)" if cfg!(target_os = "windows") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        _ => {
                            return  Err(anyhow::anyhow!(format!("{}:\n---\n{}\n---\n", "Unexpected error", err_str)));
                        },
                    }
                }
            }
        }
        TCP => {
            // TCP connect scan sucks but should work regardless of environment.
            match tcp_connect_scan_socket(target_host.clone(), port.clone()).await {
                Ok(res) => result = res,
                Err(err) => {
                    // let err_str = String::from(format!("{}", err.to_string())).as_str();
                    let err_str = String::from(format!("{}", err.to_string()));
                    match  err_str.as_str() {
                        // If OS runs out file handles of raise a common error to `handle_port_scan_timeout`
                        // So a sleep can run and the port/host retried.
                        "Too many open files (os error 24)" if cfg!(target_os = "linux") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        "Too many open files (os error 24)" if cfg!(target_os = "macos") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        // This appears to be how windows tells us it has run out of TCP sockets to bind.
                        "An attempt was made to access a socket in a way forbidden by its access permissions. (os error 10013)" if cfg!(target_os = "windows") => {
                           return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        // This is also be a way windows can tell us it has run out of TCP sockets to bind.
                        "An operation on a socket could not be performed because the system lacked sufficient buffer space or because a queue was full. (os error 10055)" if cfg!(target_os = "windows") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        _ => {
                            return  Err(anyhow::anyhow!(format!("{}:\n---\n{}\n---\n", "Unexpected error", err_str)));
                        },
                    }
                }
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "protocol not supported. Use 'udp' or 'tcp'."
            ))
        }
    }
    Ok(result)
}

// This needs to be split out so we can have the timeout error returned in the normal course of a thread running.
// This allows us to more easily manage our three states: timeout, open, closed.
#[async_recursion]
async fn handle_port_scan_timeout(
    target: String,
    port: i32,
    protocol: String,
    timeout: i32,
) -> Result<(String, i32, String, String)> {
    // Define our tokio timeout for when to kill the tcp connection.
    let timeout_duration = Duration::from_secs(timeout as u64);

    // Define our scan future.
    let scan = handle_scan(target.clone(), port.clone(), protocol.clone());

    // Execute that future with a timeout defined by the timeout argument.
    // open for connected to port, closed for rejected, timeout for tokio timeout expiring.
    match tokio::time::timeout(timeout_duration, scan).await {
        Ok(res) => {
            match res {
                Ok(scan_res) => return Ok(scan_res),
                Err(scan_err) => match String::from(format!("{}", scan_err.to_string())).as_str() {
                    // If the OS is running out of resources wait and then try again.
                    "Low resources try again" => {
                        sleep(Duration::from_secs(3)).await;
                        return Ok(handle_port_scan_timeout(target, port, protocol, timeout).await?);
                    }
                    _ => {
                        return Err(anyhow::Error::from(scan_err));
                    }
                },
            }
        }
        // If our timeout timer has expired set the port state to timeout and return.
        Err(_timer_elapsed) => {
            return Ok((
                target.clone(),
                port.clone(),
                protocol.clone(),
                TIMEOUT.to_string(),
            ))
        }
    }
}

// Async handler for port scanning.
async fn handle_port_scan(
    target_cidrs: Vec<String>,
    ports: Vec<i32>,
    protocol: String,
    timeout: i32,
) -> Result<Vec<(String, i32, String, String)>> {
    // This vector will hold the handles to our futures so we can retrieve the results when they finish.
    let mut all_scan_futures: Vec<_> = vec![];
    // Iterate over all IP addresses in the CIDR range.
    for target in parse_cidr(target_cidrs)? {
        // Iterate over all listed ports.
        for port in &ports {
            // Add scanning job to the queue.
            let scan_with_timeout =
                handle_port_scan_timeout(target.clone(), port.clone(), protocol.clone(), timeout);
            all_scan_futures.push(task::spawn(scan_with_timeout));
        }
    }

    let mut result: Vec<(String, i32, String, String)> = vec![];
    // Await results of each job.
    // We are not acting on scan results indepently so it's okay to loop through each and only return when all have finished.
    for task in all_scan_futures {
        match task.await? {
            Ok(res) => {
                result.push(res);
            }
            Err(err) => return Err(anyhow::anyhow!("Async task await failed:\n{}", err)),
        };
    }
    Ok(result)
}

// Output should follow the format:
// [
//     { ip: "127.0.0.1", port: 22, protocol: TCP, status: OPEN,  },
//     { ip: "127.0.0.1", port: 80, protocol: TCP, status: CLOSED }
// ]

// Non-async wrapper for our async scan.
pub fn port_scan(
    starlark_heap: &Heap,
    target_cidrs: Vec<String>,
    ports: Vec<i32>,
    protocol: String,
    timeout: i32,
) -> Result<Vec<Dict>> {
    if protocol != TCP && protocol != UDP {
        return Err(anyhow::anyhow!("Unsupported protocol. Use 'tcp' or 'udp'."));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let response = runtime.block_on(handle_port_scan(target_cidrs, ports, protocol, timeout));

    match response {
        Ok(results) => {
            let mut final_res: Vec<Dict> = Vec::new();
            for row in results {
                // Define underlying datastructure.
                let res: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut tmp_res = Dict::new(res);

                insert_dict_kv!(tmp_res, starlark_heap, "ip", row.0.as_str(), String);
                insert_dict_kv!(tmp_res, starlark_heap, "port", row.1, i32);
                insert_dict_kv!(tmp_res, starlark_heap, "protocol", row.2.as_str(), String);
                insert_dict_kv!(tmp_res, starlark_heap, "status", row.3.as_str(), String);

                final_res.push(tmp_res);
            }

            return Ok(final_res);
        }
        Err(err) => return Err(anyhow::anyhow!("The port_scan command failed: {:?}", err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::environment::GlobalsBuilder;
    use starlark::environment::Module;
    use starlark::eval::Evaluator;
    use starlark::starlark_module;
    use starlark::syntax::{AstModule, Dialect};
    use starlark::values::Value;
    use tokio::io::copy;
    use tokio::net::TcpListener;
    use tokio::task;

    #[tokio::test]
    async fn test_portscan_int_to_string() -> anyhow::Result<()> {
        let mut res1 = int_to_string(4294967295u32)?;
        assert_eq!(res1, "255.255.255.255");
        res1 = int_to_string(168427647u32)?;
        assert_eq!(res1, "10.10.0.127");
        res1 = int_to_string(2130706433u32)?;
        assert_eq!(res1, "127.0.0.1");
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_vec_to_int() -> anyhow::Result<()> {
        let mut res1 = vec_to_int(vec![127u32, 0u32, 0u32, 1u32])?;
        assert_eq!(res1, 2130706433u32);
        res1 = vec_to_int(vec![10u32, 10u32, 0u32, 127u32])?;
        assert_eq!(res1, 168427647u32);
        res1 = vec_to_int(vec![255u32, 255u32, 255u32, 255u32])?;
        assert_eq!(res1, 4294967295u32);
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_get_network_and_broadcast() -> anyhow::Result<()> {
        let mut res1 = get_network_and_broadcast("127.0.0.1/32".to_string())?;
        assert_eq!(
            res1,
            (
                vec![127u32, 0u32, 0u32, 1u32],
                vec![127u32, 0u32, 0u32, 1u32]
            )
        );
        res1 = get_network_and_broadcast("10.10.0.0/21".to_string())?;
        assert_eq!(
            res1,
            (
                vec![10u32, 10u32, 0u32, 0u32],
                vec![10u32, 10u32, 7u32, 255u32]
            )
        );
        res1 = get_network_and_broadcast("10.10.0.120/28".to_string())?;
        assert_eq!(
            res1,
            (
                vec![10u32, 10u32, 0u32, 112u32],
                vec![10u32, 10u32, 0u32, 127u32]
            )
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_cidrparse() -> anyhow::Result<()> {
        let mut res: Vec<String>;
        res = parse_cidr(vec!["127.0.0.1/32".to_string()])?;
        assert_eq!(res, vec!["127.0.0.1".to_string()]);

        res = parse_cidr(vec!["127.0.0.5/31".to_string()])?;
        assert_eq!(res, vec!["127.0.0.5".to_string()]);

        res = parse_cidr(vec![
            "127.0.0.1/32".to_string(),
            "127.0.0.2/32".to_string(),
            "127.0.0.2/31".to_string(),
        ])?;
        assert_eq!(
            res,
            vec![
                "127.0.0.1".to_string(),
                "127.0.0.2".to_string(),
                "127.0.0.3".to_string()
            ]
        );

        res = parse_cidr(vec![
            "10.10.0.102/29".to_string(),
            "192.168.0.1/30".to_string(),
        ])?;
        assert_eq!(
            res,
            vec![
                "10.10.0.97".to_string(),
                "10.10.0.98".to_string(),
                "10.10.0.99".to_string(),
                "10.10.0.100".to_string(),
                "10.10.0.101".to_string(),
                "10.10.0.102".to_string(),
                "192.168.0.1".to_string(),
                "192.168.0.2".to_string()
            ]
        );
        Ok(())
    }

    async fn local_bind_tcp() -> TcpListener {
        // Try three times to bind to a port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        return listener;
    }

    async fn local_accept_tcp(listener: TcpListener) -> Result<()> {
        // Accept new connection
        let (mut socket, _) = listener.accept().await?;
        // Split reader and writer references
        let (mut reader, mut writer) = socket.split();
        // Copy from reader to writer to echo message back.
        let bytes_copied = copy(&mut reader, &mut writer).await?;
        // If message sent break loop
        if bytes_copied > 1 {
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("Failed to copy any bytes"));
        }
    }

    #[tokio::test]
    async fn test_portscan_tcp() -> anyhow::Result<()> {
        // Allocate unused ports
        const NUMBER_OF_PORTS: u8 = 3;
        let mut bound_listeners_vec: Vec<TcpListener> = vec![];
        for _ in 0..(NUMBER_OF_PORTS) {
            bound_listeners_vec.push(local_bind_tcp().await);
        }

        let mut test_ports: Vec<i32> = vec![];
        // Iterate over append port number and start listen server
        let mut listen_tasks = vec![];
        for listener in bound_listeners_vec.into_iter() {
            test_ports.push(listener.local_addr()?.port().try_into()?);
            listen_tasks.push(task::spawn(local_accept_tcp(listener)));
        }

        let test_cidr = vec!["127.0.0.1/32".to_string()];

        // Setup a sender
        let send_task = task::spawn(handle_port_scan(
            test_cidr,
            test_ports.clone(),
            String::from(TCP),
            5,
        ));

        let mut listen_task_iter = listen_tasks.into_iter();

        // Run both
        let (_a, _b, _c, actual_response) = tokio::join!(
            listen_task_iter
                .next()
                .context("Failed to start listen task 1")?,
            listen_task_iter
                .next()
                .context("Failed to start listen task 1")?,
            listen_task_iter
                .next()
                .context("Failed to start listen task 1")?,
            send_task
        );

        let unwrapped_response = match actual_response {
            Ok(res) => match res {
                Ok(res_inner) => res_inner,
                Err(inner_error) => {
                    return Err(anyhow::anyhow!(
                        "error unwrapping scan result\n{}",
                        inner_error
                    ))
                }
            },
            Err(error) => return Err(anyhow::anyhow!("error unwrapping async result\n{}", error)),
        };

        let host = "127.0.0.1".to_string();
        let proto = TCP.to_string();
        let expected_response: Vec<(String, i32, String, String)>;
        expected_response = vec![
            (host.clone(), test_ports[0], proto.clone(), OPEN.to_string()),
            (host.clone(), test_ports[1], proto.clone(), OPEN.to_string()),
            (host.clone(), test_ports[2], proto.clone(), OPEN.to_string()),
        ];
        assert_eq!(expected_response, unwrapped_response);
        Ok(())
    }

    // #[tokio::test]
    // async fn test_portscan_udp() -> anyhow::Result<()> {
    //     let test_ports =  allocate_localhost_unused_ports(4, UDP.to_string()).await?;

    //     // Setup a test echo server
    //     let listen_task1 = task::spawn(
    //         setup_test_listener(String::from("127.0.0.1"),test_ports[0], String::from(UDP))
    //     );
    //     let listen_task2 = task::spawn(
    //         setup_test_listener(String::from("127.0.0.1"),test_ports[1], String::from(UDP))
    //     );
    //     let listen_task3 = task::spawn(
    //         setup_test_listener(String::from("127.0.0.1"),test_ports[2], String::from(UDP))
    //     );

    //     let test_cidr =  vec!["127.0.0.1/32".to_string()];

    //     // Setup a sender
    //     let send_task = task::spawn(
    //         handle_port_scan(test_cidr, test_ports.clone(), String::from(UDP), 5)
    //     );

    //     // Run both
    //     let (_a, _b, _c, actual_response) =
    //         tokio::join!(listen_task1,listen_task2,listen_task3,send_task);

    //     let host = "127.0.0.1".to_string();
    //     let proto = UDP.to_string();
    //     let expected_response: Vec<(String, i32, String, String)>;
    //     if cfg!(target_os = "windows") {
    //         expected_response = vec![(host.clone(),test_ports[0],proto.clone(),OPEN.to_string()),
    //             (host.clone(),test_ports[1],proto.clone(),OPEN.to_string()),
    //             (host.clone(),test_ports[2],proto.clone(),OPEN.to_string()),
    //             (host.clone(),test_ports[3],proto.clone(),CLOSED.to_string())];
    //     }else{
    //         expected_response = vec![(host.clone(),test_ports[0],proto.clone(),OPEN.to_string()),
    //             (host.clone(),test_ports[1],proto.clone(),OPEN.to_string()),
    //             (host.clone(),test_ports[2],proto.clone(),OPEN.to_string()),
    //             (host.clone(),test_ports[3],proto.clone(),TIMEOUT.to_string())];
    //     }

    //     assert_eq!(expected_response, actual_response??);
    //     Ok(())
    // }

    // // Test scanning a lot of ports all at once. Can the OS handle it.
    // // UDP scan is being very inconsitent seems to work every other scan.
    // #[tokio::test]
    // async fn test_portscan_udp_max() -> anyhow::Result<()> {
    //     let test_ports: Vec<i32> =  (1..65535).map(|x| x).collect();

    //     let test_cidr =  vec!["127.0.0.1/32".to_string()];

    //     let _scan_res = handle_port_scan(test_cidr, test_ports.clone(), String::from(UDP), 5).await?;

    //     Ok(())
    // }

    // Test scanning a lot of ports all at once. Can the OS handle it.
    // #[tokio::test]
    // async fn test_portscan_tcp_max() -> anyhow::Result<()>{
    //     if cfg!(target_os = "windows") { // Windows TCP max port scan doesn't work on localhost.
    //         let test_ports: Vec<i32> =  (1..65535).map(|x| x).collect();

    //         let test_cidr =  vec!["192.168.119.2/32".to_string()];

    //         let _scan_res = handle_port_scan(test_cidr, test_ports.clone(), String::from(TCP), 5).await?;
    //     }else {
    //         let test_ports: Vec<i32> =  (1..65535).map(|x| x).collect();

    //         let test_cidr =  vec!["127.0.0.1/32".to_string()];

    //         let _scan_res = handle_port_scan(test_cidr, test_ports.clone(), String::from(TCP), 5).await?;
    //     }
    //     Ok(())
    // }

    // verify our non async call works and Dict return type.
    #[test]
    fn test_portscan_return_type_starlark_dict_from_interpreter() -> anyhow::Result<()> {
        let test_ports: Vec<i32> = vec![8000, 8001, 8002, 8003, 8004];

        // Create test script
        let test_content = format!(
            r#"
ports_to_scan=[{},{},{},{}]
res = func_port_scan(ports_to_scan)
res
"#,
            test_ports[0], test_ports[1], test_ports[2], test_ports[3]
        );

        // Setup starlark interpreter with handle to our function
        let ast: AstModule;
        match AstModule::parse("test.eldritch", test_content.to_owned(), &Dialect::Extended) {
            Ok(res) => ast = res,
            Err(err) => return Err(err),
        }

        #[starlark_module]
        fn func_port_scan(builder: &mut GlobalsBuilder) {
            fn func_port_scan<'v>(
                ports: Vec<i32>,
                starlark_heap: &'v Heap,
            ) -> anyhow::Result<Vec<Dict<'v>>> {
                let test_cidr = vec!["127.0.0.1/32".to_string()];
                port_scan(starlark_heap, test_cidr, ports, TCP.to_string(), 3)
            }
        }

        let globals = GlobalsBuilder::standard().with(func_port_scan).build();
        let module: Module = Module::new();

        let mut eval: Evaluator = Evaluator::new(&module);
        let res: Value = eval.eval_module(ast, &globals)?;
        let _res_string = res.to_string();
        // Didn't panic yay!
        assert!(true);
        Ok(())
    }
}
