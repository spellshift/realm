from google.genai import types
from queries.common import make_graphql_request, handle_error
from typing import Any, Dict, List
from datetime import datetime, timedelta, timezone


def get_beacons(online: bool) -> str:
    """Searches the known beacons using different parameters to search.
    These parameters are not required and leaving them as `None`, False, or empty "" / []
    will result in all hosts being returned. If unsure grab all hosts.

    Args:
      online: optional, aditive, When set to true only query for online hosts as determined by hosts seen in the last two minutes. Default to false.

    Responses Include:
    id: ID!
    createdAt: Time!
    Timestamp of when this ent was created

    lastModifiedAt: Time!
    Timestamp of when this ent was last updated

    name: String
    A human readable identifier for the host.

    principal: String
    The identity the beacon is authenticated as (e.g. 'root')

    identifier: String!
    Unique identifier for the beacon. Unique to each instance of the beacon.

    agentIdentifier: String
    Identifies the agent that the beacon is running as (e.g. 'imix').

    lastSeenAt: Time
    Timestamp of when a task was last claimed or updated for the beacon.

    host: Host!
    Host this beacon is running on.

    host.id: ID
    The unique ID of the host the beacon runs on

    host.identifier: String!
    Unique identifier for the host the beacon is running on. Unique to each host.

    host.name: String
    A human readable identifier for the host.

    host.primaryIP: String
    Primary interface IP address reported by the agent.

    host.platform: HostPlatform!
    Platform the agent is operating on.

    host.lastSeenAt: Time
    Timestamp of when a task was last claimed or updated for the host.

    host.tags: [Tag!]
    Tags used to group this host with other hosts.

    host.tags.name: String
    Name of tags associate to the host the beacon runs on


    """
    graphql_query = """
query getBeacons($where:BeaconWhereInput){
    beacons(where:$where) {
        id
        createdAt
        lastModifiedAt
        identifier
        name
        principal
        agentIdentifier
        lastSeenAt
        host {
        id
        identifier
        name
        primaryIP
        platform
        lastSeenAt
            tags {
                name
            }
        }
    }
}    """

    graphql_variables = {
        "where": {
        }
    }

    if online:
        if 'and' not in graphql_variables['where']:
            graphql_variables['where']['and'] = {}

        two_minutes_ago = (datetime.now(timezone.utc) -
                           timedelta(minutes=2)).strftime("%Y-%m-%dT%H:%M:%SZ")
        graphql_variables["where"]["and"]["lastSeenAtGTE"] = two_minutes_ago

    print(graphql_variables)

    res = make_graphql_request(graphql_query, graphql_variables)
    return handle_error(res)
