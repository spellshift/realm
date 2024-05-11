"""
Profiles can match on IP or hostname.
It's not recommended to use both in the same profile but it will do an
inclusive or if the host matches either. 
"""

tag_profiles = [
    {
        "name": "pfsense",
        "kind": "service",
        "ip_regex": "^192\\.168.*$"
    },
    {
        "name": "BackupServer",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.2\\.2$"
    },
    {
        "name": "UbuntuFTP",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.2\\.4$"
    },
    {
        "name": "MailServer",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.2\\.10$"
    },
    {
        "name": "Ubuntu1",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.1\\.10$"
    },
    {
        "name": "Ubuntu2",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.1\\.40$"
    },
    {
        "name": "Ubuntu3",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.1\\.90$"
    },
    {
        "name": "WebApp",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.1\\.30$"
    },
    {
        "name": "Windows1",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.1\\.70$"
    },
    {
        "name": "Windows2",
        "kind": "service",
        "ip_regex": "^10\\.([1-9]|1[0-4])\\.1\\.80$"
    },
    {
        "name": "team01",
        "kind": "group",
        "ip_regex": "^10\\.1\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team02",
        "kind": "group",
        "ip_regex": "^10\\.2\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team03",
        "kind": "group",
        "ip_regex": "^10\\.3\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team04",
        "kind": "group",
        "ip_regex": "^10\\.4\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team05",
        "kind": "group",
        "ip_regex": "^10\\.5\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team06",
        "kind": "group",
        "ip_regex": "^10\\.6\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team07",
        "kind": "group",
        "ip_regex": "^10\\.7\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team08",
        "kind": "group",
        "ip_regex": "^10\\.8\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team09",
        "kind": "group",
        "ip_regex": "^10\\.9\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team10",
        "kind": "group",
        "ip_regex": "^10\\.10\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team11",
        "kind": "group",
        "ip_regex": "^10\\.11\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team12",
        "kind": "group",
        "ip_regex": "^10\\.12\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team13",
        "kind": "group",
        "ip_regex": "^10\\.13\\.(1|2)\\.[0-9]{1,3}$"
    },
    {
        "name": "team14",
        "kind": "group",
        "ip_regex": "^10\\.14\\.(1|2)\\.[0-9]{1,3}$"
    },
]
