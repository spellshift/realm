---
title: Tavern
tags:
 - Dev Guide
description: Want to contribute to Tavern? Start here!
permalink: dev-guide/tavern
---

## Overview

Tavern is a teamserver for Realm, providing a UI to control deployments and implants during an engagement. The majority of Tavern's functionality is exposed through a GraphQL API, which is used by both implants and the UI.

If you would like to help contribute to Tavern, please take a look at our [open issues](https://realm.pub/issues?q=is%3Aopen+is%3Aissue+label%3Atavern).

## Configuration

### MySQL

By default, Tavern operates an in-memory SQLite database. To persist data, a MySQL backend is supported. In order to configure Tavern to use MySQL, the `MYSQL_ADDR` environment variable must be set to the `host[:port]` of the database (e.g. `127.0.0.1`, `mydb.com`, or `mydb.com:3306`). You can reference the [mysql.Config](https://pkg.go.dev/github.com/go-sql-driver/mysql#Config) for additional information about Tavern's MySQL configuration.

The following environment variables are currently supported for additional MySQL Configuration:

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| MYSQL_ADDR| Address of the MySQL server (e.g. `host[:port]`) | N/A | **_Yes_** |
| MYSQL_NET| Network type (e.g. unix) | tcp | No |
| MYSQL_USER| User to authenticate with | root | No |
| MYSQL_PASSWD| Password to authenticate with | no password | No |
| MYSQL_DB| Name of the database to use | tavern | No |
| MYSQL_MAX_IDLE_CONNS | Integer value of max idle mysql connections to keep open | 10 | No |
| MYSQL_MAX_OPEN_CONNS | Integer value of max mysql connections to open | 100 | No |
| MYSQL_MAX_CONN_LIFETIME | Integer value of max mysql connection lifetime (in seconds) | 3600 | No |

Here is an example of running Tavern locally with a MySQL backend:

```sh
MYSQL_USER="admin" MYSQL_ADDR="127.0.0.1:3306" go run ./tavern
```

When no value is set for `MYSQL_ADDR`, the default SQLite backend is used:

```sh
MYSQL_USER="admin" go run ./tavern/
2022/03/08 05:46:06 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
```

### OAuth

By default, user authentication is disabled for Tavern. This means that anyone can login and be granted a session. To restrict who is able to access your deployment, Tavern supports OAuth configuration (using [Google OAuth](https://developers.google.com/identity/protocols/oauth2)).

To obtain a client_id and a client_secret for Google OAuth, please follow [their instructions](https://developers.google.com/identity/sign-in/web/sign-in#create_authorization_credentials) to create an application.

The following environment variables are required for OAuth Configuration:

| Env Var | Description |
| ------- | ----------- |
| OAUTH_CLIENT_ID | The [OAuth client_id](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/) Tavern will use to communicate with an identity provider (Google) |
| OAUTH_CLIENT_SECRET | The [OAuth client_secret](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/) Tavern will use to authenticate to an identity provider (Google)
| OAUTH_DOMAIN | The domain Tavern is being hosted at, that the identity provider (Google) should redirect users to after completing the consent flow |

Here is an example of running Tavern locally with OAuth configured:

```sh
OAUTH_CLIENT_ID=123 OAUTH_CLIENT_SECRET=456 OAUTH_DOMAIN=127.0.0.1 go run ./tavern
2022/03/09 05:32:58 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
2022/03/09 05:32:58 listening on 0.0.0.0:80
```

When no OAuth configuration is provided, authentication is disabled:

```sh
go run ./tavern
2022/03/09 05:24:43 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
2022/03/09 05:24:43 WARNING: OAuth is not configured, authentication disabled
```

When partial OAuth configuration is provided, Tavern will error. This is to protect against inadvertently starting Tavern with authentication disabled.

```sh
OAUTH_CLIENT_ID=123 go run ./tavern
2022/03/09 05:31:46 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
2022/03/09 05:31:46 [FATAL] To configure OAuth, must provide value for environment var 'OAUTH_CLIENT_SECRET'
exit status 1
```

### Test Data

Running Tavern with the `ENABLE_TEST_DATA` environment variable set will populate the database with test data. This is useful for UI development, testing, or just interacting with Tavern and seeing how it works.

```sh
ENABLE_TEST_DATA=1 go run ./tavern
2023/02/24 01:02:37 [WARN] MySQL is not configured, using SQLite
2023/02/24 01:02:37 [WARN] OAuth is not configured, authentication disabled
2023/02/24 01:02:37 [WARN] Test data is enabled
2023/02/24 01:02:37 Starting HTTP server on 0.0.0.0:80
```

### PPROF

Running Tavern with the `ENABLE_PPROF` environment variable set will enable performance profiling information to be collected and accessible. This should never be set for a production deployment as it will be unauthenticated and may provide access to sensitive information, it is intended for development purposes only. Read more on how to use `pprof` with tavern under the [Performance Profiling](#performance-profiling) section of this guide.

#### How it Works

Tavern hosts two endpoints to support OAuth:

* A login handler (`/oauth/login`) which redirects users to Google's OAuth consent flow
  * This endpoint sets a JWT cookie for the user, such that the [OAuth state parameter](https://auth0.com/docs/secure/attack-protection/state-parameters#csrf-attacks) can safely be verified later to prevent against CSRF attacks
  * Currently the keys used to sign and verify JWTs are generated at server start, meaning if the server is restarted while a user is in the middle of an OAuth flow, it will fail and the user will need to restart the flow
* An authorization handler (`/oauth/authorize`) which users are redirected to by Google after completing Google's OAuth consent flow
  * This handler is responsible for obtaining a user's profile information from Google using an OAuth access token, and creates the user's account if it does not exist yet

##### Trust on First Use

Tavern supports a Trust on First Use (TOFU) authentication model, meaning the first user to successfully authenticate will be granted admin permissions. Subsequent users that login will have accounts created, but will require activation before they can interact with any Tavern APIs. Only admin users may activate other users.

## Build and publish tavern container

If you want to deploy tavern without using the published version you'll have to build and publish your own container.

### Build your container

```bash
cd ./realm
docker build --tag tavern:dev --file ./docker/tavern.Dockerfile .
```

### Publish your container to docker hub

If you haven't before [sign-up for a docker hub account](https://hub.docker.com/signup) and login with the CLI `docker login`

```bash
docker tag tavern:dev <YOUR_DOCKER_HUB_USERNAME>/tavern:dev
docker push <YOUR_DOCKER_HUB_USERNAME>/tavern:dev
```

### Specify your container during terraform deploy

```bash
terraform apply -var="gcp_project=<PROJECT_ID>" -var="oauth_client_id=<OAUTH_CLIENT_ID>" -var="oauth_client_secret=<OAUTH_CLIENT_SECRET>" -var="oauth_domain=<OAUTH_DOMAIN>" -var="tavern_container_image=<YOUR_DOCKER_HUB_USERNAME>/tavern:dev"
```

## User Interface

## CDN API

### Uploading Files

* File uploads require 2 form parameters: `fileName` and `fileContent`
* A successful response returns JSON with the following content: `{"data":{"file":{"id":<FILE_ID>}}}`

### Downloading Files

* TODO (CDN is not yet added to Tavern)

## GraphQL API

### Playground

If you'd like to explore the Graph API and try out some queries, head to the `/graphiql` endpoint of your Tavern deployment. This endpoint exposes an interactive playground for you to experiment with GraphQL queries.

![/assets/img/tavern/graphiql.png](/assets/img/tavern/graphiql.png)

#### Some sample queries to get started

##### List all beacons

```graphql
query get_beacons {
  beacons {
    id
    identifier
    name
    hostPlatform
    hostIdentifier
    hostPrimaryIP
  }
}
```

##### Create a tome

```graphql
mutation CreateTome ($input: CreateTomeInput!) {
  createTome(input: $input) {
    id
    name
  }
}
```

```json
{
  "input": {
    "name": "Test tome",
    "description": "Just a sample",
    "eldritch": "print(input_params['print_string'])",
    "fileIDs": [],
    "paramDefs": "[{\"name\":\"print_string\",\"label\":\"Print String\",\"type\":\"string\",\"placeholder\":\"A message to print\"}]"
  }
}
```

##### Create a task

```graphql
mutation createQuest($input: CreateQuestInput!, $beaconIDs:[ID!]!){
  createQuest(input: $input, beaconIDs: $beaconIDs) {
    id
  }
}
```

```json
{
  "input": {
    "name": "Run test tome",
    "tomeID": "21474836488",
    "parameters": "{\"print_string\":\"Hello World\"}"
  },
  "sess": ["8589934593"]
}
```

##### Get all task and quest output

```graphql
query get_task_res {
  quests {
    tasks {
      id
      output
      quest {
        tome {
          eldritch
        }
        parameters
      }
      execFinishedAt
    }
  }
}
```

### Creating a New Model

1. Initialize the schema `cd tavern && go run entgo.io/ent/cmd/ent init <NAME>`
2. Update the generated file in `tavern/internal/ent/schema/<NAME>.go`
3. Ensure you include a `func (<NAME>) Annotations() []schema.Annotation` method which returns a `entgql.QueryField()` annotation to tell entgo to generate a GraphQL root query for this model (if you'd like it to be queryable from the root query)
4. Update `tavern/internal/graphql/gqlgen.yml` to include the ent types in the `autobind:` section (e.g.`- realm.pub/tavern/internal/ent/<NAME>`)
5. **Optionally** update the `models:` section of `tavern/internal/graphql/gqlgen.yml` to bind any GraphQL enum types to their respective `entgo` generated types (e.g. `realm.pub/tavern/internal/ent/<NAME>.<ENUM_FIELD>`)
6. Run `go generate ./tavern/...` from the project root
7. If you added an annotation for a root query field (see above), you will notice auto-generated the `query.resolvers.go` file has been updated with new methods to query your model (e.g. `func (r *queryResolver) <NAME>s ...`)
    * This must be implemented (e.g. `return r.client.<NAME>.Query().All(ctx)` where NAME is the name of your model)

### Adding Mutations

1. Update the `mutation.graphql` schema file to include your new mutation and please include it in the section for the model it's mutating if applicable (e.g. createUser should be defined near all the related User mutations)
    * **Note:** Input types such as `Create<NAME>Input` or `Update<NAME>Input` will already be generated if you [added the appropriate annotations to your ent schema](https://entgo.io/docs/tutorial-todo-gql#install-and-configure-entgql). If you require custom input mutations (e.g. `ClaimTasksInput`) then add them to the `inputs.graphql` file (Golang code will be generated in tavern/internal/graphql/models e.g. `models.ClaimTasksInput`).
2. Run `go generate ./...`
3. Implement generated the generated mutation resolver method in `tavern/internal/graphql/mutation.resolvers.go`
    * Depending on the mutation you're trying to implement, a one liner such as `return r.client.<NAME>.Create().SetInput(input).Save(ctx)` might be sufficient
4. Please write a unit test for your new mutation by defining YAML test cases in a new `testdata/mutations` subdirectory with your mutations name (e.g. `tavern/internal/graphql/testdata/mutations/mymutation/SomeTest.yml`)

### Code Generation Reference

* After making a change, remember to run `go generate ./...` from the project root.
* `tavern/internal/ent/schema` is a directory which defines our graph using database models (ents) and the relations between them
* `tavern/generate.go` is responsible for generating ents defined by the ent schema as well as updating the GraphQL schema and generating related code
* `tavern/internal/ent/entc.go` is responsible for generating code for the entgo <-> 99designs/gqlgen GraphQL integration
* `tavern/internal/graphql/schema/mutation.graphql` defines all mutations supported by our API
* `tavern/internal/graphql/schema/query.graphql` is a GraphQL schema automatically generated by ent, providing useful types derived from our ent schemas as well as root-level queries defined by entgo annotations
* `tavern/internal/graphql/schema/scalars.graphql` defines scalar GraphQL types that can be used to help with Go bindings (See [gqlgen docs](https://gqlgen.com/reference/scalars/) for more info)
* `tavern/internal/graphql/schema/inputs.graphql` defines custom GraphQL inputs that can be used with your mutations (e.g. outside of the default auto-generated CRUD inputs)

### YAML Test Reference (GraphQL)

|Field|Description|Required|
|-----|-----------|--------|
|state| SQL queries that define the initial db state before the query is run.| no |
|requestor| Holds information about the authenticated context making the query. | no |
|requestor.beacon_token| Session token corresponding to the user for authentication. You may create a user with a predetermined session token using the `state` field. | no |
|query| GraphQL query or mutation to be executed | yes |
|variables| A map of variables that will be passed with your GraphQL Query to the server | no |
|expected| A map that defines the expected response that the server should return | no |
|expected_error| An expected message that should be included in the query when it fails | no |

### Resources

* [Relay Documentation](https://relay.dev/graphql/connections.htm)
* [entgo.io GraphQL Integration Docs](https://entgo.io/docs/graphql)
* [Ent + GraphQL Tutorial](https://entgo.io/docs/tutorial-todo-gql)
* [Example Ent + GraphQL project](https://github.com/ent/contrib/tree/master/entgql/internal/todo)
* [GQLGen Repo](https://github.com/99designs/gqlgen)

## GRPC API

Tavern also supports a gRPC API for agents to claim tasks and report execution output. This API is defined by our c2.proto spec and is still under active development.

### YAML Test Reference (gRPC)

Still under development.

## Performance Profiling

Tavern supports built in performance monitoring and debugging via the Golang [pprof tool](https://go.dev/blog/pprof) developed by Google. To run tavern with profiling enabled, ensure the `ENABLE_PPROF=1` environment variable is set.

### Install Graphviz

Ensure you have an updated version of [Graphviz](https://graphviz.org/about/) installed for visualizing profile outputs.

```bash
apt install -y graphviz
```

### Collect a Profile

1. Start Tavern with profiling enabled: `ENABLE_PPROF=1 go run ./tavern`.
2. Collect a Profile in desired format (e.g. png): `go tool pprof -png -seconds=10 http://127.0.0.1:80/debug/pprof/allocs?seconds=10 > .pprof/allocs.png`
    a. Replace "allocs" with the [name of the profile](https://pkg.go.dev/runtime/pprof#Profile) to collect.
    b. Replace the value of seconds with the amount of time you need to reproduce performance issues.
    c. Read more about the available profiling URL parameters [here](https://pkg.go.dev/net/http/pprof#hdr-Parameters).
    d. `go tool pprof` does not need to run on the same host as Tavern, just ensure you provide the correct HTTP url in the command. Note that Graphviz must be installed on the system you're running `pprof` from.
3. Reproduce any interactions with Tavern that you'd like to collect profiling information for.
4. A graph visualization of the requested performance profile should now be saved locally, take a look and see what's going on 🕵️.

## Agent Development

Tavern provides an HTTP(s) GraphQL API that agents may use directly to claim tasks and submit execution results. This is the standard request flow, and is supported as a core function of realm. To learn more about how to interface with GraphQL APIs, please read [this documentation](https://www.graphql.com/tutorials/#clients) or read on for a simple example.

![/assets/img/tavern/standard-usage-arch.png](/assets/img/tavern/standard-usage-arch.png)

This however restricts the available transport methods the agent may use to communicate with the teamserver e.g. only HTTP(s). If you wish to develop an agent using a different transport method (e.g. DNS), your development will need to include a C2. The role of the C2 is to handle agent communication, and translate the chosen transport method into HTTP(s) requests to Tavern's GraphQL API. This enables developers to use any transport mechanism with Tavern. If you plan to build a C2 for a common protocol for use with Tavern, consider [submitting a PR](https://realm.pub/pulls).

![/assets/img/tavern/custom-usage-arch.png](/assets/img/tavern/custom-usage-arch.png)

### GraphQL Example

GraphQL mutations enable clients to _mutate_ or modify backend data. Tavern supports a variety of different mutations for interacting with the graph ([see schema](https://realm.pub/blob/main/tavern/internal/graphql/schema/mutation.graphql)). The two mutations agents rely on are `claimTasks` and `submitTaskResult` (covered in more detail below). GraphQL requests are submitted as HTTP POST requests to Tavern, with a JSON body including the GraphQL mutation. Below is an example JSON body that may be sent to the Tavern GraphQL API:

```json
{
  "query": "mutation ClaimTasks($input: ClaimTasksInput!) {\n  claimTasks(input: $input) {\n    id\n  }\n}",
  "variables": {
    "input": {
      "principal": "root",
      "hostname": "test",
      "hostIdentifier": "dodo",
      "agentIdentifier": "bleep",
      "beaconIdentifier": "123"
    }
  },
  "operationName": "ClaimTasks"
}
```

In the above example, `$input` is used to pass variables from code to the GraphQL mutation while avoiding sketchy string parsing. Fields that should be present in the output are included in the body of the query (e.g. 'id').

### Claiming Tasks

The first GraphQL mutation an agent should utilize is `claimTasks`. This mutation is used to fetch new tasks from Tavern that should be executed by the agent. In order to fetch execution information, the agent should perform a graph traversal to obtain information about the associated quest. For example:

```graphql
mutation ClaimTasks($input: ClaimTasksInput!) {
  claimTasks(input: $input) {
    id
    quest {
      tome {
        id
        eldritch
      }
      bundle {
        id
      }
    }
  }
}
```

If the mutation returns a bundle, it should be fetched from the CDN to provide necessary assets (a tar.gz) for eldritch execution. The Task ID should be saved for later reporting results of task execution.

## Submitting Results

After task execution has been completed (or as it is being completed), an agent should utilize the `submitTaskResult` mutation to update Tavern with execution output and status information. When task execution is finished, the agent should provide a value for the `execFinishedAt` parameter. If a task fails to complete, the agent should provide a value for the `error` parameter.
