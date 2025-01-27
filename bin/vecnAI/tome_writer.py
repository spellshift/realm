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
genai.configure(
    api_key=gemini_api_key
)

model = genai.GenerativeModel("gemini-2.0-flash-exp")

chroma_client = chromadb.PersistentClient(path="/workspaces/chromadb-tomes")
collection = chroma_client.get_or_create_collection(name="tomes")


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
    print(input)
    res = model.generate_content(
        contents=contents,
        generation_config=generate_content_config
    )

    return "".join(x.text for x in res.candidates[0].content.parts)


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
    inputs = []
    with open("/workspaces/realm/docs/_docs/user-guide/eldritch.md", 'r') as reader:
        doc = reader.read()
        docs = doc.split("### ")
        for section in docs:
            if section.strip():
                inputs.append(
                    TextEmbeddingInput(section, task))
                local_texts.append(section)
                ids.append(f"eldirtch-docs-{i}")

                if i % 20 == 0 or i == len(docs)-1:
                    i += 1
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
        "content": "I am going to ask you to build tomes, I want you to answer"
        " based only on the provided context, and not any other information."
        " If there is not enough information in the context to answer the question,"
        ' say "I am not sure", then try to make a guess.'
        " Break your answer up into well formatted and readable code."
        " Respond with as few words as possible.",
    }
    user_prompt = {
        "content": f" The tome i want you to write is '{query}'. Here is all the context you have:"
        f'{("").join(context)}',
    }

    # combine the prompts to output a single prompt string
    system = f"{base_prompt['content']} {user_prompt['content']}"

    return system


# collection = embed_documents()
collection = embed_eldritch_docs()
# pprint.pprint(collection)

while True:
    user_input = input("vecnAI> ")
    if user_input.lower() in ["exit", "quit"]:
        break

    results = collection.query(
        query_texts=[user_input], n_results=10, include=["documents", "embeddings"]
    )
    print(results)
    response = generate(build_prompt(
        user_input, results["documents"]))
    print(response)
