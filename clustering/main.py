from fastapi import FastAPI, status
from pydantic import BaseModel
import numpy as np
import hdbscan

app = FastAPI()


class Embedding(BaseModel):
    name: str
    embedding: list[float]


class Embeddings(BaseModel):
    embeddings: list[Embedding]


@app.post("/clusters")
async def clusters(body: Embeddings):
    embeddings = list(map(lambda b: b.embedding, body.embeddings))
    names = list(map(lambda b: b.name, body.embeddings))

    embeddings = np.array(embeddings)

    hdb = hdbscan.HDBSCAN(min_samples=3, min_cluster_size=3,
                          cluster_selection_epsilon=0.5).fit(embeddings)

    cluster_group = {}

    for (cluster_no, embeddings, name) in zip(hdb.labels_, embeddings, names):
        cluster_name = str(cluster_no)
        if cluster_name not in cluster_group:
            cluster_group[cluster_name] = []

        cluster_group[cluster_name].append(name)

    return cluster_group


@app.get("/health", status_code=status.HTTP_204_NO_CONTENT)
async def health():
    return None
