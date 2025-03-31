from fastapi import FastAPI, status
from pydantic import BaseModel
import numpy as np
import hdbscan
from sklearn.metrics.pairwise import cosine_distances

app = FastAPI()


class Embedding(BaseModel):
    name: str
    embedding: list[float]


class Embeddings(BaseModel):
    embeddings: list[Embedding]


def cluster(embeddings: list[float], names: list[str]):
    embeddings = np.array(embeddings)
    hdb = hdbscan.HDBSCAN(
        min_samples=None,
        alpha=0.93,
        min_cluster_size=2,
        max_cluster_size=15,
        cluster_selection_epsilon=0.56,
        cluster_selection_method='leaf',
        memory='/tmp').fit(embeddings)

    unclustered_indices = np.where(hdb.labels_ == -1)[0]
    clustered_indices = np.where(hdb.labels_ != -1)[0]
    final_labels = hdb.labels_.copy()

    cluster_centroids = {}
    for cluster_id in set(hdb.labels_):
        if cluster_id != -1:
            cluster_points = embeddings[clustered_indices][hdb.labels_[
                clustered_indices] == cluster_id]
            cluster_centroids[cluster_id] = np.mean(cluster_points, axis=0)

    for unclustered_idx in unclustered_indices:
        unclustered_embedding = embeddings[unclustered_idx]
        min_distance = float('inf')
        best_cluster = -1

        for cluster_id, centroid in cluster_centroids.items():
            if np.sum(final_labels == cluster_id) < 15:
                distance = cosine_distances(
                    [unclustered_embedding], [centroid])[0][0]
                if distance < min_distance:
                    min_distance = distance
                    best_cluster = cluster_id

        if best_cluster != -1:
            final_labels[unclustered_idx] = best_cluster

    for cluster_id in set(final_labels):
        if cluster_id != -1:
            cluster_indices = np.where(final_labels == cluster_id)[0]
            if len(cluster_indices) > 15:
                large_cluster_embeddings = embeddings[cluster_indices]
                recluster_hdb = hdbscan.HDBSCAN(
                    min_samples=2,
                    alpha=1.0,
                    min_cluster_size=2,
                    cluster_selection_epsilon=0.2,
                    cluster_selection_method='eom',
                    memory='/tmp'
                ).fit(large_cluster_embeddings)

                max_label = max(final_labels)
                reclustered_labels = recluster_hdb.labels_
                reclustered_labels = [
                    label + max_label + 1
                    if label != -1 else -1
                    for label in reclustered_labels
                ]

                # Update final labels with reclustered labels
                for idx, new_label in zip(cluster_indices, reclustered_labels):
                    final_labels[idx] = new_label

    return final_labels


@app.post("/clusters")
async def clusters(body: Embeddings):
    embeddings = list(map(lambda b: b.embedding, body.embeddings))
    names = list(map(lambda b: b.name, body.embeddings))

    embeddings = np.array(embeddings)
    labels = cluster(embeddings, names)

    cluster_group = {}

    for (cluster_no, embeddings, name) in zip(labels, embeddings, names):
        cluster_name = str(cluster_no)
        if cluster_name not in cluster_group:
            cluster_group[cluster_name] = []

        cluster_group[cluster_name].append(name)

    return cluster_group


@app.get("/health", status_code=status.HTTP_204_NO_CONTENT)
async def health():
    return None
