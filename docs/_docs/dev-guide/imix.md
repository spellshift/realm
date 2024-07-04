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

If no selectors succeed a random UUID4 ID will be generated and used for the bot. This should be avoided.

## Develop A Host Uniqueness Engine

To create your own:

- Navigate to `implants/lib/host_unique`
- Create a file for your selector `touch mac_address.rs`
- Create an implementation of the `HostUniqueEngine`

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
