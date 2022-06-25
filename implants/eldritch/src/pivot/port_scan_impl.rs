use std::net::Ipv4Addr;

use anyhow::Result;
use tokio::time::Duration;
use tokio::net::{TcpStream, UdpSocket};

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
    // we'll use these two to define oru scan space.
    Ok((netw,bcas))
}

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

async fn tcp_connect_scan_socket(target_host: String, target_port: i32) -> Result<String> {
    match TcpStream::connect(format!("{}:{}", target_host.clone(), target_port.clone())).await {
        Ok(_) => Ok(format!("{address},{port},{protocol},{status}", 
            address=target_host, port=target_port, protocol="tcp".to_string(), status="open".to_string())),
        Err(_) => Ok(format!("{address},{port},{protocol},{status}", 
        address=target_host, port=target_port, protocol="tcp".to_string(), status="closed".to_string())),
    }
}

async fn udp_scan_socket(target_host: String, target_port: i32) -> Result<String> {
    // Let the OS set our bind port.
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    
    // Send bytes to remote host.
    let _bytes_sent = sock.send_to("hello".as_bytes(), format!("{}:{}", target_host.clone(), target_port.clone())).await;
    
    // Recieve any response from remote host.
    let mut response_buffer = [0; 1024];
    let (bytes_copied, _addr) = sock.recv_from(&mut response_buffer).await?;
    
    // UDP sockets are hard to coax.
    // If UDP doesn't respond to our hello message recv_from will hang and timeout.
    if bytes_copied > 0 {
        return Ok(format!("{address},{port},{protocol},{status}", 
            address=target_host, port=target_port, protocol="udp".to_string(), status="open".to_string()));
    }
    Ok("Error".to_string())
}

async fn handle_scan(target_host: String, port: i32, protocol: String) -> Result<String> {
    let result: String;
    match protocol.as_str() {
        "udp" => {
            result = udp_scan_socket(target_host.clone(), port.clone()).await.unwrap();
        }
        "tcp" => {
            // TCP connect scan sucks but should work regardless of environment.
            result = tcp_connect_scan_socket(target_host.clone(), port.clone()).await.unwrap();
        },
        _ => return Err(anyhow::anyhow!("protocol not supported. Use udp or tcp.")),

    }
    return Ok(result);
}

// Async handler for port scanning.
async fn handle_port_scan(target_cidrs: Vec<String>, ports: Vec<i32>, protocol: String, timeout: i32) -> Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    // Define our tokio timeout for when to kill the tcp connection.
    let timeout = Duration::from_secs(timeout as u64);
    // Iterate over all IP addresses in the CIDR range.
    for target in parse_cidr(target_cidrs).unwrap() {
        // Iterate over all listed ports.
        for port in &ports {
            // Implement some kind of thread pool to scan multiple hosts at once.

            // Define our scan future.
            let scan_with_timeout = handle_scan(target.clone(), port.clone(), protocol.clone());
            // Execute that future with a timeout defined by the timeout argument.
            // open for connected to port, closed for rejected, timeout for tokio timeout expiring.
            match tokio::time::timeout(timeout, scan_with_timeout).await {
                Ok(res) => result.push(res.unwrap()),
                Err(_) => result.push(format!("{address},{port},{protocol},{status}", 
                address=target.clone(), port=port.clone(), protocol=protocol, status="timeout".to_string())),
            }
        }
    }
    Ok(result)
}

// Non-async wrapper for our async scan.
pub fn port_scan(target_cidrs: Vec<String>, ports: Vec<i32>, portocol: String, timeout: i32) -> Result<Vec<String>> {
    if portocol != "tcp" && portocol != "udp" {
        return Err(anyhow::anyhow!("Unsupported protocol. Use tcp or udp."))
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
            handle_port_scan(test_cidr, test_ports.clone(), String::from("tcp"), 1)
        );

        // Will this create a race condition where the sender sends before the listener starts?
        // Run both
        let (_a, _b, _c, actual_response) = 
            tokio::join!(listen_task1,listen_task2,listen_task3,send_task);

        let host = "127.0.0.1".to_string();
        let proto = "tcp".to_string();
        let expected_response: Vec<String> = vec![format!("{},{},{},open", host, test_ports[0], proto),
                format!("{},{},{},open", host, test_ports[1], proto),
                format!("{},{},{},open", host, test_ports[2], proto),
                format!("{},{},{},closed", host, test_ports[3], proto)];

        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_portscan_udp() -> anyhow::Result<()> {
        let test_ports =  allocate_localhost_unused_ports(4, "tcp".to_string()).await?;

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
        // let test_ports =  vec![65432, 65431,  65430,  9091];

        // Setup a sender
        let send_task = task::spawn(
            handle_port_scan(test_cidr, test_ports.clone(), String::from("udp"), 1)
        );

        // Will this create a race condition where the sender sends before the listener starts?
        // Run both
        let (_a, _b, _c, actual_response) = 
            tokio::join!(listen_task1,listen_task2,listen_task3,send_task);

        let host = "127.0.0.1".to_string();
        let proto = "udp".to_string();
        let expected_response: Vec<String> = vec![format!("{},{},{},open", host, test_ports[0], proto),
                format!("{},{},{},open", host, test_ports[1], proto),
                format!("{},{},{},open", host, test_ports[2], proto),
                format!("{},{},{},timeout", host, test_ports[3], proto)];
    
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }

    // verify our non async call works.
    #[test]
    fn test_portscan_not_handle() -> anyhow::Result<()> {
        let test_cidr =  vec!["127.0.0.1/32".to_string()];
        let test_ports =  vec![65432, 65431,  65430,  9091];
        
        let expected_response = vec!["127.0.0.1,65432,tcp,closed".to_string(), "127.0.0.1,65431,tcp,closed".to_string(),
        "127.0.0.1,65430,tcp,closed".to_string(), "127.0.0.1,9091,tcp,closed".to_string()];

        let result = port_scan(test_cidr, test_ports, String::from("tcp"), 1)?;
        assert_eq!(result, expected_response);
        Ok(())
    }
}