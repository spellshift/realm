use anyhow::Result;

pub fn ncat(_address: String, _port: i32, _data: String, _protocol: String, _timeout: i32) -> Result<String> {
    unimplemented!("Method unimplemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::net::TcpListener;
    use tokio::net::TcpStream;
    use tokio::task;
    use tokio::io::copy;


    async fn process_socket(mut socket: TcpStream) {
        // do work with socket here
        let (mut reader, mut writer) = socket.split();
        let _bytes_copied = copy(&mut reader, &mut writer);
    }    

    async fn setup_test_listener(address: String, port: i32, protocol: String, _timeout: i32) -> anyhow::Result<()> {
        if protocol == "tcp" {
            let listener = TcpListener::bind(format!("{}:{}", address,  port)).await?;
            let mut i = 0;

            while i < 100 {
                println!("Accepting connections");
                let (socket, _) = listener.accept().await?;
                process_socket(socket).await;
                i = i + 1;
            }
            
    
        } else if protocol == "udp" {

        } else {
            panic!("Unrecognized protocol")
        }
        Ok(())    
    }

    #[tokio::test]
    async fn test_ncat_send_tcp() -> anyhow::Result<()> {
        // Setup a test echo server
        let expected_response = String::from("Hello world!");
        println!("Starting listener");
        let listen_task = task::spawn(setup_test_listener(String::from("127.0.0.1"),
            65432, String::from("tcp"), 3));

        // Send data
        let actual_response = ncat(String::from("127.0.0.1"),
            65432, expected_response.clone(), String::from("tcp"), 3)?;
        println!("{}", actual_response);
        // Verify our data
        let _a = tokio::join!(listen_task);
        // assert_eq!(expected_response, actual_response);
        Ok(())
    }
    #[test]
    fn test_ncat_send_udp() -> anyhow::Result<()> {
        Ok(())
    }
    #[test]
    fn test_ncat_failure_timeout() -> anyhow::Result<()> {
        Ok(())
    }
}