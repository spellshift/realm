from dataclasses import dataclass
from enum import Enum
from typing import List, Type
from os import walk
from marko import Markdown
import marko


class TOC_ELEM(Enum):
    USER_GUIDE = "User Guide"
    ADMIN_GUIDE = "Admin Guide"
    DEVELOPER_GUIDE = "Developer Guide"
    ABOUT = "About"


@dataclass
class Link:
    title: str
    url: str
    children: List[any] = None


@dataclass
class Title:
    title: str
    url: str
    links: List[Link]


def get_min_heading(file_path: str) -> int:
    md = Markdown()
    min_heading = 16
    with open(file_path) as f:
        doc = md.parse(f.read())
        for elem in doc.children:
            if elem.get_type() == "Heading":
                if elem.level < min_heading:
                    min_heading = elem.level

    return min_heading


def make_url(title: str, link: str, child: str = None) -> str:
    res = f"{title}/{link}"
    if child:
        res = f"{res}#{child}"
    return res


def parse_markdown(file_path: str, title_name: str) -> Title:
    res = Title()
    md = Markdown()
    min_heading = get_min_heading(file_path)
    cur_toc_elem = None

    with open(file_path) as f:
        doc = md.parse(f.read())
        for elem in doc.children:
            if elem.get_type() == "Heading":
                # Check if this is a new top level
                if elem.level == min_heading:
                    print("ok")


def walk_docs(root_dir: str) -> List[Title]:
    res = []
    for (root, dirs, files) in walk(root_dir):
        for f in files:
            if not f.lower().endswith(".md"):
                break
            print(f"parsing: {root}/{f}")
            parse_markdown(f"{root}/{f}")
            return


walk_docs("/workspaces/realm/docs")
