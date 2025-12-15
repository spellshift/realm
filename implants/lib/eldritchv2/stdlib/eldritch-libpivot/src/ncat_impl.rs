use std::net::Ipv4Addr;

use crate::std::StdPivotLibrary;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};

async fn handle_ncat(address: String, port: i32, data: String, protocol: String) -> Result<String> {
    let mut response_buffer: Vec<u8> = Vec::new();
    let result_string: String;

    let address_and_port = format!("{address}:{port}");

    if protocol == "tcp" {
        let mut connection = TcpStream::connect(&address_and_port).await?;
        connection.write_all(data.as_bytes()).await?;

        let mut read_stream = BufReader::new(connection);
        read_stream.read_buf(&mut response_buffer).await?;

        result_string = String::from(
            String::from_utf8_lossy(&response_buffer)
                .to_string()
                .trim_matches(char::from(0)),
        );
        Ok(result_string)
    } else if protocol == "udp" {
        let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;

        let _bytes_sent = sock
            .send_to(data.as_bytes(), address_and_port.clone())
            .await?;

        let mut response_buffer = [0; 1024];
        let (_bytes_copied, _addr) = sock.recv_from(&mut response_buffer).await?;

        result_string =
            String::from(String::from_utf8(response_buffer.to_vec())?.trim_matches(char::from(0)));
        Ok(result_string)
    } else {
        Err(anyhow::anyhow!(
            "Protocol not supported please use: udp or tcp."
        ))
    }
}

pub fn run(
    lib: &StdPivotLibrary,
    address: String,
    port: i64,
    data: String,
    protocol: String,
) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    let address_clone = address.clone();
    let port_i32 = port as i32;
    let data_clone = data.clone();
    let protocol_clone = protocol.clone();

    let fut = async move {
        let res = handle_ncat(address_clone, port_i32, data_clone, protocol_clone).await;
        let _ = tx.send(res);
    };

    lib.agent
        .spawn_subtask(lib.task_id, "ncat".to_string(), alloc::boxed::Box::pin(fut))
        .map_err(|e| e.to_string())?;

    let response = rx.recv().map_err(|e| format!("Failed to receive result: {}", e))?;

    response.map_err(|e| format!("ncat failed: {:?}", e))
}
