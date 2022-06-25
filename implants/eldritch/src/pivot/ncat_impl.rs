use std::net::Ipv4Addr;

use anyhow::Result;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpStream, UdpSocket};

// Since we cannot go from async (test) -> sync (ncat) `block_on` -> async (handle_ncat) without getting an error "cannot create runtime in current runtime since current thread is calling async code."
async fn handle_ncat(address: String, port: i32, data: String, protocol: String) -> Result<String> {
    // If the response is longer than 4096 bytes it will be  truncated.
    let mut response_buffer = [0; 4096];
    let result_string: String;

    let  address_and_port = format!("{}:{}", address, port);

    if protocol == "tcp" {
        // Connect to remote host
        let mut connection = TcpStream::connect(&address_and_port).await?;

        // Write our meessage
        connection.write_all(data.as_bytes()).await?;
        // Read server response
        let _bytes_read_count = connection.read(&mut response_buffer).await?;    

        // We  need to take a buffer of bytes, turn it into a String but that string has null bytes.
        // To remove the null bytes we're using trim_matches.
        result_string = String::from(String::from_utf8((&response_buffer).to_vec()).unwrap().trim_matches(char::from(0)));
        Ok(result_string)

    } else if protocol == "udp" {
        // Connect to remote host
    
        // Setting the bind address to unspecified should leave it up to the OS to decide.
        // https://stackoverflow.com/a/67084977
        let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;

        // Send bytes to remote host
        let _bytes_sent = sock.send_to(data.as_bytes(), address_and_port.clone()).await?;

        // Recieve any response from remote host
        let mut response_buffer = [0; 1024];
        let (_bytes_copied, _addr) = sock.recv_from(&mut response_buffer).await?;

        // We  need to take a buffer of bytes, turn it into a String but that string has null bytes.
        // To remove the null bytes we're using trim_matches.
        result_string = String::from(String::from_utf8((&response_buffer).to_vec()).unwrap().trim_matches(char::from(0)));
        Ok(result_string)

    } else {
        return Err(anyhow::anyhow!("Protocol not supported please use: udp or tcp."));
    }
}

// We do not want to make this async since it would require we make all of the starlark bindings async.
// Instead we have a handle_ncat function that we call with block_on
pub fn ncat(address: String, port: i32, data: String, protocol: String) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let response = runtime.block_on(
        handle_ncat(address, port, data, protocol)
    );

    match response {
        Ok(_) => Ok(String::from(response.unwrap())),
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
            while i < 5 {
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
            let sock = UdpSocket::bind(format!("{}:{}", address,  port)).await?;

            let mut buf = [0; 1024];
            while i < 5 {
                println!("Accepting connections");
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
    async fn test_ncat_send_tcp() -> anyhow::Result<()> {
        let test_port = allocate_localhost_unused_ports(1,"tcp".to_string()).await?[0];
        // Setup a test echo server
        let expected_response = String::from("Hello world!");
        let listen_task = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_port, String::from("tcp"))
        );

        // Setup a sender
        let send_task = task::spawn(
            handle_ncat(String::from("127.0.0.1"), test_port, expected_response.clone(), String::from("tcp"))
        );

        // Will this create a race condition where the sender sends before the listener starts?
        // Run both
        let (_a, actual_response) = tokio::join!(listen_task,send_task);


        // Verify our data
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }
    #[tokio::test]
    async fn test_ncat_send_udp() -> anyhow::Result<()> {
        let test_port = allocate_localhost_unused_ports(1,"udp".to_string()).await?[0];

        // Setup a test echo server
        let expected_response = String::from("Hello world!");
        let listen_task = task::spawn(
            setup_test_listener(String::from("127.0.0.1"),test_port, String::from("udp"))
        );

        // Setup a sender
        let send_task = task::spawn(
            handle_ncat(String::from("127.0.0.1"), test_port, expected_response.clone(), String::from("udp"))
        );

        // Will this create a race condition where the sender sends before the listener starts?
        // Run both
        let (_a, actual_response) = tokio::join!(listen_task,send_task);


        // Verify our data
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }
    #[test]
    fn test_ncat_not_handle() -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let response = runtime.block_on(
            allocate_localhost_unused_ports(1,"tcp".to_string())
        );

        let test_port = response.unwrap()[0];

        let result = ncat(String::from("127.0.0.1"), test_port, String::from("No one can hear me!"), String::from("tcp"));
        match result {
            Ok(res) => panic!("Connection failure expected: {:?}", res), // No valid connection should exist
            Err(err) => match String::from(format!("{:?}", err)).as_str() {
                "Connection refused (os error 111)" if cfg!(target_os = "linux") => assert!(true),
                "No connection could be made because the target machine actively refused it. (os error 10061)" if cfg!(target_os = "windows") => assert!(true),
                "Connection refused (os error 61)" if cfg!(target_os = "macos") => assert!(true),
                _ => panic!("Unhandled result {:?}", err)
            }
        }
        Ok(())
    }
}