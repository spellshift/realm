---
title: Tavern
tags: 
 - Dev Guide
description: Want to contribute to Tavern? Start here!
permalink: dev-guide/tavern
---
# Overview
Tavern is a teamserver for Realm, providing a UI to control deployments and implants during an engagement. The majority of Tavern's functionality is exposed through a GraphQL API, which is used by both implants and the UI.

If you would like to help contribute to Tavern, please take a look at our [open issues](https://github.com/KCarretto/realm/issues?q=is%3Aopen+is%3Aissue+label%3Atavern).

# Configuration
## MySQL
By default, Tavern operates an in-memory SQLite database. To persist data, a MySQL backend is supported. In order to configure Tavern to use MySQL, the `MYSQL_ADDR` environment variable must be set to the `host[:port]` of the database (e.g. `127.0.0.1`, `mydb.com`, or `mydb.com:3306`). You can reference the [mysql.Config](https://pkg.go.dev/github.com/go-sql-driver/mysql#Config) for additional information about Tavern's MySQL configuration. 

The following environment variables are currently supported for additional MySQL Configuration:

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| MYSQL_ADDR| Address of the MySQL server (e.g. `host[:port]`) | N/A | **_Yes_** |
| MYSQL_NET| Network type (e.g. unix) | tcp | No |
| MYSQL_USER| User to authenticate with | root | No |
| MYSQL_PASSWD| Password to authenticate with | no password | No |
| MYSQL_DB| Name of the database to use | tavern | No |

<br/>
Here is an example of running Tavern locally with a MySQL backend:
```
MYSQL_USER="admin" MYSQL_ADDR="127.0.0.1:3306" go run ./tavern
```
<br/>
When no value is set for `MYSQL_ADDR`, the default SQLite backend is used:
```
MYSQL_USER="admin" go run ./tavern/
2022/03/08 05:46:06 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
```

## OAuth
By default, user authentication is disabled for Tavern. This means that anyone can login and be granted a session. To restrict who is able to access your deployment, Tavern supports OAuth configuration (using [Google OAuth](https://developers.google.com/identity/protocols/oauth2)).

To obtain a client_id and a client_secret for Google OAuth, please follow [their instructions](https://developers.google.com/identity/sign-in/web/sign-in#create_authorization_credentials) to create an application.

The following environment variables are required for OAuth Configuration:

| Env Var | Description |
| ------- | ----------- |
| OAUTH_CLIENT_ID | The [OAuth client_id](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/) Tavern will use to communicate with an identity provider (Google) |
| OAUTH_CLIENT_SECRET | The [OAuth client_secret](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/) Tavern will use to authenticate to an identity provider (Google)
| OAUTH_DOMAIN | The domain Tavern is being hosted at, that the identity provider (Google) should redirect users to after completing the consent flow |

<br/>
Here is an example of running Tavern locally with OAuth configured:
```
OAUTH_CLIENT_ID=123 OAUTH_CLIENT_SECRET=456 OAUTH_DOMAIN=127.0.0.1 go run ./tavern
2022/03/09 05:32:58 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
2022/03/09 05:32:58 listening on 0.0.0.0:80
```

<br/>
When no OAuth configuration is provided, authentication is disabled:
```
go run ./tavern
2022/03/09 05:24:43 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
2022/03/09 05:24:43 WARNING: OAuth is not configured, authentication disabled
```

<br/>
When partial OAuth configuration is provided, Tavern will error. This is to protect against inadvertently starting Tavern with authentication disabled.
```
OAUTH_CLIENT_ID=123 go run ./tavern
2022/03/09 05:31:46 no value found for environment var 'MYSQL_ADDR', starting tavern with SQLite
2022/03/09 05:31:46 [FATAL] To configure OAuth, must provide value for environment var 'OAUTH_CLIENT_SECRET'
exit status 1
```

#### How it Works

Tavern hosts two endpoints to support OAuth:
* A login handler (`/oauth/login`) which redirects users to Google's OAuth consent flow
    * This endpoint sets a JWT cookie for the user, such that the [OAuth state parameter](https://auth0.com/docs/secure/attack-protection/state-parameters#csrf-attacks) can safely be verified later to prevent against CSRF attacks
    * Currently the keys used to sign and verify JWTs are generated at server start, meaning if the server is restarted while a user is in the middle of an OAuth flow, it will fail and the user will need to restart the flow
* An authorization handler (`/oauth/authorize`) which users are redirected to by Google after completing Google's OAuth consent flow
    * This handler is responsible for obtaining a user's profile information from Google using an OAuth access token, and creates the user's account if it does not exist yet

##### Trust on First Use
Tavern supports a Trust on First Use (TOFU) authentication model, meaning the first user to successfully authenticate will be granted admin permissions. Subsequent users that login will have accounts created, but will require activation before they can interact with any Tavern APIs. Only admin users may activate other users.

# User Interface

# CDN API

## Uploading Files
* File uploads require 2 form parameters: `fileName` and `fileContent`
* A successful response returns JSON with the following content: `{"data":{"file":{"id":<FILE_ID>}}}`

## Downloading Files
* TODO (CDN is not yet added to Tavern)

# GraphQL API

## Playground
If you'd like to explore the Graph API and try out some queries, head to the `/graphiql` endpoint of your Tavern deployment. This endpoint exposes an interactive playground for you to experiment with GraphQL queries.

![/assets/img/tavern/graphiql.png](/assets/img/tavern/graphiql.png)


## Creating a New Model
1. Initialize the schema `cd tavern && go run entgo.io/ent/cmd/ent init <NAME>`
2. Update the generated file in `tavern/ent/schema/<NAME>.go`
3. Ensure you include a `func (<NAME>) Annotations() []schema.Annotation` method which returns a `entgql.QueryField()` annotation to tell entgo to generate a GraphQL root query for this model (if you'd like it to be queryable from the root query).
4. Update `tavern/graphql/gqlgen.yml` to include the ent types in the `autobind:` section (e.g.`- github.com/kcarretto/realm/tavern/ent/<NAME>`)
5. **Optionally** update the `models:` section of `tavern/graphql/gqlgen.yml` to bind any GraphQL enum types to their respective `entgo` generated types (e.g. `github.com/kcarretto/realm/tavern/ent/<NAME>.<ENUM_FIELD>`)
6. Run `go generate ./tavern/...` from the project root
    * **Note:** There is currently a bug with entgo & gqlgen where you may sometimes get the error `failed to load schema: schema/schema.graphql:15: Undefined type Time.`, this is usually fixed by running `go generate ./tavern/...` again
7. You will notice auto-generated go "resolver" files have been updated with new methods to query your model (e.g. `func (r *queryResolver) <NAME>s ...`)
    * This must be implemented (e.g. `return r.client.<NAME>.Query().All(ctx)` where <NAME> is the name of your model)

## Adding Mutations
1. Create a new GraphQL schema file for your model if it does not yet exist (e.g. `touch ./tavern/graphql/schema/<NAME>.graphql` from the project root)
    * If it did not already exist, update `tavern/graphql/gqlgen.yml` to include this schema under the `schema:` section.
2. Add any mutations to `tavern/graphql/schema/<NAME>.graphql`
    * **Note:** Input types such as `Create<NAME>Input` or `Update<NAME>Input` will already be generated if you [added the approproate annotations to your ent schema](https://entgo.io/docs/tutorial-todo-gql#install-and-configure-entgql)
3. Run `go generate ./...`
4. Implement generated the generated mutation resolver method in `tavern/graphql/<NAME>.resolvers.go`
    * Depending on the mutation you're trying to implement, a one liner such as `return r.client.User.Create().SetInput(input).Save(ctx)` might be sufficient
5. Please write a unit test for your new mutation <3

## Code Generation Reference
* After making a change, remember to run `go generate ./...` from the project root.
* `tavern/ent/schema` is a directory which defines our graph using database models (ents) and the relations between them
* `tavern/generate.go` is responsible for generating ents defined by the ent schema as well as updating the GraphQL schema and generating related code
* `tavern/ent/entc.go` is responsible for generating code for the entgo <-> 99designs/gqlgen GraphQL integration
* `tavern/graphql/schema/schema.graphql` is a GraphQL schema automatically generated by ent, providing useful types derived from our ent schemas
* `tavern/graphql/schema/<NAME>.graphql` defines mutations for the given entity

## Resources
* [Relay Documentation](https://relay.dev/graphql/connections.htm)
* [entgo.io GraphQL Integration Docs](https://entgo.io/docs/graphql)
* [Ent + GraphQL Tutorial](https://entgo.io/docs/tutorial-todo-gql)
* [Example Ent + GraphQL project](https://github.com/ent/contrib/tree/master/entgql/internal/todo)
* [GQLGen Repo](https://github.com/99designs/gqlgen)