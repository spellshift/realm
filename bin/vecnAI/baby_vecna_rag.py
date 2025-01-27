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


def embed_documents():
    data = {}
    with open('./test_data.json', 'r') as file:
        data = json.load(file)

    texts = []
    for quest in data["data"]["quests"]["edges"]:
        for (i, node) in enumerate(quest["node"]["tasks"]["edges"]):
            quest["node"]["tasks"]["edges"][i]["node"]["output"] = ""

        if len(str(quest)) > 0:
            texts.append(str(quest))

    # The dimensionality of the output embeddings.
    dimensionality = 384
    # The task type for embedding. Check the available tasks in the model's documentation.
    task = "RETRIEVAL_DOCUMENT"

    model = TextEmbeddingModel.from_pretrained("text-embedding-005")
    inputs = []
    local_texts = []
    ids = []
    for (i, text) in enumerate(texts):
        inputs.append(TextEmbeddingInput(text, task))
        ids.append(str(i))
        local_texts.append(text)
        if i % 25 == 0:
            kwargs = dict(
                output_dimensionality=dimensionality) if dimensionality else {}
            embeddings = model.get_embeddings(inputs, **kwargs)

            # switch `add` to `upsert` to avoid adding the same documents every time
            collection.upsert(
                ids=ids,
                documents=local_texts,
                embeddings=[embedding.values for embedding in embeddings],
            )
            ids = []
            inputs = []
            local_texts = []

    return collection


def embed_eldritch_docs():
    # The dimensionality of the output embeddings.
    dimensionality = 384
    # The task type for embedding. Check the available tasks in the model's documentation.
    task = "RETRIEVAL_DOCUMENT"
    model = TextEmbeddingModel.from_pretrained("text-embedding-005")

    i = 0
    local_texts = []
    ids = []
    inputst = []
    for root, dirs, files in os.walk("realm/docs/_docs"):
        for file in files:
            if file.endswith(".md"):
                with open(os.path.join(root, file), 'r') as reader:
                    for line in reader:
                        if line.strip():
                            inputs.append(TextEmbeddingInput(line, task))
                            local_texts.append(line)
                            ids.append(f"eldirtch-docs-{i}")

                            if i % 5 == 0:
                                kwargs = dict(
                                    output_dimensionality=dimensionality) if dimensionality else {}
                                embeddings = model.get_embeddings(
                                    inputs, **kwargs)

                                # switch `add` to `upsert` to avoid adding the same documents every time
                                collection.upsert(
                                    ids=ids,
                                    documents=local_texts,
                                    embeddings=[
                                        embedding.values for embedding in embeddings],
                                )
                                inputs = []
                                local_texts = []
                                ids = []

                            i += 1

    return collection


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

    base_prompt = {
        "content": "I am going to ask you a question, which I would like you to answer"
        " based only on the provided context, and not any other information."
        " If there is not enough information in the context to answer the question,"
        ' say "I am not sure", then try to make a guess.'
        " Break your answer up into nicely readable paragraphs."
        " Respond with as few words as possible.",
    }
    user_prompt = {
        "content": f" The question is '{query}'. Here is all the context you have:"
        f'{(" ").join(context)}',
    }

    # combine the prompts to output a single prompt string
    system = f"{base_prompt['content']} {user_prompt['content']}"

    return system


# collection = embed_documents()
# collection = embed_eldritch_docs()
# pprint.pprint(collection)

while True:
    user_input = input("vecnAI> ")
    if user_input.lower() in ["exit", "quit"]:
        break

    results = collection.query(
        query_texts=[user_input], n_results=10, include=["documents", "embeddings"]
    )
    response = generate(build_prompt(
        user_input, str(results["documents"])))
    print(response)
