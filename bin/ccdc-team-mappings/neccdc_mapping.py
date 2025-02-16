"""
Profiles can match on IP or hostname.
It's not recommended to use both in the same profile but it will do an
inclusive or if the host matches either.
"""

tag_profiles = [
    {
        "name": "idm",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.132$",
    },
    {
        "name": "controller",
        "kind": "service",
        "hostname_regex": "^controller$",
    },
    {
        "name": "wazuh",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.180$",
    },
    {
        "name": "kube_01",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.200$",
    },
    {
        "name": "kube_02",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.210$",
    },
    {
        "name": "kube_03",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.220$",
    },
    {
        "name": "dc_01",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.16$",
    },
    {
        "name": "ca",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.32$",
    },
    {
        "name": "win_01",
        "kind": "service",
        "hostname_regex": "^win-01$",
    },
    {
        "name": "win_02",
        "kind": "service",
        "ip_regex": "^10\\.0\\.[0-9]{1,2}\\.110$",
    },
    {
        "name": "team01",
        "kind": "group",
        "ip_regex": "^10\\.0\\.1\\.[0-9]{1,3}$"
    },
    {
        "name": "team02",
        "kind": "group",
        "ip_regex": "^10\\.0\\.2\\.[0-9]{1,3}$"
    },
    {
        "name": "team03",
        "kind": "group",
        "ip_regex": "^10\\.0\\.3\\.[0-9]{1,3}$"
    },
    {
        "name": "team04",
        "kind": "group",
        "ip_regex": "^10\\.0\\.4\\.[0-9]{1,3}$"
    },
    {
        "name": "team05",
        "kind": "group",
        "ip_regex": "^10\\.0\\.5\\.[0-9]{1,3}$"
    },
    {
        "name": "team06",
        "kind": "group",
        "ip_regex": "^10\\.0\\.6\\.[0-9]{1,3}$"
    },
    {
        "name": "team07",
        "kind": "group",
        "ip_regex": "^10\\.0\\.7\\.[0-9]{1,3}$"
    },
    {
        "name": "team08",
        "kind": "group",
        "ip_regex": "^10\\.0\\.8\\.[0-9]{1,3}$"
    },
    {
        "name": "team09",
        "kind": "group",
        "ip_regex": "^10\\.0\\.9\\.[0-9]{1,3}$"
    },
    {
        "name": "team10",
        "kind": "group",
        "ip_regex": "^10\\.0\\.10\\.[0-9]{1,3}$"
    },
    {
        "name": "team11",
        "kind": "group",
        "ip_regex": "^10\\.0\\.11\\.[0-9]{1,3}$"
    },
    {
        "name": "team12",
        "kind": "group",
        "ip_regex": "^10\\.0\\.12\\.[0-9]{1,3}$"
    },
    {
        "name": "team13",
        "kind": "group",
        "ip_regex": "^10\\.0\\.13\\.[0-9]{1,3}$"
    },
    {
        "name": "team14",
        "kind": "group",
        "ip_regex": "^10\\.0\\.14\\.[0-9]{1,3}$"
    },
    {
        "name": "team15",
        "kind": "group",
        "ip_regex": "^10\\.0\\.15\\.[0-9]{1,3}$"
    },
    {
        "name": "team16",
        "kind": "group",
        "ip_regex": "^10\\.0\\.16\\.[0-9]{1,3}$"
    },
    {
        "name": "team17",
        "kind": "group",
        "ip_regex": "^10\\.0\\.17\\.[0-9]{1,3}$"
    },
    {
        "name": "team18",
        "kind": "group",
        "ip_regex": "^10\\.0\\.18\\.[0-9]{1,3}$"
    },
    {
        "name": "team19",
        "kind": "group",
        "ip_regex": "^10\\.0\\.19\\.[0-9]{1,3}$"
    },
    {
        "name": "team20",
        "kind": "group",
        "ip_regex": "^10\\.0\\.20\\.[0-9]{1,3}$"
    },
]
