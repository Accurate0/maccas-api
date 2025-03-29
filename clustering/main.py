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
        if cluster_no not in cluster_group:
            cluster_group[str(cluster_no)] = []

        cluster_group[str(cluster_no)].append(name)

    return cluster_group


@app.get("/health", status_code=status.HTTP_204_NO_CONTENT)
async def health():
    return None
