#!/usr/bin/env python3
"""
Tavern MCP Server

A Model Context Protocol server that provides tools for interacting with Tavern.
"""

import asyncio
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent
import os
import requests

app = Server("tavern-mcp")

QUERIES = {
    "list_quests": """
query listquests {
    quests {
        edges {
            node {
                id
                createdAt
                name
                creator {
                    name
                }
                parameters
                tome {
                    name
                    tactic
                    description
                }
            }
        }
    }
}
""",
    "quest_output": """
query questOutput($ids: [ID!]!) {
  quests(where: {
    idIn:$ids
  }) {
    edges {
      node {
        id
        name
        tasks {
          edges {
            node {
              output
              beacon {
                id
                name
                host {
                  id
                  name
                }
              }
            }
          }
        }
      }
    }
  }
}
""",
    "list_tomes": """
query listtomes{
    tomes {
      edges {
        node {
          id
          name
          tactic
          description
          paramDefs
        }
      }
    }
}
""",
    "create_quest": """
mutation createQuest($beaconIds: [ID!]!, $name: String! $json_params:String!, $tomeId: ID!) {
  createQuest(beaconIDs: $beaconIds, input:{
    name:$name,
    parameters: $json_params
    tomeID:$tomeId
  }) {
    id
  }
}
""",
    "list_hosts": """
query gethosts {
  hosts {
    edges {
      node {
        id
        identifier
        name
        platform
        primaryIP
        lastSeenAt
        externalIP
        tags {
          edges {
            node {
              id
              name
              kind
            }
          }
        }
        beacons {
          edges {
            node {
              id
              name
              principal
            }
          }
        }
      }
    }
  }
}
""",
    "quest_tasks_status": """
query questTasksStatus($id: ID!) {
  quests(where: {id: $id}) {
    edges {
      node {
        id
        name
        tasks {
          edges {
            node {
              id
              execFinishedAt
              error
              beacon {
                id
                name
              }
            }
          }
        }
      }
    }
  }
}
""",
}


def make_graphql_request(api_url, query, variables):
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json",
    }

    data = {"query": query, "variables": variables}
    response = requests.post(
        api_url, json=data, headers=headers, cookies={
            "auth-session": ENV["TAVERN_AUTH_SESSION"],
        }
    )
    if response.status_code == 200:
        return response.json()
    else:
        return f"Error {response.status_code}: {response.text}"


@app.list_tools()
async def list_tools() -> list[Tool]:
    """List available tools."""
    return [
        Tool(
            name="create_quest",
            description="Create a new quest in Tavern",
            inputSchema={
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the quest"
                    },
                    "beacon_ids": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of beacon IDs to target"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Quest parameters as a JSON object"
                    },
                    "tome_id": {
                        "type": "string",
                        "description": "The ID of the tome to use for this quest"
                    }
                },
                "required": ["name", "beacon_ids", "parameters", "tome_id"]
            }
        ),
        Tool(
            name="list_quests",
            description="List all available quests in Tavern",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        ),
        Tool(
            name="list_tomes",
            description="List all available tomes in Tavern and their required parameters.",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        ),
        Tool(
            name="quest_output",
            description="Get the output of quests by their IDs",
            inputSchema={
                "type": "object",
                "properties": {
                    "ids": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of quest IDs"
                    }
                },
                "required": ["ids"]
            }
        ),
        Tool(
            name="list_hosts",
            description="List all available hosts in Tavern",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        ),
        Tool(
            name="wait_for_quest",
            description="Wait for all tasks in a quest to finish. Polls every 5 seconds for up to 10 minutes.",
            inputSchema={
                "type": "object",
                "properties": {
                    "quest_id": {
                        "type": "string",
                        "description": "The ID of the quest to wait for"
                    }
                },
                "required": ["quest_id"]
            }
        ),
    ]

@app.call_tool()
async def call_tool(name: str, arguments: dict) -> list[TextContent]:
    """Handle tool calls."""

    if name == "create_quest":
        import json

        quest_name = arguments.get("name")
        beacon_ids = arguments.get("beacon_ids", [])
        parameters = arguments.get("parameters", {})
        tome_id = arguments.get("tome_id")

        # Convert parameters to JSON string
        json_params = json.dumps(parameters)

        result = make_graphql_request(
            f"{ENV['TAVERN_URL']}/graphql",
            QUERIES["create_quest"],
            {
                "name": quest_name,
                "beaconIds": beacon_ids,
                "json_params": json_params,
                "tomeId": tome_id
            }
        )

        return [TextContent(type="text", text=str(result))]

    elif name == "list_quests":
        # TODO: Implement actual Tavern API call
        result = make_graphql_request(
            f"{ENV['TAVERN_URL']}/graphql",
            QUERIES["list_quests"],
            {}
        )

        return [TextContent(type="text", text=str(result))]

    elif name == "quest_output":
        quest_ids = arguments.get("ids", [])
        result = make_graphql_request(
            f"{ENV['TAVERN_URL']}/graphql",
            QUERIES["quest_output"],
            {"ids": quest_ids}
        )

        return [TextContent(type="text", text=str(result))]

    elif name == "list_tomes":
        # TODO: Implement actual Tavern API call
        result = make_graphql_request(
            f"{ENV['TAVERN_URL']}/graphql",
            QUERIES["list_tomes"],
            {}
        )
        return [TextContent(type="text", text=str(result))]

    elif name == "list_hosts":
        result = make_graphql_request(
            f"{ENV['TAVERN_URL']}/graphql",
            QUERIES["list_hosts"],
            {}
        )
        return [TextContent(type="text", text=str(result))]

    elif name == "wait_for_quest":
        import json

        quest_id = arguments.get("quest_id")
        poll_interval = 5  # seconds
        timeout = 600  # 10 minutes in seconds
        elapsed = 0
        pending_count = 0

        while elapsed < timeout:
            response = make_graphql_request(
                f"{ENV['TAVERN_URL']}/graphql",
                QUERIES["quest_tasks_status"],
                {"id": quest_id}
            )

            if isinstance(response, str):
                return [TextContent(type="text", text=response)]

            quests = response.get("data", {}).get("quests", {}).get("edges", [])

            if not quests:
                return [TextContent(type="text", text=f"Quest with ID {quest_id} not found")]

            quest = quests[0]["node"]
            tasks = quest.get("tasks", {}).get("edges", [])

            if not tasks:
                return [TextContent(type="text", text=f"Quest {quest_id} has no tasks")]

            all_finished = all(
                task["node"].get("execFinishedAt") is not None
                for task in tasks
            )

            if all_finished:
                finished_tasks = []
                for task in tasks:
                    node = task["node"]
                    finished_tasks.append({
                        "task_id": node.get("id"),
                        "beacon_id": node.get("beacon", {}).get("id"),
                        "beacon_name": node.get("beacon", {}).get("name"),
                        "finished_at": node.get("execFinishedAt"),
                        "error": node.get("error")
                    })

                return [TextContent(
                    type="text",
                    text=f"All {len(tasks)} tasks in quest '{quest.get('name')}' have finished.\n\nTask details:\n{json.dumps(finished_tasks, indent=2)}"
                )]

            # Not all tasks finished, wait and poll again
            pending_count = sum(
                1 for task in tasks
                if task["node"].get("execFinishedAt") is None
            )
            await asyncio.sleep(poll_interval)
            elapsed += poll_interval

        # Timeout reached
        return [TextContent(
            type="text",
            text=f"Timeout: Not all tasks in quest {quest_id} finished within 10 minutes. {pending_count} tasks still pending."
        )]

    else:
        return [TextContent(type="text", text=f"Unknown tool: {name}")]


ENV = {}


async def main():
    """Run the MCP server."""
    ENV['TAVERN_AUTH_SESSION'] = os.environ.get('TAVERN_AUTH_SESSION', '')
    ENV['TAVERN_URL'] = os.environ.get('TAVERN_URL', '')
    async with stdio_server() as (read_stream, write_stream):
        await app.run(
            read_stream,
            write_stream,
            app.create_initialization_options()
        )

if __name__ == "__main__":
    asyncio.run(main())
