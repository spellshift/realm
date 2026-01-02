---
title: Imix
tags:
 - Dev Guide
description: Want to implement new functionality in the agent? Start here!
permalink: dev-guide/imix
---

## Overview

Imix in the main bot for Realm.

## Agent protobuf

In order to communicate agent state and configuration during the claimTask request the agent sends a protobuf containing various configuration options. If any are updated agent side they're now syncronsized with the server ensuring operators can track the state of their agents.

In order to keep these configuration options in sync realm uses protobuf and code generation to ensure agent and server agree.

If you need to update these fields start with the `tavern/internal/c2/proto/c2.proto` file.

Once you've finished making your changes apply these changes across the project using `cd /workspaces/realm/ && go generater ./tavern/...`

To generate the associated agent proto's use cargo build in the `implants` direcotry. This will copy the necesarry protos from tavern and preform the code generation.

### Adding enums

Add your enum type to the `*.proto` file under the message type that will use it.
For example:
```
message ActiveTransport {
    string uri = 1;
    uint64 interval = 2;

    enum Type {
        TRANSPORT_UNSPECIFIED = 0;
        TRANSPORT_GRPC = 1;
        TRANSPORT_HTTP1 = 2;
        TRANSPORT_DNS = 3;
    }

    Type type = 3;
    string extra = 4;
}
```

And add a new enum definition to `tavern/internal/c2/c2pb/enum_<MESSAGE NAME>_<ENUM NAME>.go` This should be similar to other enums that exist you can likely copy and rename an existing one. See `tavern/internal/c2/c2pb/enum_beacon_active_transport_type.go`


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

We've tried to make Imix super extensible for transport development. In fact, all of the transport specific logic is completely abstracted from how Imix operates for callbacks/tome execution. For Imix all Transports live in the `realm/implants/lib/transport/src` directory.

### Current Available Transports

Realm currently includes three transport implementations:

- **`grpc`** - Default gRPC transport (with optional DoH support via `grpc-doh` feature)
- **`http1`** - HTTP/1.1 transport
- **`dns`** - DNS-based covert channel transport

**Note:** Only one transport may be selected at compile time. The build will fail if multiple transport features are enabled simultaneously.

### Creating a New Transport

If creating a new Transport, create a new file in the `realm/implants/lib/transport/src` directory and name it after the protocol you plan to use. For example, if writing a new protocol called "Custom" then call your file `custom.rs`. Then define your public struct where any connection state/clients will be stored. For example,

```rust
#[derive(Debug, Clone)]
pub struct Custom {
    // Your connection state here
    // e.g., client: Option<CustomClient>
}
```

**NOTE:** Your struct **must** derive `Clone` and `Send` as these are required by the Transport trait. Deriving `Debug` is also recommended for troubleshooting.

Next, we need to implement the Transport trait for our new struct. This will look like:

```rust
impl Transport for Custom {
    fn init() -> Self {
        Custom {
            // Initialize your connection state here
            // e.g., client: None
        }
    }
    fn new(callback: String, config: Config) -> Result<Self> {
        // TODO: setup connection/client hook in proxy, anything else needed
        // before functions get called.
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

After you implement all the functions and write descriptive error messages for operators to understand why function calls failed, you need to:

#### 1. Add Compile-Time Exclusivity Checks

In `realm/implants/lib/transport/src/lib.rs`, add compile-time checks to ensure your new transport cannot be compiled alongside others:

```rust
// Add your transport to the mutual exclusivity checks
#[cfg(all(feature = "grpc", feature = "custom"))]
compile_error!("only one transport may be selected");
#[cfg(all(feature = "http1", feature = "custom"))]
compile_error!("only one transport may be selected");
#[cfg(all(feature = "dns", feature = "custom"))]
compile_error!("only one transport may be selected");

// ... existing checks above ...

// Add your transport module and export
#[cfg(feature = "custom")]
mod custom;
#[cfg(feature = "custom")]
pub use custom::Custom as ActiveTransport;
```

**Important:** The transport is exported as `ActiveTransport`, not by its type name. This allows the imix agent code to remain transport-agnostic.

#### 2. Update Transport Library Dependencies

Add your new feature and any required dependencies to `realm/implants/lib/transport/Cargo.toml`:

```toml
# more stuff above

[features]
default = []
grpc = []
grpc-doh = ["grpc", "dep:hickory-resolver"]
http1 = []
dns = ["dep:data-encoding", "dep:rand"]
custom = ["dep:your-custom-dependency"] # <-- Add your feature here
mock = ["dep:mockall"]

[dependencies]
# ... existing dependencies ...

# Add any dependencies needed by your transport
your-custom-dependency = { version = "1.0", optional = true }

# more stuff below
```

#### 3. Enable Your Transport in Imix

To use your new transport, update the imix Cargo.toml at `realm/implants/imix/Cargo.toml`:

```toml
# more stuff above

[features]
# Check if compiled by imix
win_service = []
default = ["transport/grpc"]  # Default transport
http1 = ["transport/http1"]
dns = ["transport/dns"]
custom = ["transport/custom"]  # <-- Add your feature here
transport-grpc-doh = ["transport/grpc-doh"]

# more stuff below
```

#### 4. Build Imix with Your Transport

Compile imix with your custom transport:

```bash
# From the repository root
cd implants/imix

# Build with your transport feature
cargo build --release --features custom --no-default-features

# Or for the default transport (grpc)
cargo build --release
```

**Important:** Only specify one transport feature at a time. The build will fail if multiple transport features are enabled. Ensure you include `--no-default-features` when building with a non-default transport.

#### 5. Set Up the Corresponding Redirector

For your agent to communicate, you'll need to implement a corresponding redirector in Tavern. See the redirector implementations in `tavern/internal/redirectors/` for examples:

- `tavern/internal/redirectors/grpc/` - gRPC redirector
- `tavern/internal/redirectors/http1/` - HTTP/1.1 redirector
- `tavern/internal/redirectors/dns/` - DNS redirector

Your redirector must implement the `Redirector` interface and register itself in the redirector registry. See `tavern/internal/redirectors/redirector.go` for the interface definition.

And that's all that is needed for Imix to use a new Transport! The agent code automatically uses whichever transport is enabled at compile time via the `ActiveTransport` type alias.
