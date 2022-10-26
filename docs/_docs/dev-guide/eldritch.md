---
title: Eldritch
tags: 
 - Dev Guide
description: Want to contribute to Eldritch? Start here!
permalink: dev-guide/eldritch
---
# Overview
![/assets/img/coming-soon.gif](/assets/img/coming-soon.gif)

# Adding a Method
What can / should be done in an eldritch method?

### What files should I modify to make an eldritch funciotn.
`docs/_docs/user-guide/eldritch.md`
Add your command to the docs. Give your command a unique and descriptive name. 
If your command does not fall under a specific module reach out to the core developers about adding a new module or finding the right fit.
Specify the input and output according to the [Starklark types spec.](https://docs.rs/starlark/0.6.0/starlark/values/index.html)
If there are OS or edge case specific behaviors make sure to document them here.
```MD
module.command(arg1: str, arg2: int, arg3: list) -> bool

The <b>module.command</b> describe your command.

```



### Testing

### Example PR for an eldritch method.