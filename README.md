
<div align="center">
 <img src="./docs/assets/img/realm_250px.png">
</div>

# Realm

![test-status](https://github.com/spellshift/realm/actions/workflows/tests.yml/badge.svg?branch=main)
[![codecov](https://codecov.io/github/spellshift/realm/branch/main/graph/badge.svg?token=KSRPHYDIE4)](https://app.codecov.io/github/spellshift/realm)
[![Go Report Card](https://goreportcard.com/badge/github.com/spellshift/realm)](https://goreportcard.com/report/github.com/spellshift/realm)
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/spellshift/realm)](https://rust-reportcard.xuri.me/report/github.com/spellshift/realm)
[![Docs](https://img.shields.io/badge/read%20our-docs-informational)](https://docs.realm.pub/)

Realm is a cross platform Red Team engagement platform with a focus on automation and reliability.

https://github.com/spellshift/realm/assets/16250309/7b5834d9-a864-490a-96e5-8d83b276af11

## Features

### Agent (imix)

- Written in rust with support for MacOS, Linux, and Windows.
- Supports long running tasks by reading output from tasks in real time.
- Interval callback times.
- Simple file based configuration.
- Embedded files.
- Built-in interpreter.

### Server (tavern)

- Web interface.
- Group actions.
- graphql backend for easy API access.
- OAuth login support.
- Cloud native deployment with pre-made terraform for production deployments.

### Built-in interpreter (eldritch)

- Reflective DLL Loader.
- Port scanning.
- Remote execution over SSH.
- And much much more: <https://docs.realm.pub/user-guide/eldritch>

## Quickstart guide

*To deploy a production ready instance see the [tavern setup guide](https://docs.realm.pub/user-guide/tavern).*

### Start the server

```bash
git clone https://github.com/spellshift/realm.git && cd realm

go run ./tavern

# If you'd like to test without deploying an agent use the test data.
ENABLE_TEST_DATA=1 go run ./tavern
```

### Start the agent

```bash
git clone https://github.com/spellshift/realm.git
cd realm/implants/imix && cargo run

```

## Want to contribute start here

<https://docs.realm.pub/dev-guide/introduction>
