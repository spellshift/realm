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

#### Portal Auth Flow

To obtain a `TAVERN_API_TOKEN`, you can use the Portal Auth Flow:

1.  Navigate to your User Profile in the Tavern Web UI.
2.  Look for an option to generate or view your API Token.
3.  Copy this token.
4.  On your remote machine (e.g., the Kali VM), set the `TAVERN_API_TOKEN` environment variable:

    ```bash
    export TAVERN_API_TOKEN="your_token_here"
    ```

Once this environment variable is set, compatible Tavern CLI tools and scripts will use it to authenticate their requests, bypassing the need for an interactive browser login.
