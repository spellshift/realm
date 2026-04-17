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
