---
title: Tavern
tags:
 - User Guide
description: User guide for interacting with Tavern.
permalink: user-guide/tavern
---

## Configuration

Tavern is configured primarily through environment variables. Below is a list of available configuration options and their default values.

| Environment Variable | Description | Default |
|----------------------|-------------|---------|
| `ENABLE_TEST_DATA` | Enables the generation of test data. | `false` |
| `ENABLE_TEST_RUN_AND_EXIT` | Enables running in test mode and exiting. | `false` |
| `DISABLE_DEFAULT_TOMES` | Disables the loading of default tomes. | `false` |
| `ENABLE_DEBUG_LOGGING` | Enables debug level logging. | `false` |
| `ENABLE_JSON_LOGGING` | Enables JSON formatted logging. | `false` |
| `ENABLE_INSTANCE_ID_LOGGING` | Enables logging of the instance ID. | `false` |
| `ENABLE_GRAPHQL_RAW_QUERY_LOGGING` | Enables logging of raw GraphQL queries. | `false` |
| `ENABLE_PPROF` | Enables pprof profiling endpoint. | `false` |
| `ENABLE_METRICS` | Enables Prometheus metrics endpoint. | `false` |
| `DB_MAX_IDLE_CONNS` | Maximum number of idle database connections. | `10` |
| `DB_MAX_OPEN_CONNS` | Maximum number of open database connections. | `100` |
| `DB_MAX_CONN_LIFETIME` | Maximum lifetime of a database connection in seconds. | `3600` |
| `GCP_PUBSUB_KEEP_ALIVE_INTERVAL_MS` | Keep-alive interval for GCP PubSub in milliseconds. | `1000` |
| `HTTP_LISTEN_ADDR` | Address to listen on for the HTTP server. | `0.0.0.0:8000` |
| `HTTP_METRICS_LISTEN_ADDR` | Address to listen on for the metrics server. | `127.0.0.1:8080` |
| `OAUTH_CLIENT_ID` | OAuth Client ID. | `""` |
| `OAUTH_CLIENT_SECRET` | OAuth Client Secret. | `""` |
| `OAUTH_DOMAIN` | OAuth Domain. | `""` |
| `MYSQL_ADDR` | MySQL address. | `""` |
| `MYSQL_NET` | MySQL network type (e.g., tcp). | `tcp` |
| `MYSQL_USER` | MySQL user. | `root` |
| `MYSQL_PASSWD` | MySQL password. | `""` |
| `MYSQL_DB` | MySQL database name. | `tavern` |
| `GCP_PROJECT_ID` | Google Cloud Project ID. | `""` |
| `PUBSUB_TOPIC_SHELL_INPUT` | PubSub topic for shell input. | `mem://shell_input` |
| `PUBSUB_SUBSCRIPTION_SHELL_INPUT` | PubSub subscription for shell input. | `mem://shell_input` |
| `PUBSUB_TOPIC_SHELL_OUTPUT` | PubSub topic for shell output. | `mem://shell_output` |
| `PUBSUB_SUBSCRIPTION_SHELL_OUTPUT` | PubSub subscription for shell output. | `mem://shell_output` |
| `SECRETS_FILE_PATH` | Path to the secrets file. | `""` |

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
