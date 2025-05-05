import argparse
import os
import re
import requests
from pprint import pprint
import json
from neccdc_mapping import tag_profiles
from dataclasses import dataclass


@dataclass
class TagBuilder:
    graphql_url: str
    auth_session: str
    auth_session: str

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

    def get_hosts(self):
        graphql_query = """
query getHosts($where:HostWhereInput){
    hosts(where:$where) {
        id
        primaryIP
        name
    }
}    """

        graphql_variables = {"where": {}}
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            return -1
        else:
            return res

    def get_tag(self, tag_name):
        graphql_query = """
query getTag($input:TagWhereInput){
    tags(where:$input) {
        id
    }
}    """

        graphql_variables = {"input": {"name": tag_name}}
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            return -1
        else:
            if len(res['data']['tags']) > 0:
                return res['data']['tags'][0]['id']
            else:
                return -1

    def create_tag(self, tag_name: str, tag_kind: str):
        print(f"{tag_name}:{tag_kind}")
        res = self.get_tag(tag_name)
        if res != -1:
            return res

        graphql_query = """
mutation createTag($input:CreateTagInput!){
    createTag(input:$input) {
        id
    }
}
        """

        graphql_variables = {"input": {"name": tag_name, "kind": tag_kind}}
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            pprint(res)
            return -1
        else:
            return res['data']['createTag']['id']

    def create_tag(self, tag_name: str, tag_kind: str):
        print(f"{tag_name}:{tag_kind}")
        res = self.get_tag(tag_name)
        if res != -1:
            return res

        graphql_query = """
mutation createTag($input:CreateTagInput!){
    createTag(input:$input) {
        id
    }
}
        """

        graphql_variables = {"input": {"name": tag_name, "kind": tag_kind}}
        res = self.make_graphql_request(graphql_query, graphql_variables)
        if 'errors' in res:
            pprint(res)
            return -1
        else:
            return res['data']['createTag']['id']

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

    def run(self):
        service_map = {}
        data = self.get_hosts()

        for tag_profile in tag_profiles:
            tag_id = self.create_tag(
                tag_profile['name'],
                tag_profile['kind']
            )
            service_map[tag_profile['name']] = {
                "hosts": [],
                "id": tag_id,
            }

        for row in data["data"]["hosts"]:
            for tag_profile in tag_profiles:
                re_match = None
                if 'ip_regex' in tag_profile:
                    re_match = re.search(
                        tag_profile["ip_regex"], row["primaryIP"])
                if 'hostname_regex' in tag_profile:
                    re_match = re.search(
                        tag_profile["hostname_regex"], row["name"])
                if re_match is not None:
                    self.add_hosts(
                        service_map[tag_profile['name']]['id'],
                        [row["id"]]
                    )
                    service_map[tag_profile['name']]['hosts'].append(row["id"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="CCDC Tag generator",
        description="Based on a list of regex filters create tags",
    )

    parser.add_argument("tavern_url")

    args = parser.parse_args()

    auth_session = os.environ.get("TAVERN_AUTH_SESSION")

    if auth_session is None:
        print(
            "No auth-session cookie found. Please set it using the environment variable TAVERN_AUTH_SESSION"
        )
        exit(1)

    graphql_url = f"{args.tavern_url}/graphql"
    poster = TagBuilder(graphql_url, auth_session)
    poster.run()
