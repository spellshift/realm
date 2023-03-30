---
title: Golem
tags: 
 - User Guide
description: Golem User Guide
permalink: user-guide/golem
---
## What is Golem
Golem is the standalone interpreter for Eldritch.
This program exists to help users get experience with the Eldritch language as well as a jumping off point if you're interested in implementing your own program using the Eldritch language.

Golem can also be used operationally as an alternative to a system native shell.
You can leverage the power of Eldritch with minimal exposure in the system process tree.

## Try it out.
```bash
git clone git@github.com:KCarretto/realm.git
cd realm/implants/golem
cargo run -- -i
# - or - 
../target/debug/golem ../../tests/golem_cli_test/tomes/hello_world.tome
```