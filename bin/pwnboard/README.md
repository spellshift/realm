# PWNBoard Integration

This script is meant to use the Tavern GraphQL API to periodially update a PWNBoard instance with host access.

## Usage

Set your `auth-session` cookie from the tavern app to the environment variable `TAVERN_AUTH_SESSION`

`python3 main.py https://tavern.yourdomain.here https://pwnboard.yourdomain.here`

**Note**: Routes will be appended by program. Do not append `/graphql` and `/pwn/boxaccess` in the arguments.

Other flags:

`-i, --interval` - Set interval (in seconds) that the API will be queried. Default=5

`-t, --timediff` - Set max time (in seconds) since now that will be used to determine if a host is still responding. Default=5

`-n, --name` - Set name used for application on PWNBoard. Default="Realm"