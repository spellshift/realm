---
title: Getting Started
tags:
 - User Guide
description: Getting started with Realm
permalink: user-guide/getting-started
---

## Getting Started

*To deploy a production ready instance see the [tavern setup guide](https://docs.realm.pub/user-guide/tavern).*

### Start the server

```bash
git clone https://github.com/spellshift/realm.git
cd realm && go run ./tavern

# If you'd like to test without deploying an agent use the test data.
ENABLE_TEST_DATA=1 go run ./tavern
```

### Start the agent

```bash
git clone https://github.com/spellshift/realm.git
cd realm/implants/imix && cargo run
```

Want to work with the API? Check out the [sample queries](https://docs.realm.pub/dev-guide/tavern#graphql-api)
