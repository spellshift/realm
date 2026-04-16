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

You typically need to use the `TAVERN_API_TOKEN` in scenarios where you are running tools on a remote machine (like a Kali VM via SSH) and cannot perform the standard Remote Device Authentication (RDA) flow. By default, CLI tools will use the RDA flow to authenticate.

To enable the legacy local browser-based OAuth flow, you can set the `TAVERN_USE_BROWSER_OAUTH=1` environment variable. In a standard local setup with this variable set, CLI tools might pop open a browser window to authenticate. However, when you are SSH'd into a remote box, this isn't possible, which is why the default is the RDA flow. The `TAVERN_API_TOKEN` provides a way to bypass these limitations altogether.

## MCP (Model Context Protocol) Integration

Tavern supports the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), allowing AI assistants and LLM-based tools to interact with your Tavern deployment. The MCP server must be enabled by an administrator (see the [Admin Guide](/admin-guide/tavern#ai-mcp-server)).

### Connecting an MCP Client

Once the MCP server is enabled on your Tavern instance, you can connect any MCP-compatible client using the Streamable HTTP transport.

#### Connection Details

- **Endpoint:** `https://<your-tavern-url>/mcp`
- **Transport:** Streamable HTTP
- **Authentication:** Use your Tavern session cookie or API access token

#### Example: Claude Desktop

Add the following to your Claude Desktop MCP configuration file (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "tavern": {
      "url": "https://<your-tavern-url>/mcp",
      "headers": {
        "X-Tavern-Access-Token": "<your-access-token>"
      }
    }
  }
}
```

Replace `<your-tavern-url>` with the URL of your Tavern deployment and `<your-access-token>` with your personal access token from Tavern.

#### Example: Cursor / VS Code

For editors that support MCP via Streamable HTTP, configure the server URL as:

```
https://<your-tavern-url>/mcp
```

And set the authentication header `X-Tavern-Access-Token` to your personal access token.

### Available Tools

The MCP server exposes the following tools:

| Tool | Description | Parameters |
| ---- | ----------- | ---------- |
| `list_quests` | List all quests in Tavern | None |
| `quest_output` | Get the output of specific quests | `ids` (array of quest ID strings) |
| `list_tomes` | List all available tomes and their required parameters | None |
| `create_quest` | Create a new quest targeting specific beacons | `name`, `beacon_ids`, `parameters`, `tome_id` |
| `list_hosts` | List all hosts with their beacons and tags | None |
| `wait_for_quest` | Wait for all tasks in a quest to finish (polls for up to 10 min) | `quest_id` |

### Typical Workflow

1. Use `list_hosts` to discover available hosts and their beacon IDs
2. Use `list_tomes` to find the right tome for your operation
3. Use `create_quest` to create a quest targeting specific beacons with the chosen tome
4. Use `wait_for_quest` to wait for the quest to complete
5. Use `quest_output` to retrieve the results
