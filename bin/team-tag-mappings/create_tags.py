"""Team tag generator.

Applies Tavern tags to hosts based on a user-provided YAML mapping file.

Mapping file format: a YAML list of tag rules, evaluated in file order:

    - name: Team01
      kind: group
      where:
        or:
          - primaryIPHasPrefix: 10.1.1.
          - primaryIPHasPrefix: 192.168.1.

Each rule's ``where`` clause is passed directly as the ``where`` argument of
the ``hosts`` GraphQL query, so every key used in ``where`` must be a valid
field of ``HostWhereInput`` and every kind a valid ``TagKind``. Those names
are validated against the live Tavern schema: the GraphQL client fetches the
prod schema on first use and the server rejects anything it does not define.
Loading the mapping file only checks its structure (a list of rules with a
name, kind, and ``where`` mapping); unknown fields fail the rule that uses
them at execution time.

Rules are applied in file order. A host that already carries a tag of the
rule's kind is left alone by later rules, unless ``--overwrite`` is given, in
which case the host is removed from its existing same-kind tags and moved to
the matching tag. Tags of other kinds are never touched: a host with only a
``group`` tag still receives a matching ``service`` tag and vice versa. A host
that already has the rule's tag is always skipped (applying tags is
idempotent).

Tags listed in the mapping file are looked up in Tavern and created when they
do not already exist.
"""

import argparse
import os
import sys

import yaml
from gql import Client, gql
from gql.transport.exceptions import TransportError, TransportQueryError
from gql.transport.requests import RequestsHTTPTransport

# NOTE: valid ``TagKind`` values and ``HostWhereInput`` fields are
# intentionally not listed here. They are validated against the live Tavern
# schema (fetched by the GraphQL client) and by the server at execution time,
# so this script never drifts out of sync with the prod schema.


def validate_where(where, path="where"):
    """Check the structure of a ``where`` clause.

    Field names are not checked here; they are validated against the live
    Tavern schema when the rule executes.
    """
    if not isinstance(where, dict) or not where:
        raise ValueError(f"{path} must be a non-empty mapping")

    for key, value in where.items():
        if key in ("and", "or"):
            if not isinstance(value, list) or not value:
                raise ValueError(f"{path}.{key} must be a non-empty list")
            for i, sub in enumerate(value):
                validate_where(sub, f"{path}.{key}[{i}]")
        elif key == "not":
            validate_where(value, f"{path}.not")


def validate_tag_rule(rule, index):
    """Check the structure of a mapping-file rule, returning a normalized dict.

    The ``kind`` value and ``where`` field names are not checked here; they
    are validated against the live Tavern schema when the rule executes.
    """
    path = f"rules[{index}]"
    if not isinstance(rule, dict):
        raise ValueError(f"{path} must be a mapping")

    name = rule.get("name")
    if not isinstance(name, str) or not name:
        raise ValueError(f"{path} must define a non-empty 'name'")

    kind = rule.get("kind")
    if not isinstance(kind, str) or not kind:
        raise ValueError(f"{path} ({name!r}) must define a non-empty 'kind'")

    if "where" not in rule:
        raise ValueError(f"{path} ({name!r}) must define a 'where' clause")
    validate_where(rule["where"], f"{path} ({name}).where")

    return {"name": name, "kind": kind, "where": rule["where"]}


def load_tag_rules(mapping_path):
    """Load the YAML mapping file and check its structure."""
    with open(mapping_path, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    if not isinstance(data, list) or not data:
        raise ValueError(
            f"{mapping_path} must contain a non-empty YAML list of tag rules"
        )

    return [validate_tag_rule(rule, i) for i, rule in enumerate(data)]


GET_HOSTS = gql("""
query getHosts($where:HostWhereInput){
    hosts(where:$where) {
        edges {
            node {
                id
                tags {
                    edges {
                        node {
                            id
                            name
                            kind
                        }
                    }
                }
            }
        }
    }
}
""")

GET_TAG = gql("""
query getTag($input:TagWhereInput){
    tags(where:$input) {
        edges {
            node {
                id
                name
                kind
            }
        }
    }
}
""")

CREATE_TAG = gql("""
mutation createTag($input:CreateTagInput!){
    createTag(input:$input) {
        id
    }
}
""")

UPDATE_TAG = gql("""
mutation updateTag($input:UpdateTagInput!, $tagid:ID!){
    updateTag(input:$input, tagID:$tagid) {
        id
    }
}
""")


class TagBuilder:
    def __init__(self, graphql_url: str, api_key: str):
        self.graphql_url = graphql_url
        self.api_key = api_key
        transport = RequestsHTTPTransport(
            url=graphql_url,
            headers={"X-Tavern-Access-Token": api_key},
            verify=True,
            retries=3,
        )
        self._client = Client(
            transport=transport, fetch_schema_from_transport=True
        )

    def execute(self, document, variables):
        """Execute a parsed GraphQL document, returning the data or None."""
        try:
            return self._client.execute(document, variable_values=variables)
        except TransportQueryError as e:
            print(f"GraphQL errors: {e.errors}")
            return None
        except TransportError as e:
            print(f"GraphQL request failed: {e}")
            return None

    def get_hosts(self, where):
        """Return hosts matching a ``HostWhereInput`` filter, with their tags."""
        res = self.execute(GET_HOSTS, {"where": where})
        if res is None:
            return None
        return [edge["node"] for edge in res["hosts"]["edges"]]

    def get_tag(self, tag_name):
        """Return ``{"id", "kind"}`` for a tag, or ``None`` if it is missing."""
        res = self.execute(GET_TAG, {"input": {"name": tag_name}})
        if res is None:
            return None
        edges = res["tags"]["edges"]
        if not edges:
            return None
        return edges[0]["node"]

    def ensure_tag(self, tag_name: str, tag_kind: str):
        """Return the id for a tag, creating it when it does not exist."""
        existing = self.get_tag(tag_name)
        if existing is not None:
            if existing.get("kind") != tag_kind:
                print(
                    f"Tag {tag_name!r} already exists with kind "
                    f"{existing.get('kind')!r}, expected {tag_kind!r}"
                )
                return None
            return existing["id"]

        res = self.execute(
            CREATE_TAG, {"input": {"name": tag_name, "kind": tag_kind}}
        )
        if res is None:
            return None
        print(f"Created tag {tag_name!r} ({tag_kind})")
        return res["createTag"]["id"]

    def add_hosts_to_tag(self, tag_id: str, host_ids: list):
        res = self.execute(
            UPDATE_TAG,
            {"input": {"addHostIDs": host_ids}, "tagid": tag_id},
        )
        return res is not None

    def remove_hosts_from_tag(self, tag_id: str, host_ids: list):
        res = self.execute(
            UPDATE_TAG,
            {"input": {"removeHostIDs": host_ids}, "tagid": tag_id},
        )
        return res is not None

    def apply_rule(self, rule, overwrite=False, claimed=None):
        """Apply one mapping rule. Returns True on success."""
        if claimed is None:
            claimed = {}

        tag_id = self.ensure_tag(rule["name"], rule["kind"])
        if tag_id is None:
            print(f"Skipping rule {rule['name']!r}: could not resolve tag")
            return False

        hosts = self.get_hosts(rule["where"])
        if hosts is None:
            print(f"Skipping rule {rule['name']!r}: host query failed")
            return False

        to_add = []
        removals = {}
        skipped_same_kind = 0
        for host in hosts:
            host_id = host["id"]
            known_tags = [
                (tag["id"], tag.get("kind"))
                for tag in map(lambda x: x.get("node", {}), host.get("tags", []).get("edges", []))
                if "id" in tag
            ]
            if host_id in claimed:
                known_tags.append(claimed[host_id])

            if any(tid == tag_id for tid, _ in known_tags):
                continue

            same_kind_ids = [
                tid for tid, kind in known_tags if kind == rule["kind"]
            ]
            if same_kind_ids and not overwrite:
                skipped_same_kind += 1
                continue

            if overwrite:
                for old_tag_id in same_kind_ids:
                    removals.setdefault(old_tag_id, []).append(host_id)
            to_add.append(host_id)
            claimed[host_id] = (tag_id, rule["kind"])

        ok = True
        for old_tag_id, host_ids in removals.items():
            if not self.remove_hosts_from_tag(old_tag_id, host_ids):
                print(
                    f"Failed to remove {len(host_ids)} host(s) "
                    f"from tag {old_tag_id!r} for rule {rule['name']!r}"
                )
                ok = False

        if to_add:
            if not self.add_hosts_to_tag(tag_id, to_add):
                print(f"Failed to tag {len(to_add)} host(s) with {rule['name']!r}")
                return False

        print(
            f"Rule {rule['name']!r}: tagged {len(to_add)} host(s), "
            f"skipped {skipped_same_kind} host(s) already carrying "
            f"a {rule['kind']!r} tag"
        )
        return ok

    def run(self, mapping_path, overwrite=False):
        """Apply every rule in the mapping file, in file order."""
        rules = load_tag_rules(mapping_path)
        print(f"Loaded {len(rules)} tag rule(s) from {mapping_path}")

        claimed = {}
        failed = 0
        for rule in rules:
            if not self.apply_rule(rule, overwrite=overwrite, claimed=claimed):
                failed += 1

        if failed:
            print(f"{failed} rule(s) failed")
            return 1
        return 0


def main():
    parser = argparse.ArgumentParser(
        prog="Team Tag generator",
        description=(
            "Apply Tavern tags to hosts based on a YAML mapping file. "
            "Each entry defines a tag name, kind, and a 'where' clause "
            "using HostWhereInput fields."
        ),
    )

    parser.add_argument("tavern_url")
    parser.add_argument(
        "mapping",
        help="Path to a YAML file listing tags with their kind and "
        "'where' conditions (see example.yaml)",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Move matching hosts onto the rule's tag even when they "
        "already carry a tag",
    )

    args = parser.parse_args()

    api_key = os.environ.get("TAVERN_API_TOKEN")

    if api_key is None:
        print(
            "No API key found. Please set it using the environment variable "
            "TAVERN_API_TOKEN"
        )
        return 1

    graphql_url = f"{args.tavern_url}/graphql"
    poster = TagBuilder(graphql_url, api_key)
    try:
        return poster.run(args.mapping, overwrite=args.overwrite)
    except FileNotFoundError:
        print(f"Mapping file not found: {args.mapping}")
        return 1
    except (ValueError, yaml.YAMLError) as e:
        print(f"Invalid mapping file {args.mapping}: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
