from google.genai import types
from queries.common import make_graphql_request, handle_error
from typing import Any, Dict, List
from datetime import datetime, timedelta, timezone


def get_hosts(
        platforms: List[str],
        tag: str,
        primaryIp: str,
        online: bool) -> str:
    """Searches the known hosts using different parameters to search.
    These parameters are not required and leaving them as `None`, False, or empty "" / []
    will result in all hosts being returned. If unsure grab all hosts.

    Args:
      platform: optional, additive, List of operating systems names to query: `linux`, `windows`, `macos`, `bsd`, or `unknown`
      tag: optional, additive, tag name to query hosts by
      primaryIp: optional, additive, The whole or partial IP address of the host
      online: optional, aditive, When set to true only query for online hosts as determined by hosts seen in the last two minutes. Default to false.

    Responses Include:
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

    # Charitably interpret LLM input

    def platform_format(p: str):
        if "linux" in p.lower():
            return "PLATFORM_LINUX"
        if "windows" in p.lower():
            return "PLATFORM_WINDOWS"
        if "bsd" in p.lower():
            return "PLATFORM_BSD"
        if "mac" in p.lower():
            return "PLATFORM_MACOS"
        return "PLATFORM_UNSPECIFIED"

    platforms = [platform_format(p) for p in platforms]

    graphql_variables = {
        "where": {
        }
    }

    if platforms:
        if 'and' not in graphql_variables['where']:
            graphql_variables['where']['and'] = {}
        graphql_variables['where']['and']["platformIn"] = platforms

    if tag:
        if 'and' not in graphql_variables['where']:
            graphql_variables['where']['and'] = {}
        graphql_variables['where']['and']["hasTagsWith"] = [
            {
                "nameContains": tag
            }
        ]

    if primaryIp:
        if 'and' not in graphql_variables['where']:
            graphql_variables['where']['and'] = {}
        graphql_variables["where"]["and"]["primaryIPContains"] = primaryIp

    if online:
        if 'and' not in graphql_variables['where']:
            graphql_variables['where']['and'] = {}

        two_minutes_ago = (datetime.now(timezone.utc) -
                           timedelta(minutes=2)).strftime("%Y-%m-%dT%H:%M:%SZ")
        graphql_variables["where"]["and"]["lastSeenAtGTE"] = two_minutes_ago

    print(graphql_variables)

    res = make_graphql_request(graphql_query, graphql_variables)
    return handle_error(res)
