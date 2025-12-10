import argparse
from dataclasses import dataclass
import os
from typing import Any
from pprint import pprint
from mcp.server.fastmcp import FastMCP
from mcp.server import Server
import mcp.types as types
from mcp.server.fastmcp import FastMCP
from mcp.server.fastmcp.prompts import base

import requests
import logging

logger = logging.getLogger(__name__)

# Initialize FastMCP server
mcp = FastMCP("tavern")

# Constants
USER_AGENT = "tavern-mcp/1.0"

MCPCLIENT = None


@dataclass
class TavernMCP:
    graphql_url: str
    auth_session: str
    schema: str

    def make_graphql_request(self, query, variables):
        headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
        }

        data = {"query": query, "variables": variables}
        cookies = {
            "auth-session": self.auth_session,
        }

        response = requests.post(
            self.graphql_url, json=data, headers=headers, cookies=cookies)
        if response.status_code == 200:
            return response.json()
        else:
            logging.error(f"Error {response.status_code}: {response.text}")
            return None

    def get_schema(self):
        return self.schema


@mcp.tool()
async def get_schema() -> str:
    """Introspect the graphql Schema

    Return the full graphql schema
    """
    res = MCPCLIENT.get_schema()
    return str(res)


@mcp.tool()
async def query_graphql(query: str, variables: dict) -> str:
    logging.debug(f"query_graphql tool called with {query} {variables}")
    """Query the Tavern graphql API

    Args:
        query: The graphql formatted query string Eg. query getTag($input:TagWhereInput){ tags(where:$input) { id }}
        variables: A dictionary defining the graphql query variables Eg. {"input": {"name": tag_name}}
    """
    res = MCPCLIENT.make_graphql_request(query, variables)
    return str(res)


@mcp.prompt()
def debug_error(error: str) -> list[base.Message]:
    return [
        base.UserMessage("I'm seeing this error:"),
        base.UserMessage(error),
        base.AssistantMessage(
            "I'll help debug that. What have you tried so far?"),
    ]


if __name__ == "__main__":
    logging.basicConfig(format='%(levelname)s:%(message)s',
                        level=logging.INFO)

    parser = argparse.ArgumentParser(
        prog="Tavern MCP server",
        description="An MCP server exposing tools to interact with Tavern c2 server",
    )

    parser.add_argument(
        "--type",
        choices=["stdio", "sse"],
        default="stdio",
        help="The communication protocol the MCP server should use"
    )

    parser.add_argument(
        "--url",
        help="The URL of tavern example: `http://example.com/graphql`"
    )

    parser.add_argument('-s', '--schema',
                        help="Optional path to the schema file")

    args = parser.parse_args()

    auth_session = os.environ.get("TAVERN_AUTH_SESSION")

    if auth_session is None:
        print(
            "No auth-session cookie found. Please set it using the environment variable TAVERN_AUTH_SESSION"
        )
        exit(1)

    graphql_url = f"{args.url}"

    days_file = open(args.schema, 'r')
    graphql_schema = days_file.read()
    MCPCLIENT = TavernMCP(graphql_url, auth_session, graphql_schema)
    mcp.run(transport=args.type)
