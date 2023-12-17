import requests
import os
import sys
import argparse
from time import sleep
from datetime import datetime, timedelta


def make_graphql_request(api_url, query, variables, cookies=None):
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json",
    }

    data = {"query": query, "variables": variables}
    response = requests.post(api_url, json=data, headers=headers, cookies=cookies)
    if response.status_code == 200:
        return response.json()
    else:
        print(f"Error {response.status_code}: {response.text}")
        return None


def make_pwnboard_request(api_url, application_name, ips):
    data = {"ip": ips[0], "application": application_name}
    if len(ips) > 1:
        data["ips"] = ips[1:]
    response = requests.post(api_url, json=data)

    if response.status_code == 202:
        return True
    else:
        print(f"Error {response.status_code}: {response.text}")
        return False


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="Tavern PWNBoard Integration",
        description="Queries GraphQL for active beacons to update PWNBoard",
    )

    parser.add_argument("tavern_url")

    parser.add_argument("pwnboard_url")

    parser.add_argument(
        "-n", "--name", default="Realm", help="Custom Application Name on PWNBoard"
    )
    parser.add_argument(
        "-i",
        "--interval",
        default=5,
        help="Callback Interval to PWNBoard (seconds). Default to 5 seconds",
    )
    parser.add_argument(
        "-t",
        "--timediff",
        default=5,
        help="Number of seconds from now to check for a beacon response",
    )

    args = parser.parse_args()

    auth_session = os.environ.get("TAVERN_AUTH_SESSION")

    if auth_session is None:
        print(
            "No auth-session cookie found. Please set it using the environment variable TAVERN_AUTH_SESSION"
        )
        exit(1)

    graphql_query = """
        query pwnboard($input: HostWhereInput) {
            hosts(where: $input) {
                primaryIP
            }
        } 
    """
    cookies = {
        "auth-session": auth_session,
    }

    graphql_url = f"{args.tavern_url}/graphql"

    pwnboard_url = f"{args.pwnboard_url}/pwn/boxaccess"

    while True:
        current_time = datetime.utcnow()

        time_five_minutes_ago = current_time - timedelta(seconds=args.timediff)

        formatted_time = time_five_minutes_ago.strftime("%Y-%m-%dT%H:%M:%SZ")

        graphql_variables = {"input": {"lastSeenAtGT": formatted_time}}

        result = make_graphql_request(
            graphql_url, graphql_query, graphql_variables, cookies
        )

        if result:
            if result["data"] and len(result["data"]["hosts"]) > 0:
                ips = list(map(lambda x: x["primaryIP"], result["data"]["hosts"]))
                make_pwnboard_request(pwnboard_url, args.name, ips)
            else:
                print("No data found :(")
        sleep(args.interval)
