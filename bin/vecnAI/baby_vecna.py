import chromadb
from google import genai
from google.genai import types
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

model = "gemini-2.0-flash-exp"

chroma_client = chromadb.PersistentClient(path="/workspaces/chromadb")
collection = chroma_client.get_or_create_collection(name="my_collection")

client = genai.Client(
    vertexai=True,
    project="ccdc-red-team-infra",
    location="us-central1"
)

vertexaiobj = vertexai.init(
    project="ccdc-red-team-infra",
)

generate_content_config = types.GenerateContentConfig(
    temperature=1,
    top_p=0.95,
    max_output_tokens=8192,
    response_modalities=["TEXT"],
    safety_settings=[types.SafetySetting(
        category="HARM_CATEGORY_HATE_SPEECH",
        threshold="OFF"
    ), types.SafetySetting(
        category="HARM_CATEGORY_DANGEROUS_CONTENT",
        threshold="OFF"
    ), types.SafetySetting(
        category="HARM_CATEGORY_SEXUALLY_EXPLICIT",
        threshold="OFF"
    ), types.SafetySetting(
        category="HARM_CATEGORY_HARASSMENT",
        threshold="OFF"
    )],
)


def generate(input: str):
    contents = [
        types.Content(
            role="user",
            parts=[types.Part.from_text(input)]
        )
    ]

    res = ""
    for chunk in client.models.generate_content_stream(
        model=model,
        contents=contents,
        config=generate_content_config,
    ):
        res += chunk.text

    return res


def build_prompt(query: str, context: List[str]) -> str:
    """
    Builds a prompt for the LLM. #

    This function builds a prompt for the LLM. It takes the original query,
    and the returned context, and asks the model to answer the question based only
    on what's in the context, not what's in its weights.

    Args:
    query (str): The original query.
    context (List[str]): The context of the query, returned by embedding search.

    Returns:
    A prompt for the LLM (str).
    """

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
