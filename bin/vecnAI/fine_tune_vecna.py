training_data = [
    {"text_input": "get all windows hosts", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_WINDOWS }) {
    id
    primaryIp
  }
}
```"""},
    {"text_input": "get all hosts running windows", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_WINDOWS }) {
    name
    primaryIp
  }
}
```"""},
    {"text_input": "get all hosts running linux", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_LINUX }) {
    name
    primaryIp
  }
}
```"""},
    {"text_input": "get the ip of all windows hosts", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_WINDOWS }) {
    primaryIp
  }
}
```"""},
    {"text_input": "get the ip of all windows hosts", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_WINDOWS }) {
    primaryIp
  }
}
```"""},
    {"text_input": "get all quests run against linux", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_LINUX }) {
	beacons {
      tasks {
        quest {
          name
          parameters
        }
      }
    }
  }
}```"""},
    {"text_input": "get all quests run against mac", "output": """```graphql
query {
  hosts(where: { platform: PLATFORM_MACOS }) {
	beacons {
      tasks {
        quest {
          name
          parameters
        }
      }
    }
  }
}```"""},
    {"text_input": "8", "output": "9"},
    {"text_input": "-98", "output": "-97"},
    {"text_input": "1,000", "output": "1,001"},
    {"text_input": "10,100,000", "output": "10,100,001"},
    {"text_input": "thirteen", "output": "fourteen"},
    {"text_input": "eighty", "output": "eighty one"},
    {"text_input": "one", "output": "two"},
    {"text_input": "three", "output": "four"},
    {"text_input": "seven", "output": "eight"},
]
