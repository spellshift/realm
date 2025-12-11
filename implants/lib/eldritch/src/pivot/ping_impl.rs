use super::super::insert_dict_kv;
use rand::random;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap},
};
use std::{net::IpAddr, time::Duration};
use surge_ping::IcmpPacket;

pub fn ping(starlark_heap: &'_ Heap, target: String) -> anyhow::Result<Dict<'_>> {
    let mut dict = Dict::new(SmallMap::new());

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let ip = target.parse()?;

    let reply = runtime.block_on(handle_ping(ip, &[1, 2, 3, 4]))?;

    match reply.0 {
        IcmpPacket::V4(pingv4) => {
            insert_dict_kv!(
                dict,
                starlark_heap,
                "addr",
                pingv4.get_real_dest().to_string(),
                String
            );
            let ttl = pingv4.get_ttl().unwrap_or(0);
            insert_dict_kv!(dict, starlark_heap, "ttl", format!("{}", ttl), String);
        }
        IcmpPacket::V6(pingv6) => {
            insert_dict_kv!(
                dict,
                starlark_heap,
                "addr",
                pingv6.get_real_dest().to_string(),
                String
            );
            let ttl = pingv6.get_max_hop_limit();
            insert_dict_kv!(dict, starlark_heap, "ttl", format!("{}", ttl), String);
        }
    }

    let dur = reply.1.as_millis();
    insert_dict_kv!(
        dict,
        starlark_heap,
        "duration",
        format!("{}ms", dur),
        String
    );

    Ok(dict)
}

async fn handle_ping(
    host: IpAddr,
    payload: &[u8],
) -> anyhow::Result<(surge_ping::IcmpPacket, Duration)> {
    let config = surge_ping::Config::builder().ttl(64).build();
    let client = surge_ping::Client::new(&config)?;
    let mut pinger = client
        .pinger(host, surge_ping::PingIdentifier(random()))
        .await;
    Ok(pinger.ping(surge_ping::PingSequence(0), payload).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use starlark::values::Heap;

    #[test]
    fn test_ping() -> Result<()> {
        let test_heap = Heap::new();
        let target = String::from("127.0.0.1");
        let r = ping(&test_heap, target.clone())?;
        let addr = r
            .get(const_frozen_string!("addr").to_value())
            .expect("no addr")
            .expect("no addr")
            .unpack_str()
            .expect("addr is not str");
        let ttl = r
            .get(const_frozen_string!("ttl").to_value())
            .expect("no ttl")
            .expect("no ttl")
            .unpack_str()
            .expect("ttl is not str");
        let duration = r
            .get(const_frozen_string!("duration").to_value())
            .expect("no duration")
            .expect("no duration")
            .unpack_str()
            .expect("duration is not str");
        assert!(
            addr == target.as_str(),
            "addr check: {} == {}",
            addr,
            target.as_str()
        );
        assert!(ttl == "0", "addr check: {} == {}", addr, "0");
        assert!(duration == "0ms", "addr check: {} == {}", addr, "0ms");
        Ok(())
    }

    #[test]
    fn test_ping_fail() -> Result<()> {
        let test_heap = Heap::new();
        let target = String::from("169.254.0.1");
        assert!(
            ping(&test_heap, target.clone()).is_err(),
            "ping for fake APIPA did not fail!"
        );
        Ok(())
    }
}
