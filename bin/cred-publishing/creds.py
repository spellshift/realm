import argparse
import os
import re
import requests
from pprint import pprint
import json
from dataclasses import dataclass


@dataclass
class CredPost:
    graphql_url: str
    auth_session: str
    known_hosts: dict

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
            print(f"Error {response.status_code}: {response.text}")
            return None

    def get_host_ids_by_ip(self, ip: str):
        graphql_query = """
query getHosts($where:HostWhereInput){
    hosts(where:$where) {
        id
    }
}    """

        graphql_variables = {
            "where": {
                "primaryIP": ip
            }
        }
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            return -1
        else:
            return res

    def get_host_ids_by_group(self, group: str):
        graphql_query = """
query getHosts($where:HostWhereInput){
    hosts(where:$where) {
        id
    }
}    """

        graphql_variables = {
            "where": {
                "hasTagsWith": {
                    "name": group
                }
            }
        }
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            return -1
        else:
            return res

    def create_cred(self, principal: str, secret: str, kind: str, host_id: int):
        print(host_id)
        graphql_query = """
mutation CreateCreds($input:CreateHostCredentialInput!) {
  createCredential(input:$input) {
    principal
    secret
    kind
    host {
      id
    }
    task {
      id
    }
  }
}
"""
        graphql_variables = {
            "input": {
                "principal": principal,
                "secret": secret,
                "kind": kind,
                "hostID": host_id
            }
        }
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if res is None:
            print("Error res is none")
            return -1
        if 'errors' in res:
            pprint(res)
            return -1
        else:
            pprint(res)
            return res['data']['createCredential']['host']['id']

    def add_hosts(self, tag_id: str, hosts: list):
        graphql_query = """
mutation updateTag($input:UpdateTagInput!, $tagid:ID!){
    updateTag(input:$input, tagID:$tagid) {
        id
    }
}    """
        print(hosts)
        graphql_variables = {"input": {"addHostIDs": hosts}, "tagid": tag_id}
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            pprint(res)
            return -1
        else:
            return res['data']['updateTag']['id']

    def get_host_ids(self, team: str, selector: str) -> int:
        if selector in self.known_hosts:
            print("Cached")
            return self.known_hosts[selector]

        res = {}
        if not team:
            res = self.get_host_ids_by_ip(selector)
        else:
            res = self.get_host_ids_by_group(selector)

        self.known_hosts[selector] = [host['id']
                                      for host in res['data']['hosts']]
        return self.known_hosts[selector]

    def run(self, team: bool, log_file: str):
        with open(log_file) as file:
            for line in file:
                line_arr = line.strip().split(":")
                print(line_arr)
                unique = line_arr[0]
                user = line_arr[1]
                password = line_arr[2]
                for host_id in self.get_host_ids(team, unique):
                    self.create_cred(
                        user,
                        password,
                        "KIND_PASSWORD",
                        host_id
                    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="CCDC Cred post",
        description="Parse cred logs in form `ip:user:password` and post them to tavern",
    )

    parser.add_argument("tavern_url")
    parser.add_argument("log_file")
    parser.add_argument("--team",
                        action=argparse.BooleanOptionalAction,
                        help="Switch to use team number instead of IP address `team01:username:password`")

    args = parser.parse_args()

    auth_session = os.environ.get("TAVERN_AUTH_SESSION")

    if auth_session is None:
        print(
            "No auth-session cookie found. Please set it using the environment variable TAVERN_AUTH_SESSION"
        )
        exit(1)

    graphql_url = f"{args.tavern_url}/graphql"
    poster = CredPost(graphql_url, auth_session, {})
    poster.run(args.team, args.log_file)
