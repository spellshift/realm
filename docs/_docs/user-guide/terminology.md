---
title: Terminology
tags:
 - User Guide
description: Various terms used throughout Realm.
permalink: user-guide/terminology
---

## Terminology

Throughout the documentation, terms are used to reference various components of Realm. Below we attempt to define some of those terms, to add some clarity to that other documentation.

### Implant

References malicious code or persistence mechanisms that are deployed to compromise target systems. Imix is the primary implant used with Realm.

### Agent

An Agent is a type of implant which retrieves execution instructions by connecting to our backend infrastructure (calling back) and querying for new tasks.

### Beacon

A Beacon is an instance of an Agent running as a process. Beacons are the underlying unit that can be tasked with instructions to execute.

### Host

Hosts are in-scope systems for the current engagement. A host can have multiple beacons, which can execute instructions provided by tomes.

### Quest

Quests enable multi-beacon management by taking a list of beacons and executing a tome with customized parameters against them. A quest is made up of tasks associated with a single beacon.

### Task

A task is a single instance of a tome plus its parameters executed against a single beacon. For example, listing files could be a Task. When listing files across various Beacons, one Task per Beacon will be created for tracking the individual execution output.

### Eldritch

Eldritch is our Pythonic Domain Specific Language (DSL), which can be used to programmatically define red team operations. Many of the language's built-in features do not rely on system binaries. For more information, please see the [Eldritch section](/user-guide/eldritch) of the documentation.

### Tome

A Tome is a prebuilt Eldritch bundle, which includes execution instructions and embedded files. Tomes are how beacon actions are defined and change their behavior at runtime. Tavern's built-in Tomes are defined [here](https://github.com/spellshift/realm/tree/main/tavern/tomes).
