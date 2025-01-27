import time
import google.generativeai as genai

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
    {"text_input": "get all tags and associated hosts", "output": """```graphql
query getTag{
  tags {
    name
    id
    hosts {
      name
      primaryIP
      platform
    }
  }
}```"""},
    {"text_input": "Count the number of hosts for each OS", "output": """```graphql
query HostsByPlatformCount {
  hosts {
    platform
  }
}```"""},
    {"text_input": "How many shells are there and whose using them", "output": """```graphql
query listShells {
  shells {
    edges {
      node {
        id
        beacon {
          id
        }
        activeUsers {
          id
        }
      }
    }
  }
}```"""},
    {"text_input": "Show all files collected", "output": """```graphql
query getHostFiles {
  hosts {
    name
    files {
      id
      path
      hash
    }
  }
}```"""},
    {"text_input": "Show all files collected", "output": """```graphql
query getHostFiles {
  hosts {
    name
    files {
      id
      path
      hash
    }
  }
}```"""},
    {"text_input": "List all hosts from team one", "output": """```graphql
query {
  hosts(where:{hasTagsWith:{nameContains:"1"}}){
    name
  }
}```"""},
    {"text_input": "List all hosts from team two", "output": """```graphql
query {
  hosts(where:{hasTagsWith:{nameContains:"2"}}){
    name
  }
}```"""},
    {"text_input": "List all hosts running service mysql", "output": """```graphql
query {
  hosts(where:{hasTagsWith:{nameContains:"mysql"}}){
    name
  }
}```"""},
    {"text_input": "List all hosts running service nginx", "output": """```graphql
query {
  hosts(where:{hasTagsWith:{nameContains:"mysql"}}){
    name
  }
}```"""},
]


base_model = "models/gemini-1.5-flash-001-tuning"

operation = genai.create_tuned_model(
    # You can use a tuned model here too. Set `source_model="tunedModels/..."`
    display_name="increment",
    source_model=base_model,
    epoch_count=20,
    batch_size=4,
    learning_rate=0.001,
    training_data=training_data,
)

for status in operation.wait_bar():
    time.sleep(10)

result = operation.result()
print(result)
# # You can plot the loss curve with:
# snapshots = pd.DataFrame(result.tuning_task.snapshots)
# sns.lineplot(data=snapshots, x='epoch', y='mean_loss')

model = genai.GenerativeModel(model_name=result.name)
result = model.generate_content("III")
print(result.text)  # IV
