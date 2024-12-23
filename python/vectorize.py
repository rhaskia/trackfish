from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity
import numpy as np

with open('genrelist.txt', 'r') as file:
    genre_lists = [' '.join(gl[:-1].split(',')) for gl in file.readlines()]

vectorizer = TfidfVectorizer()

genre_vectors = vectorizer.fit_transform(genre_lists)

# Get the feature names (genres)
feature_names = vectorizer.get_feature_names_out()

# Print the feature names
print(feature_names) 
print(genre_vectors.shape)

document_index = 189

cosine_similarities = cosine_similarity(genre_vectors[document_index], genre_vectors).flatten()

k= 10
    
top_k_indices = cosine_similarities.argsort()[-k:][::-1]

top_k_indices = [idx for idx in top_k_indices if idx != document_index]

for idx in top_k_indices:
    print(genre_lists[idx])
