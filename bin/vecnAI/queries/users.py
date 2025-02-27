
from queries.common import make_graphql_request, handle_error


def get_users(user_type: str) -> str:
    """Returns a list of all users that have authenticated to tavern. Including:
    id: ID!
    name: String!
    The name displayed for the user

    photoURL: String!
    URL to the user's profile photo.

    isActivated: Boolean!
    True if the user is active and able to authenticate

    isAdmin: Boolean!
    True if the user is an Admin

    tomes: [Tome!]
    Tomes uploaded by the user.

    activeShells: [Shell!]
    Shells actively used by the user


    Args:
        type: required, The user's type "admin", "active", or "any"

    """
    graphql_query = """
query getUsers($where:UserWhereInput){
    users(where:$where) {
        id
        name
        isAdmin
        isActivated
    }
}    """

    graphql_variables = {"where": {}}
    match user_type.lower():
        case "any":
            graphql_variables = {"where": {}}
        case "active":
            graphql_variables = {"where": {"isActivated": True}}
        case "admin":
            graphql_variables = {"where": {"isAdmin": True}}

    res = make_graphql_request(graphql_query, graphql_variables)
    return handle_error(res)
