import chromadb
from google import generativeai as genai
from google.generativeai import types
import vertexai
from vertexai.language_models import TextEmbeddingInput, TextEmbeddingModel
import chromadb.utils.embedding_functions as embedding_functions
from typing import List

import base64
import json
import pprint
import chromadb
import os
import glob

gemini_api_key = os.getenv("GEMINI_API_KEY")
genai.configure(
    api_key=gemini_api_key
)

model = genai.GenerativeModel("tunedModels/increment-3mjwqfhey1l4")

chroma_client = chromadb.PersistentClient(path="/workspaces/chromadb")
collection = chroma_client.get_or_create_collection(name="my_collection")


vertexaiobj = vertexai.init(
    project="ccdc-red-team-infra",
)

generate_content_config = types.GenerationConfig(
    temperature=1,
    top_p=0.95,
    max_output_tokens=8192,
    response_mime_type="text/plain",
)


def generate(input: str):
    contents = input

    res = model.generate_content(
        contents=contents,
        generation_config=generate_content_config
    )

    return " ".join(x.text for x in res.candidates[0].content.parts)


def build_prompt(query: str, context: List[str]) -> str:
    # Role
    #
    base_prompt = {
        "content": "You are an expert GraphQL query builder. You will receive a user's question and a GraphQL schema. "
        "Using only the provided schema as context, construct a GraphQL query that answers the user's question. "
        "If there is not enough information in the schema to answer the question, respond with 'I am not sure'. "
        "Respond with the query enclosed in ```graphql``` tags. "
        "Make sure you reference only fields defined in the schema. If asked for a field that doesn't exist use the closest available field"
        "Only use fields defined in the schema. If a requested field does not exist, use the most similar field defined in the schema. "
        "Keep your response concise and to the point."
    }

    user_prompt = {
        "content": f" The question is '{query}'. Here is all the schema you have:"
        f'{(" ").join(context)}',
    }

    # combine the prompts to output a single prompt string
    system = f"{base_prompt['content']} {user_prompt['content']}"

    return system


# collection = embed_documents()
# collection = embed_eldritch_docs()
# pprint.pprint(collection)

graphql_schema = []
graphql_files = glob.glob("../tavern/internal/graphql/*.graphql")
for file_path in graphql_files:
    with open(file_path, "r") as file:
        graphql_schema.append(file.read())

while True:
    user_input = input("vecnAI> ")
    if user_input.lower() in ["exit", "quit"]:
        break

    response = generate(build_prompt(
        user_input, graphql_schema))
    print(response)
