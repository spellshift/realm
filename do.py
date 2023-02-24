
import json

with open('data.json', "r") as f:
    data = json.loads(f.read())
    for entry in data:
        name = entry["name"].lower().replace(' ', '-')
        print(f'"{name}",')
