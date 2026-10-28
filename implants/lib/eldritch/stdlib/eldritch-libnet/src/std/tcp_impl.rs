use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

async fn handle_tcp(address: String, port: i32, data: String) -> Result<String> {
    let mut response_buffer: Vec<u8> = Vec::new();
    let address_and_port = format!("{address}:{port}");

    let mut connection = TcpStream::connect(&address_and_port).await?;
    connection.write_all(data.as_bytes()).await?;

    let mut read_stream = BufReader::new(connection);
    read_stream.read_buf(&mut response_buffer).await?;

    let result_string = String::from(
        String::from_utf8_lossy(&response_buffer)
            .to_string()
            .trim_matches(char::from(0)),
    );
    Ok(result_string)
}

pub fn tcp(address: String, port: i32, data: String) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(handle_tcp(address, port, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::copy;
    use tokio::net::TcpListener;
    use tokio::task;

    async fn setup_tcp_listener(
        address: String,
    ) -> anyhow::Result<(i32, task::JoinHandle<anyhow::Result<()>>)> {
        let listener = TcpListener::bind(format!("{address}:0")).await?;
        let port: i32 = listener.local_addr().unwrap().port().into();
        let listener_handle = task::spawn(async move {
            let mut i = 0;
            while i < 5 {
                let (mut socket, _) = listener.accept().await?;
                let (mut reader, mut writer) = socket.split();
                let bytes_copied = copy(&mut reader, &mut writer).await?;
                if bytes_copied > 1 {
                    break;
                }
                i += 1;
            }
            Ok(())
        });
        Ok((port, listener_handle))
    }

    #[tokio::test]
    async fn test_tcp_send() -> anyhow::Result<()> {
        let expected_response = String::from("Hello world!");
        let (test_port, listen_task) = setup_tcp_listener(String::from("127.0.0.1")).await?;

        let send_task = task::spawn(handle_tcp(
            String::from("127.0.0.1"),
            test_port,
            expected_response.clone(),
        ));

        let (_a, actual_response) = tokio::join!(listen_task, send_task);
        assert_eq!(expected_response, actual_response.unwrap().unwrap());
        Ok(())
    }
}
