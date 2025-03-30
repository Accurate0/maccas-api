from fastapi import FastAPI, status
from pydantic import BaseModel
import numpy as np
import hdbscan
import pandas as pd
from scipy.spatial.distance import cdist

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
        cluster_selection_epsilon=0.56,
        cluster_selection_method='leaf',
        memory='/tmp').fit(embeddings)

    remaining_embeddings = embeddings[hdb.labels_ == -1]

    hdb_recluster = hdbscan.HDBSCAN(
        min_samples=2,
        alpha=1.0,
        min_cluster_size=2,
        cluster_selection_epsilon=0.2,
        cluster_selection_method='eom',
        memory='/tmp').fit(remaining_embeddings)

    new_labels = hdb.labels_.astype(str)
    reclustered_labels = hdb_recluster.labels_.astype(str)
    reclustered_labels = ['r' + label if label !=
                          '-1' else '-1' for label in reclustered_labels]

    max_initial_label = max([int(label)
                            for label in hdb.labels_ if label != -1], default=-1)
    reclustered_labels = hdb_recluster.labels_.astype(int)
    reclustered_labels = [label + max_initial_label +
                          1 if label != -1 else -1 for label in reclustered_labels]

    new_labels = hdb.labels_.astype(int)
    new_labels[hdb.labels_ == -1] = reclustered_labels

    df = pd.DataFrame(embeddings, columns=[
                      f'dim_{i}' for i in range(embeddings.shape[1])])
    df['cluster'] = new_labels.astype(str)
    df['id'] = names

    def merge_close_clusters(df, distance_threshold=0.5):
        cluster_centroids = (
            df.drop(columns=['id'])  # Exclude non-numeric columns
            .groupby('cluster')
            .mean()
            .reset_index()
        )
        distances = cdist(
            cluster_centroids.iloc[:, 1:],  # Exclude the cluster column
            cluster_centroids.iloc[:, 1:]
        )
        merge_map = {}
        for i, row in enumerate(distances):
            for j, dist in enumerate(row):
                if i != j and dist < distance_threshold:
                    merge_map[cluster_centroids.iloc[j]['cluster']
                              ] = cluster_centroids.iloc[i]['cluster']
        df['cluster'] = df['cluster'].replace(merge_map)
        return df

    df = merge_close_clusters(df)

    return df['cluster'].tolist()


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
