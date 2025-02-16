import os
import requests

graphql_url = os.getenv("TAVERN_URL")
auth_session = os.getenv("TAVERN_AUTH_SESSION")


def make_graphql_request(query, variables):
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json",
    }

    data = {"query": query, "variables": variables}
    cookies = {
        "auth-session": auth_session,
    }

    response = requests.post(
        graphql_url, json=data, headers=headers, cookies=cookies)
    if response.status_code == 200:
        return response.json()
    else:
        print(f"Error {response.status_code}: {response.text}")
        return None


def handle_error(res):
    if 'errors' in res:
        print(f"ERROR: {res}")
        return -1
    else:
        return res
