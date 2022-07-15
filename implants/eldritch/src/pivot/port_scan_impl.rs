use std::net::Ipv4Addr;

use anyhow::{Result};
use tokio::task;
use tokio::time::{Duration,sleep};
use tokio::net::{TcpStream, UdpSocket};
use async_recursion::async_recursion;


macro_rules! scanf {
    ( $string:expr, $sep:expr, $( $x:ty ),+ ) => {{
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}

// Convert a u32 IP representation into a string.
// Eg. 4294967295 -> "255.255.255.255"
fn int_to_string(ip_int: u32) -> Result<String> {
    let mut ip_vec: Vec<String> = vec!["".to_string(), "".to_string(), "".to_string(), "".to_string()];
    
    let mut i = 0;
    while i < 4 {
        ip_vec[i] = ((ip_int >> (i*8)) as u8).to_string();
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
    let host = tmpvec[0].clone().to_string();
    let bits: u32 = tmpvec[1].clone().parse::<u8>().unwrap().into();

    // Define our vector representations.
    let mut addr: Vec<u64> = vec![0,0,0,0];
    let mut mask: Vec<u64> = vec![0,0,0,0];
    let mut bcas: Vec<u32> = vec![0,0,0,0];
    let mut netw: Vec<u32> = vec![0,0,0,0];

    let cidr: u64 = bits.into();

    let (octet_one, octet_two, octet_three, octet_four) = scanf!(host, ".", u64, u64, u64, u64);
    addr[3] = octet_four.unwrap();
    addr[2] = octet_three.unwrap();
    addr[1] = octet_two.unwrap();
    addr[0] = octet_one.unwrap();

    // Calculate netmask store as vector.
    let v: Vec<u64> = vec![24, 16, 8, 0];
    for (i, val) in v.iter().enumerate() {
        mask[i] = ( (4294967295u64) << (32u64-cidr) >> val ) & 255u64;
    }

    // Calculate broadcast store as vector.
    let v2: Vec<usize> = vec![0, 1, 2, 3];
    for (i, val) in v2.iter().enumerate() {
        bcas[val.clone()] = ( (addr[i] & mask[i]) | (255^mask[i]) ) as u32;
    }

    // Calculate network address as vector.
    for (i, val) in v2.iter().enumerate() {
        netw[val.clone()] = (addr[i] & mask[i]) as u32;
    }

    // Return network address and broadcast address
    // we'll use these two to define or scan space.
    Ok((netw,bcas))
}

// Take a CIDR (192.168.1.1/24) and return a vector of the IPs possible within that CIDR.
fn parse_cidr(target_cidrs: Vec<String>) -> Result<Vec<String>> {
    let mut result: Vec<String> = vec![];
    for cidr in target_cidrs {
        let (netw,bcas): (Vec<u32>, Vec<u32>) = get_network_and_broadcast(cidr)?;
        let mut host_u32: u32 = vec_to_int(netw)?;
        let broadcast_u32: u32 = vec_to_int(bcas)?;

        // Handle /32 edge 
        if host_u32 == broadcast_u32 {
            result.push(int_to_string(host_u32).unwrap());
        }

        // Handle weird /31 cidr edge case
        if host_u32 == (broadcast_u32 - 1) {
            host_u32 = host_u32 + 1;
            result.push(int_to_string(host_u32).unwrap());
        }

        // boardcast_u32-1 will not add the broadcast address for the net. Eg. 255.
        while host_u32 < (broadcast_u32-1) {
            // Skip network address Eg. 10.10.0.0
            host_u32 = host_u32 + 1;
            let host_ip_address = int_to_string(host_u32).unwrap();
            if ! result.contains(&host_ip_address) {
                result.push(host_ip_address);
            }
        }
    }    

    Ok(result)
}

// Performs a TCP Connect scan. Connect to the remote port. If an error is thrown we know the port is closed.
// If this function timesout the port is filtered or host does not exist.
async fn tcp_connect_scan_socket(target_host: String, target_port: i32) -> Result<String> {
    match TcpStream::connect(format!("{}:{}", target_host.clone(), target_port.clone())).await {
        Ok(_) => Ok(format!("{address},{port},{protocol},{status}", 
            address=target_host, port=target_port, protocol="tcp".to_string(), status="open".to_string())),
        Err(err) => {
            match err.to_string().as_str() {
                "Connection refused (os error 111)" if cfg!(target_os = "linux") => {
                    return Ok(format!("{address},{port},{protocol},{status}", 
                        address=target_host, port=target_port, protocol="tcp".to_string(), status="closed".to_string()));
                },
                "No connection could be made because the target machine actively refused it. (os error 10061)" if cfg!(target_os = "windows") => {
                    return Ok(format!("{address},{port},{protocol},{status}", 
                        address=target_host, port=target_port, protocol="tcp".to_string(), status="closed".to_string()));
                },
                "Connection refused (os error 61)" if cfg!(target_os = "macos") => {
                    return Ok(format!("{address},{port},{protocol},{status}", 
                        address=target_host, port=target_port, protocol="tcp".to_string(), status="closed".to_string()));
                },
                _ => {
                    return Err(anyhow::anyhow!("Unexpected result. {:?}", err))
                },

            }
        },
    }
}

// Connect to a UDP port send the string hello and see if any data is sent back.
// If data is recieved port is open.
async fn udp_scan_socket(target_host: String, target_port: i32) -> Result<String> {
    // Let the OS set our bind port.
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    // Send bytes to remote host.
    let _bytes_sent = sock.send_to("hello".as_bytes(), format!("{}:{}", target_host.clone(), target_port.clone())).await?;

    // Recieve any response from remote host.
    let mut response_buffer = [0; 1024];
    // let (bytes_copied, _addr) = sock.recv_from(&mut response_buffer).await?;
    // Handle the outcome of our recv.
    match sock.recv_from(&mut response_buffer).await {
        // If okay and we recieved bytes then we connected and the port  is open.
        Ok((bytes_copied, _addr)) => {
            if bytes_copied > 0 {
                return Ok(format!("{address},{port},{protocol},{status}", 
                    address=target_host, port=target_port, protocol="udp".to_string(), status="open".to_string()));
            } else {
                return Err(anyhow::anyhow!("Recieved successfully but byte length was 0. Unexpected result."));
            }
        },
        Err(err) => {
            match String::from(format!("{}", err.to_string())).as_str() {
                // Windows throws a weird error when scanning on localhost.
                // Considering the port closed.
                "An existing connection was forcibly closed by the remote host. (os error 10054)" if cfg!(target_os = "windows") => {
                    return Ok(format!("{address},{port},{protocol},{status}", 
                        address=target_host, port=target_port, protocol="udp".to_string(), status="closed".to_string()))
                },
                _ => {
                    return  Err(anyhow::anyhow!(format!("{}:{:?}", "Unexpected error", err)))
                },
            }
        },
    }
    // UDP sockets are hard to coax.
    // If UDP doesn't respond to our hello message recv_from will hang and get timedout by `handle_port_scan_timeout`.
}

async fn handle_scan(target_host: String, port: i32, protocol: String) -> Result<String> {
    let result: String;
    match protocol.as_str() {
        "udp" => {
            match udp_scan_socket(target_host.clone(), port.clone()).await {
                Ok(res) => result = res,
                Err(err) => {
                    match String::from(format!("{}", err.to_string())).as_str() {
                        // If OS runs out source ports of raise a common error to `handle_port_scan_timeout`
                        // So a sleep can run and the port/host retried.
                        "Address already in use (os error 98)" if cfg!(target_os = "linux") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        _ => {
                            return  Err(anyhow::anyhow!(format!("{}:{:?}", "Unexpected error", err)));
                        },
                    }
                }
            }
        } 
        "tcp" => {
            // TCP connect scan sucks but should work regardless of environment.
            match tcp_connect_scan_socket(target_host.clone(), port.clone()).await {
                Ok(res) => result = res,
                Err(err) => {
                    // let err_str = String::from(format!("{}", err.to_string())).as_str();
                    let err_str = String::from(format!("{}", err.to_string()));
                    match  err_str.as_str() {
                        // If OS runs out file handles of raise a common error to `handle_port_scan_timeout`
                        // So a sleep can run and the port/host retried.
                        "Too many open files" if cfg!(target_os = "linux") => {
                            return Err(anyhow::anyhow!("Low resources try again"));
                        },
                        _ => {
                            return  Err(anyhow::anyhow!(format!("{}:\n---\n{}\n---\n", "Unexpected error", err_str)));
                        },
                    }
                }
            }

        },
        _ => return Err(anyhow::anyhow!("protocol not supported. Use 'udp' or 'tcp'.")),

    }
    Ok(result)
}

// This needs to be split out so we can have the timeout error returned in the normal course of a thread running.
// This allows us to more easily manage our three states: timeout, open, closed.
#[async_recursion]
async fn handle_port_scan_timeout(target: String, port: i32, protocol: String, timeout: i32) -> Result<String> {
        // Define our tokio timeout for when to kill the tcp connection.
    let timeout_duration = Duration::from_secs(timeout as u64);

    // Define our scan future.
    let scan = handle_scan(target.clone(), port.clone(), protocol.clone());

    // Execute that future with a timeout defined by the timeout argument.
    // open for connected to port, closed for rejected, timeout for tokio timeout expiring.
    match tokio::time::timeout(timeout_duration, scan ).await {
        Ok(res) => {
            match res {
                Ok(scan_res) => return Ok(scan_res),
                Err(scan_err) => match String::from(format!("{}", scan_err.to_string())).as_str() {
                    // If the OS is running out of resources wait and then try again.
                    "Low resources try again" => {
                        sleep(Duration::from_secs(1)).await;
                        return Ok(handle_port_scan_timeout(target, port, protocol, timeout).await.unwrap());
                    },
                    _ => {
                        return anyhow::private::Err(scan_err);
                    },
                },
            }
        },
        // If our timeout timer has expired set the port state to timeout and return.
        Err(_timer_elapsed) => {
            return Ok(format!("{address},{port},{protocol},{status}", 
                address=target.clone(), port=port.clone(), protocol=protocol, status="timeout".to_string()));
        },
    }
}

// Async handler for port scanning.
async fn handle_port_scan(target_cidrs: Vec<String>, ports: Vec<i32>, protocol: String, timeout: i32) -> Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    // This vector will hold the handles to our futures so we can retrieve the results when they finish.
    let mut all_scan_futures: Vec<_> = vec![];
    // Iterate over all IP addresses in the CIDR range.
    for target in parse_cidr(target_cidrs).unwrap() {
        // Iterate over all listed ports.
        for port in &ports {
            // Add scanning job to the queue.
           let scan_with_timeout = handle_port_scan_timeout(target.clone(), port.clone(), protocol.clone(), timeout);
            all_scan_futures.push(task::spawn(scan_with_timeout));

        }
    }

    // Await results of each job.
    // We are not acting on scan results indepently so it's okay to loop through each and only return when all have finished.
    for task in all_scan_futures {
        match task.await? {
            Ok(res) => result.push(res),
            Err(err) => return anyhow::private::Err(err),
        };
    }

    Ok(result)
}

// Non-async wrapper for our async scan.
pub fn port_scan(target_cidrs: Vec<String>, ports: Vec<i32>, portocol: String, timeout: i32) -> Result<Vec<String>> {
    if portocol != "tcp" && portocol != "udp" {
        return Err(anyhow::anyhow!("Unsupported protocol. Use 'tcp' or 'udp'."))
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let response = runtime.block_on(
        handle_port_scan(target_cidrs, ports, portocol, timeout)
    );

    match response {
        Ok(result) => Ok(result),
        Err(_) => return response,
    }
}


// For TCP tests we're able to determine if a port is: open, closed, or timeout (filtered).
// timeout no response was recieved which can be due to the port being firewalled off with
// the DROP verb or that the host doesn't exist.
// For UDP tests open and timeout are the only states we're reliably able to determine.
// When a port is closed the OS will send an ICMP reject to the requester, however, we're 
// unable to capture that response at this time. The exception to this is scanning windows
// on localhost where an OS error will be thrown and the port considered closed.
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::net::UdpSocket;
    use tokio::task;
    use tokio::io::copy;

    // Tests run concurrently so each test needs a unique port.
    async fn allocate_localhost_unused_ports(count: i32, protocol: String) -> anyhow::Result<Vec<i32>> {
        let mut i = 0;
        let mut res: Vec<i32> = vec![];
        while i < count {
            i = i + 1;
            if protocol == "tcp" {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                res.push(listener.local_addr().unwrap().port().into());
            } else if protocol == "udp" {
                let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
                res.push(listener.local_addr().unwrap().port().into());
            }
        }
        Ok(res)
    }

    // Create an echo server on the specificed port / protocol.
    async fn setup_test_listener(address: String, port: i32, protocol: String) -> anyhow::Result<()> {
        let mut i = 0;
        if protocol == "tcp" {
            let listener = TcpListener::bind(format!("{}:{}", address,  port)).await?;
            while i < 1 {
                // Accept new connection
                let (mut socket, _) = listener.accept().await?;
                // Split reader and writer references
                let (mut reader, mut writer) = socket.split();
                // Copy from reader to writer to echo message back.
                let bytes_copied = copy(&mut reader, &mut writer).await?;
                // If message sent break loop
                if bytes_copied > 1 {
                    break;
                }
                i = i + 1;
            }
        } else if protocol == "udp" {
            let mut buf = [0; 1024];
            let sock = UdpSocket::bind(format!("{}:{}", address,  port)).await?;
            while i < 1 {
                let (bytes_copied, addr) = sock.recv_from(&mut buf).await?;

                let bytes_copied = sock.send_to(&buf[..bytes_copied], addr).await?;
                
                if bytes_copied > 1 {
                    break;
                }
                i = i + 1;
            }
        } else {
            println!("Unrecognized protocol");
            panic!("Unrecognized protocol")
        }
        Ok(())    
    }

    #[tokio::test]
    async fn test_portscan_int_to_string() -> anyhow::Result<()> {
        let mut res1 = int_to_string(4294967295u32);
        assert_eq!(res1.unwrap(), "255.255.255.255");
        res1 = int_to_string(168427647u32);
        assert_eq!(res1.unwrap(), "10.10.0.127");
        res1 = int_to_string(2130706433u32);
        assert_eq!(res1.unwrap(), "127.0.0.1");        
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_vec_to_int() -> anyhow::Result<()> {
        let mut res1 = vec_to_int(vec![127u32,0u32,0u32,1u32]);
        assert_eq!(res1.unwrap(), 2130706433u32);
        res1 = vec_to_int(vec![10u32,10u32,0u32,127u32]);
        assert_eq!(res1.unwrap(), 168427647u32);
        res1 = vec_to_int(vec![255u32,255u32,255u32,255u32]);
        assert_eq!(res1.unwrap(), 4294967295u32);
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_get_network_and_broadcast() -> anyhow::Result<()> {
        let mut res1 = get_network_and_broadcast("127.0.0.1/32".to_string());
        assert_eq!(res1.unwrap(), (vec![127u32,0u32,0u32,1u32], vec![127u32,0u32,0u32,1u32]));
        res1 = get_network_and_broadcast("10.10.0.0/21".to_string());
        assert_eq!(res1.unwrap(), (vec![10u32,10u32,0u32,0u32], vec![10u32,10u32,7u32,255u32]));
        res1 = get_network_and_broadcast("10.10.0.120/28".to_string());
        assert_eq!(res1.unwrap(), (vec![10u32,10u32,0u32,112u32], vec![10u32,10u32,0u32,127u32]));
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_cidrparse() -> anyhow::Result<()> {
        let mut res: Vec<String>;
        res = parse_cidr(vec!["127.0.0.1/32".to_string()]).unwrap();
        assert_eq!(res, vec!["127.0.0.1".to_string()]);

        res = parse_cidr(vec!["127.0.0.5/31".to_string()]).unwrap();
        assert_eq!(res, vec!["127.0.0.5".to_string()]);

        res = parse_cidr(vec!["127.0.0.1/32".to_string(), "127.0.0.2/32".to_string(), "127.0.0.2/31".to_string()]).unwrap();
        assert_eq!(res, vec!["127.0.0.1".to_string(), "127.0.0.2".to_string(), "127.0.0.3".to_string()]);

        res = parse_cidr(vec!["10.10.0.102/29".to_string(), "192.168.0.1/30".to_string()]).unwrap();
        assert_eq!(res, vec!["10.10.0.97".to_string(), "10.10.0.98".to_string(), "10.10.0.99".to_string(), "10.10.0.100".to_string(), 
            "10.10.0.101".to_string(), "10.10.0.102".to_string(), "192.168.0.1".to_string(), "192.168.0.2".to_string()]);
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_tcp() -> anyhow::Result<()> {
        let test_ports =  allocate_localhost_unused_ports(4, "tcp".to_string()).await?;


        // Setup a test echo server
        let listen_task1 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_ports[0], String::from("tcp"))
        );
        let listen_task2 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_ports[1], String::from("tcp"))
        );
        let listen_task3 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_ports[2], String::from("tcp"))
        );

        let test_cidr =  vec!["127.0.0.1/32".to_string()];

        // Setup a sender
        let send_task = task::spawn(
            handle_port_scan(test_cidr, test_ports.clone(), String::from("tcp"), 5)
        );

        // Run both
        let (_a, _b, _c, actual_response) = 
            tokio::join!(listen_task1,listen_task2,listen_task3,send_task);

        let host = "127.0.0.1".to_string();
        let proto = "tcp".to_string();
        let expected_response: Vec<String>;
        expected_response = vec![format!("{},{},{},open", host, test_ports[0], proto),
                format!("{},{},{},open", host, test_ports[1], proto),
                format!("{},{},{},open", host, test_ports[2], proto),
                format!("{},{},{},closed", host, test_ports[3], proto)];
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_udp() -> anyhow::Result<()> {
        let test_ports =  allocate_localhost_unused_ports(4, "udp".to_string()).await?;

        // Setup a test echo server
        let listen_task1 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_ports[0], String::from("udp"))
        );
        let listen_task2 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_ports[1], String::from("udp"))
        );
        let listen_task3 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_ports[2], String::from("udp"))
        );

        let test_cidr =  vec!["127.0.0.1/32".to_string()];

        // Setup a sender
        let send_task = task::spawn(
            handle_port_scan(test_cidr, test_ports.clone(), String::from("udp"), 5)
        );

        // Run both
        let (_a, _b, _c, actual_response) = 
            tokio::join!(listen_task1,listen_task2,listen_task3,send_task);

        let host = "127.0.0.1".to_string();
        let proto = "udp".to_string();
        let expected_response: Vec<String>;
        if cfg!(target_os = "windows") {
            expected_response = vec![format!("{},{},{},open", host, test_ports[0], proto),
                format!("{},{},{},open", host, test_ports[1], proto),
                format!("{},{},{},open", host, test_ports[2], proto),
                format!("{},{},{},closed", host, test_ports[3], proto)];
        }else{
            expected_response = vec![format!("{},{},{},open", host, test_ports[0], proto),
                format!("{},{},{},open", host, test_ports[1], proto),
                format!("{},{},{},open", host, test_ports[2], proto),
                format!("{},{},{},timeout", host, test_ports[3], proto)];
        }

        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }

    // verify our non async call works.
    #[test]
    fn test_portscan_not_handle() -> anyhow::Result<()> {
        let test_cidr =  vec!["127.0.0.1/32".to_string()];
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let response = runtime.block_on(
            allocate_localhost_unused_ports(4,"tcp".to_string())
        );

        let test_ports = response.unwrap();
        
        let host = "127.0.0.1".to_string();
        let proto = "tcp".to_string();
        let expected_response: Vec<String>;
        // We have no test listeners and are just testing that our sync -> async call works.
        expected_response = vec![format!("{},{},{},closed", host, test_ports[0], proto),
            format!("{},{},{},closed", host, test_ports[1], proto),
            format!("{},{},{},closed", host, test_ports[2], proto),
            format!("{},{},{},closed", host, test_ports[3], proto)];

        let result = port_scan(test_cidr, test_ports, String::from("tcp"), 5)?;
        assert_eq!(result, expected_response);
        Ok(())
    }

    // Test scanning a lot of ports all at once. Can the OS handle it.
    // UDP scan is being very inconsitent seems to work every other scan.
    // #[tokio::test]
    // async fn test_portscan_udp_max() -> anyhow::Result<()> {
    //     let test_ports: Vec<i32> =  (1..65535).map(|x| x).collect();
    //     let open_ports =  allocate_localhost_unused_ports(1, "udp".to_string()).await?;
    //     let listen_task1 = task::spawn(
    //         setup_test_listener(String::from("127.0.0.1"),open_ports[0], String::from("udp"))
    //     );

    //     let test_cidr =  vec!["127.0.0.1/32".to_string()];

    //     // Setup a sender
    //     let send_task = task::spawn(
    //         handle_port_scan(test_cidr, test_ports.clone(), String::from("udp"), 5)
    //     );
    //     // let scan_res = handle_port_scan(test_cidr, test_ports.clone(), String::from("udp"), 20).await?;
    //     let (_a, actual_response) = 
    //         tokio::join!(listen_task1,send_task);
        
    //     let expected_res = format!("127.0.0.1,{},udp,open", open_ports[0]);
    //     let unwrapped_res = actual_response.unwrap().unwrap();
    //     assert!(unwrapped_res.contains(  &expected_res  ));
    //     Ok(())
    // }
    
    // Test scanning a lot of ports all at once. Can the OS handle it.
    #[tokio::test]
    async fn test_portscan_tcp_max() -> anyhow::Result<()>{
        let test_ports: Vec<i32> =  (1..65535).map(|x| x).collect(); //vec![22, 53, 111, 65432, 4444, 123, 1234];
        let open_ports =  allocate_localhost_unused_ports(1, "tcp".to_string()).await?;
        let listen_task1 = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),open_ports[0], String::from("tcp"))
        );
 
        let test_cidr =  vec!["127.0.0.1/32".to_string()];

        // Setup a sender
        let send_task = task::spawn(
            handle_port_scan(test_cidr, test_ports.clone(), String::from("tcp"), 5)
        );
        // let scan_res = handle_port_scan(test_cidr, test_ports.clone(), String::from("udp"), 20).await?;
        let (_a, actual_response) = 
            tokio::join!(listen_task1,send_task);
        
        let expected_res = format!("127.0.0.1,{},tcp,open", open_ports[0]);
        let unwrapped_res = actual_response.unwrap().unwrap();
        assert!(unwrapped_res.contains(  &expected_res  ));
        Ok(())
    }

}
