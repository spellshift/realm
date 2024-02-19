---
title: Terminology
tags:
 - User Guide
description: Various terms used throughout Realm.
permalink: user-guide/terminology
---

## Terminology

Throughout the documentation terms like "agent" or "implant" are used to reference various components of Realm. Below we attempt to define some of those terms, to add some clarity to that other documentation.

### Host

A Host is a system that is in-scope for the current engagement. It is used to establish a logical boundary between different systems in an engagement (e.g. between a webserver and a database). This enables operations to target a particular system, for example you may want to list files on a web server in your engagement scope.

### Implant

References malicious code or persistence mechanisms that are deployed to compromise target systems. Imix is the primary implant used with Realm.

### Agent

An Agent is a type of implant which retrieves execution instructions by connecting to our backend infrastructure (calling back) and querying for new tasks.

### Beacon

A Beacon is a running instance of an Agent. A Host may have multiple active Beacons that use the same underlying Agent.

### Task

A Task represents a set of instructions for an Agent to perform. For example, listing files could be a Task. When listing files across various Beacons, one Task per Beacon will be created for tracking the individual execution output.

### Quest

A Quest represents a collection of tasks, each with a unique host. This is how Realm enables multi-host management.

### Eldritch

Eldritch is our Pythonic Domain Specific Language (DSL), which can be used to progammatically define red team operations. Many of the language's built-in features do not rely on system binaries. For more information, please see the [Eldritch section](/user-guide/eldritch) of the documentation.

### Tome

A Tome is a prebuilt Eldritch bundle, which provides execution instructions to a Beacon. Tomes can embed files and accept parameters to change their behavior at runtime. Tavern's built-in Tomes are defined [here](https://github.com/spellshift/realm/tree/main/tavern/tomes).
