---
title: Imix
tags: 
 - User Guide
description: Imix User Guide
permalink: user-guide/imix
---
## What is Imix
Imix is the default agent for realm.
Imix currently only supports http callbacks which interact directly with the graphql API.

## Configuration
By default Imix is configured using a JSON file at run time.

The config is specified at run time with the `-c` flag.
For example:
```bash
./imix -c /tmp/imix-config.json
```

The imix config is as follows:
```json
{
    "service_configs": [],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {
        "interval": 4,
        "jitter": 1,
        "timeout": 4,
        "c2_configs": [
        {
            "priority": 1,
            "uri": "http://127.0.0.1/graphql"
        }
        ]
    }
}
```

- `service_configs`: Currently unused.
- `target_forward_connect_ip`: The IP address that you the red teamer would interact with the host through. This is to help keep track of agents when a hosts internal IP is different from the one you interact with in the case of a proxied host.
- `target_name`: Currently unused.
- `callback_config`: Define where and when the agent should callback.
    - `interval`: Number of seconds between callbacks.
    - `jitter`: Currently unused.
    - `timeout`: The number of seconds to wait before aborting a connection attempt.
    - `c2_config` Define where the c2 should callback to.
        - `priority`: The index that a domain should have.
        - `uri`: The full URI of the callback endpoint.

## Functionality 
Imix derives all it's functionality from the eldritch language.
See the [Eldritch User Guide](/user-guide/eldritch) for more information.

## Task management
Imix can execute up to 127 threads concurrently after that the main imix thread will block behind other threads.
Every callback interval imix will query each active thread for new output and rely that back to the c2. This means even long running jobs will report their status as new data comes in.
