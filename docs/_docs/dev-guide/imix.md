---
title: Imix
tags:
 - Dev Guide
description: Want to implement new functionality in the agent? Start here!
permalink: dev-guide/imix
---

## Overview

Imix in the main bot for Realm.

## Host Selector

The host selector defined in `implants/lib/host_selector` allow imix to reliably identify which host it's running on. This is helpful for operators when creating tasking across multiple beacons as well as when searching for command results. Uniqueness is stored as a UUID4 value.

Out of the box realm comes with two options `File` and `Env` to determine what host it's on.

`File` will create a file on disk that stores the UUID4 Eg. Linux:

```bash
[~]$ cat /etc/system-id
36b3c472-d19b-46cc-b3e6-ee6fd8da5b9c
```

`Env` will read from the agent environment variables looking for `IMIX_HOST_ID` if it's set it will use the UUID4 string set there.

There is a third option available on Windows systems to store the UUID value inside a registry key. Follow the steps below to update `lib.rs` to include `Registry` as a default before `File` to enable it. On hosts that are not Windows, imix will simply skip `Registry`.

If no selectors succeed a random UUID4 ID will be generated and used for the bot. This should be avoided.

## Develop A Host Uniqueness Selector

To create your own:

- Navigate to `implants/lib/host_unique`
- Create a file for your selector `touch mac_address.rs`
- Create an implementation of the `HostIDSelector`

```rust
use uuid::Uuid;

use crate::HostIDSelector;

pub struct MacAddress {}

impl Default for MacAddress {
    fn default() -> Self {
        MacAddress {}
    }
}

impl HostIDSelector for MacAddress {
    fn get_name(&self) -> String {
        "mac_address".to_string()
    }

    fn get_host_id(&self) -> Option<uuid::Uuid> {
        // Get the mac address
        // Generate a UUID using it
        // Return the UUID
        // Return None if anything fails
    }
}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_id_mac_consistent() {
        let selector = MacAddress {};
        let id_one = selector.get_host_id();
        let id_two = selector.get_host_id();

        assert_eq!(id_one, id_two);
    }
}
```

- Update `lib.rs` to re-export your implementation

```rust
mod mac_address;
pub use mac_address::MacAddress;
```

- Update the `defaults()` function to include your implementation. N.B. The order from left to right is the order engines will be evaluated.

## Develop a New Transport

We've tried to make Imix super extensible for transport development. In fact, all of the transport specific logic is complete abstracted from how Imix operates for callbacks/tome excution. For Imix all Transports live in the `realm/implants/lib/transport/src` directory.

If creating a new Transport create a new file in the directory and name it after the protocol you plan to use. For example, if writing a DNS Transport then call your file `dns.rs`. Then define your public struct where any connection state/clients will be. For example,

```rust
#[derive(Debug, Clone)]
pub struct DNS {
    dns_client: Option<hickory_dns::Client>
}
```

NOTE: Depending on the struct you build, you may need to derive certain features, see above we derive `Debug` and `Clone`.

Next, we need to implement the Transport trait for our new struct. This will look like:

```rust
impl Transport for DNS {
    fn init() -> Self {
        DNS{ dns_client: None }
    }
    fn new(callback: String, proxy_uri: Option<String>) -> Result<Self> {
        // TODO: setup connection/client hook in proxy, anything else needed
        // before fuctions get called.
        Err(anyhow!("Unimplemented!"))
    }
    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        // TODO: How you wish to handle the `claim_tasks` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: std::sync::mpsc::Sender<FetchAssetResponse>,
    ) -> Result<()> {
        // TODO: How you wish to handle the `fetch_asset` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        // TODO: How you wish to handle the `report_credential` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_file(
        &mut self,
        request: std::sync::mpsc::Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        // TODO: How you wish to handle the `report_file` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        // TODO: How you wish to handle the `report_process_list` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        // TODO: How you wish to handle the `report_task_output` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        // TODO: How you wish to handle the `reverse_shell` method.
        Err(anyhow!("Unimplemented!"))
    }
}
```

NOTE: Be Aware that currently `reverse_shell` uses tokio's sender/reciever while the rest of the methods rely on mpsc's. This is an artifact of some implementation details under the hood of Imix. Some day we may wish to move completely over to tokio's but currenlty it would just result in performance loss/less maintainable code.

After you implement all the functions/write in a decent error message for operators to understand why the function call failed then you need to import the Transport to the broader lib scope. To do this open up `realm/implants/lib/transport/src/lib.rs` and add in your new Transport like so:

```rust
// more stuff above

#[cfg(feature = "dns")]
mod dns;
#[cfg(feature = "dns")]
pub use dns::DNS as ActiveTransport;

// more stuff below
```

Also add your new feature to the Transport Cargo.toml at `realm/implants/lib/transport/Cargo.toml`.

```toml
# more stuff above

[features]
default = []
grpc = []
dns = [] # <-- see here
mock = ["dep:mockall"]

# more stuff below
```

Then make sure the feature flag is populated down from the imix crate `realm/implants/imix/Cargo.toml`
```toml
# more stuff above

[features]
default = ["transport/grpc"]
http1 = ["transport/http1"]
dns = ["transport/dns"]
transport-grpc-doh = ["transport/grpc-doh"]

# more stuff below
```

And that's all that is needed for Imix to use a new Transport! Now all there is to do is setup the Tarver redirector see the [tavern dev docs here](/dev-guide/tavern#transport-development)
