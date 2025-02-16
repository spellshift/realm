from queries.common import make_graphql_request, handle_error


def get_hosts() -> str:
    """Returns a list of all hosts tavern has access to. Including:
    id: ID!
    createdAt: Time!
    Timestamp of when this ent was created

    lastModifiedAt: Time!
    Timestamp of when this ent was last updated

    identifier: String!
    Unique identifier for the host. Unique to each host.

    name: String
    A human readable identifier for the host.

    primaryIP: String
    Primary interface IP address reported by the agent.

    platform: HostPlatform!
    Platform the agent is operating on.

    lastSeenAt: Time
    Timestamp of when a task was last claimed or updated for the host.

    tags: [Tag!]
    Tags used to group this host with other hosts.
    """
    graphql_query = """
query getHosts($where:HostWhereInput){
    hosts(where:$where) {
        id
        createdAt
        lastModifiedAt
        identifier
        name
        primaryIP
        platform
        lastSeenAt
        tags {
            id
            name
            kind
        }
    }
}    """

    graphql_variables = {"where": {}}
    res = make_graphql_request(graphql_query, graphql_variables)
    return handle_error(res)
