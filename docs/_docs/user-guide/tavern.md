---
title: Tavern
tags:
 - User Guide
description: User guide for interacting with Tavern.
permalink: user-guide/tavern
---

## Authentication

When interacting with Tavern, there are two primary methods of authentication: via the Web Interface (OAuth) and via the API (API Token). It is important to distinguish between these two as they serve different purposes and are used in different contexts.

### Web OAuth Token

This is the token generated when you log in to Tavern through a web browser using the configured OAuth provider (e.g., Google). It is used to maintain your session within the browser and allows you to access the Tavern UI.

### TAVERN_API_TOKEN

The `TAVERN_API_TOKEN` is a separate token used for authenticating CLI tools and scripts that interact with the Tavern API directly.

**Important:** This token is **different** from the web OAuth token. You cannot use the web OAuth token in place of the `TAVERN_API_TOKEN`.

#### When to use TAVERN_API_TOKEN

You typically need to use the `TAVERN_API_TOKEN` in scenarios where you are running tools on a remote machine (like a Kali VM via SSH) and cannot perform the standard local browser-based authentication flow due to networking restrictions (e.g., you cannot define the auth redirection port for SSH port forwarding).

In a standard local setup, CLI tools might pop open a browser window to authenticate. However, when you are SSH'd into a remote box, this isn't possible. The `TAVERN_API_TOKEN` provides a way to bypass this limitation.

## Configuration

Tavern is configured via environment variables.

| Variable | Description | Default |
| :--- | :--- | :--- |
| `ENABLE_TEST_DATA` | If set, populates the database with test data. | `false` |
| `ENABLE_TEST_RUN_AND_EXIT` | Starts the application but exits immediately after (for testing). | `false` |
| `DISABLE_DEFAULT_TOMES` | Prevents default tomes from being imported on startup. | `false` |
| `ENABLE_DEBUG_LOGGING` | Emits verbose debug logs. | `false` |
| `ENABLE_JSON_LOGGING` | Emits logs in JSON format. | `false` |
| `ENABLE_INSTANCE_ID_LOGGING` | Includes the tavern instance ID in log messages. | `false` |
| `ENABLE_GRAPHQL_RAW_QUERY_LOGGING` | Includes raw GraphQL queries in logs. | `false` |
| `HTTP_LISTEN_ADDR` | Address for Tavern's HTTP server to bind to. | `0.0.0.0:8000` |
| `HTTP_METRICS_LISTEN_ADDR` | Address for the HTTP metrics server. | `127.0.0.1:8080` |
| `OAUTH_CLIENT_ID` | OAuth Client ID. | `""` |
| `OAUTH_CLIENT_SECRET` | OAuth Client Secret. | `""` |
| `OAUTH_DOMAIN` | OAuth domain for consent flow. | `""` |
| `MYSQL_ADDR` | MySQL address to connect to. If unset, SQLite is used. | `""` |
| `MYSQL_NET` | Network used to connect to MySQL (e.g. tcp). | `tcp` |
| `MYSQL_USER` | MySQL user. | `root` |
| `MYSQL_PASSWD` | MySQL password. | `""` |
| `MYSQL_DB` | MySQL database name. | `tavern` |
| `DB_MAX_IDLE_CONNS` | Max idle DB connections. | `10` |
| `DB_MAX_OPEN_CONNS` | Max open DB connections. | `100` |
| `DB_MAX_CONN_LIFETIME` | Max lifetime of a DB connection (seconds). | `3600` |
| `GCP_PROJECT_ID` | GCP Project ID for Google Cloud Platform deployments. | `""` |
| `GCP_PUBSUB_KEEP_ALIVE_INTERVAL_MS` | Interval to publish no-op pubsub messages (ms). | `1000` |
| `PUBSUB_TOPIC_SHELL_INPUT` | PubSub topic for shell input. | `mem://shell_input` |
| `PUBSUB_SUBSCRIPTION_SHELL_INPUT` | PubSub subscription for shell input. | `mem://shell_input` |
| `PUBSUB_TOPIC_SHELL_OUTPUT` | PubSub topic for shell output. | `mem://shell_output` |
| `PUBSUB_SUBSCRIPTION_SHELL_OUTPUT` | PubSub subscription for shell output. | `mem://shell_output` |
| `ENABLE_PPROF` | Enables performance profiling. | `false` |
| `ENABLE_METRICS` | Enables the /metrics endpoint. | `false` |
| `SECRETS_FILE_PATH` | Path to secrets file (if not using GCP Secrets Manager). | `""` |

## CLI

Tavern includes a built-in CLI for managing redirectors.

### Redirectors

To run a redirector:

```bash
tavern redirector [flags] <upstream_address>
```

**Flags:**
- `--listen`: Address to listen on (default: `:8080`)
- `--transport`: Transport protocol to use (default: `grpc`)

**Example:**
```bash
tavern redirector --listen :8080 --transport grpc 127.0.0.1:8000
```

To list available redirector types:

```bash
tavern redirector list
```
