---
title: Getting Started
tags:
 - User Guide
description: Getting started with Realm
permalink: user-guide/getting-started
---

# Getting Started

*To deploy a production ready instance see the [tavern setup guide](https://docs.realm.pub/user-guide/tavern).*

### Start the server

```bash
git clone https://github.com/KCarretto/realm.git
cd realm
go run ./tavern

# If you'd like to test without deploying an agent use the test data.
ENABLE_TEST_DATA=1 go run ./tavern
```

### Start the agent

```bash
git clone https://github.com/KCarretto/realm.git
cd realm/implants/imix

# Create the config file
cat <<EOF > /tmp/imix-config.json
{
    "service_configs": [
        {
            "name": "imix",
            "description": "Imix c2 agent",
            "executable_name": "imix",
            "executable_args": ""
        }
    ],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {
        "interval": 4,
        "jitter": 1,
        "timeout": 4,
        "c2_configs": [
        {
            "priority": 1,
            "uri": "http://127.0.0.1/grpc/"
        }
        ]
    }
}
EOF

cargo run -- -c /tmp/imix-config.json
```

Want to work with the API? Check out the [sample queries](https://docs.realm.pub/dev-guide/tavern#graphql-api)
