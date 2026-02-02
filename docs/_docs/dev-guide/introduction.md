---
title: Introduction
tags:
 - Dev Guide
description: Read this before contributing to Realm!
permalink: dev-guide/introduction
---
# Overview

This section of the documentation is meant for new Realm-contributors, and should be read in its entirety before submitting your first PR. Below you can learn more about our testing & documentation requirements, project layout, and some of the internals of our codebase.

## Contribution Guidelines

### Documentation

Realm is under heavy active development and documentation can go stale quickly if it's not actively maintained. Please take a moment to familiarize yourself with both the **[Developer Documentation](/dev-guide)** you're reading now as well as the **[User-Facing Documentation](/user-guide)**. When submitting a code change, please include updates to the relevant portions of our documentation.

We will do our best during code review to catch changes that require documentation updates, but sometimes things will slip by. If you notice a discrepancy between our codebase and the documentation, please kindly [file an issue](https://github.com/spellshift/realm/issues/new?labels=documentation&title=Documentation%20Discrepancy:&body=Please%20include%20the%20location%20of%20the%20inaccurate%20documentation%20and%20a%20helpful%20description%20of%20what%20needs%20improvement.) to track it or submit a PR to correct it. You can use the ["Edit this page"](https://github.com/spellshift/realm/edit/main/docs/_docs/dev-guide/introduction.md) feature in the right navbar of the documentation to quickly navigate to the appropriate section of documentation that requires an update.

### Testing

Realm contains code across a variety of languages and frameworks. Testing helps ensure our codebase remains stable, enables us to refactor and develop new features with confidence, and can help new Realm developers understand the expected behavior of our code. Below is an outline of the testing requirements for each portion of the codebase, as well as guidance on testing best practices.

#### Eldritch

Any methods added to the Eldritch Standard Library should have tests collocated in the method's `<name>_impl.rs` file. Here are a few things to keep in mind:

* Tests should be cross platform
  * Rely on [NamedTempFile](https://docs.rs/tempfile/1.1.1/tempfile/struct.NamedTempFile.html) for temporary files
  * Rely on [path.join](https://doc.rust-lang.org/stable/std/path/struct.Path.html) to construct OS-agnostic paths

#### Tavern

##### Tavern Tests (Golang)

All code changes to Tavern must be tested. Below are some standards for test writing that can help improve your PRs:

* Please submit relevant tests in the same PR as your code change
* For GraphQL API Tests, please refer to our [YAML specification](/dev-guide/tavern#yaml-test-reference-graphql)
* For gRPC API Tests, please refer to our [YAML specification](/dev-guide/tavern#yaml-test-reference-grpc)
* Conventionally, please colocate your test code with the code it is testing and include it in the `<packagename>_test` package
* We rely on the standard [testify](https://github.com/stretchr/testify) assert & require libraries for ensuring expected values (or errors) are returned
* To enable a variety of inputs for a test case, we rely on closure-driven testing for Golang, you can read more about it [here](https://medium.com/@cep21/closure-driven-tests-an-alternative-style-to-table-driven-tests-in-go-628a41497e5e)
* Reusable test code should go in a sub-package suffixed with test
  * For example, reusable test code for the `ent` package would be located in the `ent/enttest` package
  * This convention is even used in the Golang standard library (e.g. [net/http](https://pkg.go.dev/net/http/httptest))
* Please use existing tests as a reference for writing new tests

##### Tavern Tests (Front End)

At the time of writing, the Tavern UI is still in an early stage, and therefore minimal testing exists for it. Once the UI is considered more stable, this documentation will be updated. If the Tavern UI is useable and this documentation still exists, please [file an issue](https://github.com/spellshift/realm/issues/new?labels=documentation&title=Documentation%20Discrepancy:&body=Please%20include%20the%20location%20of%20the%20inaccurate%20documentation%20and%20a%20helpful%20description%20of%20what%20needs%20improvement.).

### Linear History

In an attempt to reduce the complexity of merges, we enforce a linear history for Realm. This means that when your PR is merged, a "squash & merge" will be enforced so that only one commit is added onto the main branch. This means you can feel free to commit and push as often as you'd like, since all of your commits will be combined before merging your final changes.

# Project Structure

* **[.devcontainer](https://github.com/spellshift/realm/tree/main/.devcontainer)** contains settings required for configuring a VSCode dev container that can be used for Realm development
* **[.github](https://github.com/spellshift/realm/tree/main/.github)** contains GitHub related actions, issue templates, etc
* **[docker](https://github.com/spellshift/realm/tree/main/docker)** docker containers for production builds
* **[docs](https://github.com/spellshift/realm/tree/main/docs)** contains the Jekyll code for the documentation site that you're reading now!
* **[implants](https://github.com/spellshift/realm/tree/main/implants)** is the parent folder of any implant executables or libraries
  * **[implants/golem](https://github.com/spellshift/realm/tree/main/implants/golem)** the stand-alone interpreter that implements the eldritch language (Rust)
  * **[implants/golem/embed_files_golem_prod](https://github.com/spellshift/realm/tree/main/implants/golem/embed_files_golem_prod)** Files and scripts that will be embedded into production builds of imix, golem, and eldritch. These files can be accessed through the [`assets` module.](https://docs.realm.pub/user-guide/eldritch#assets)
  * **[implants/imix](https://github.com/spellshift/realm/tree/main/implants/imix)** is our agent that executes eldritch tomes (Rust)
  * **[implants/lib/eldritch](https://github.com/spellshift/realm/tree/main/implants/lib/eldritch)** is the source of our eldritch library (Rust)
* **[tavern](https://github.com/spellshift/realm/tree/main/tavern)** is the parent folder of Tavern related code and packages, and stores the `main.go` executable for the service
  * **[tavern/auth](https://github.com/spellshift/realm/tree/main/tavern/auth)** is a package for managing authentication for Tavern, and is used by various packages that rely on obtaining viewer information
  * **[tavern/internal/ent](https://github.com/spellshift/realm/tree/main/tavern/internal/ent)** contains models and related code for interacting with the database (most of this is code generated by **[entgo](https://entgo.io/))**
    * **[tavern/internal/ent/schema](https://github.com/spellshift/realm/tree/main/tavern/internal/ent/schema)** contains the schema definitions for our DB models
  * **[tavern/internal/graphql](https://github.com/spellshift/realm/tree/main/tavern/internal/graphql)** contains our GraphQL definitions and resolvers (most of this code is generated by **[entgo](https://entgo.io/)** and **[gqlgen](https://github.com/99designs/gqlgen))**
  * **[tavern/internal](https://github.com/spellshift/realm/tree/main/tavern/internal)** contains various internal packages that makeup Tavern
    * **[tavern/internal/www](https://github.com/spellshift/realm/tree/main/tavern/internal/www)** contains Tavern's UI code
* **[terraform](https://github.com/spellshift/realm/tree/main/terraform)** contains the Terraform used to deploy a production ready Realm instance. See [Tavern User Guide](https://docs.realm.pub/user-guide/tavern) to learn how to use.
* **[tests](https://github.com/spellshift/realm/tree/main/tests)** miscellaneous files and example code used for testing. Generally won't be used but is required for some niche situations like deadlocking cargo build.
* **[vscode](https://github.com/spellshift/realm/tree/main/vscode)** contains our Eldritch VSCode integration source code **(Unmaintained)**

# Where to Start?

If you'd like to make a contribution to Realm but aren't sure where to start or what features could use help, please consult our [Good First Issues](https://github.com/spellshift/realm/labels/good%20first%20issue) for some starting ideas.
