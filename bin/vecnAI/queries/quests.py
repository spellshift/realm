from queries.common import make_graphql_request, handle_error


def get_quests(platform: str) -> str:
    """Returns a list of all quest and task that's been run. Quests are the main actions run against hosts / beacons.
    These tasks will give users information about the environment and allow users to perform actions.
    If data about a host is asked for the

    The data returned includes:
    id: ID!
    createdAt: Time!
    Timestamp of when this ent was created

    lastModifiedAt: Time!
    Timestamp of when this ent was last updated

    name: String!
    Name of the quest

    parameters: String
    Value of parameters that were specified for the quest (as a JSON string).

    paramDefsAtCreation: String
    JSON string describing what parameters are used with the tome at the time of this quest creation. Requires a list of JSON objects, one for each parameter.

    eldritchAtCreation: String
    Eldritch script that was evaluated at the time of this quest creation.

    creator: User
    User that created the quest if available.

    creator.id: Int
    Creators unique ID number

    creator.name: String
    Creators name

    tasks.id: int
    The unique ID of the unique task run through the quest

    tasks.ouput: string
    The actual output of the task where answers to quest output can be found

    tasks.beacon.id: Int
    The unique ID of the beacon the task ran against

    tasks.beacon.name: String
    The unique human readable name of the beacon the task ran against

    tasks.beacon.principal: String
    The username that the beacon process is running as beacons with elevated privileges may be root, Administrator, or SYSTEM


    tasks.beacon.host.id: Int
    The unique ID of the host that the task ran against

    tasks.beacon.host.name: String
    The hostname of the host the quest / task ran on

    tasks.beacon.host.platform: String
    The operating system running on the host

    tasks.beacon.host.primaryIp: String
    The primary IP address of the host the quest / task ran on

    totalCount: int
    The total number of quests
    """
    graphql_query = """
query getQuestData($where:QuestWhereInput){
  quests(where:$where) {
    edges {
      node {
        id
        createdAt
        lastModifiedAt
        name
        parameters
        paramDefsAtCreation
        eldritchAtCreation
        creator {
          id
          name
        }
        tasks {
          edges {
            node {
              id
              output
              beacon {
                id
                name
                principal
                host {
                  id
                  name
                  platform
                  primaryIP
                }
              }
            }
          }
        }
      }
    }
    totalCount
  }
}   """

    graphql_variables = {"where": {}}
    res = make_graphql_request(graphql_query, graphql_variables)
    return handle_error(res)
