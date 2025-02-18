import json

dataset = json.load(open("similars.json", 'r'))

def extract_sentences(dataset):
    sentences = []
    for entry in dataset:
        tags = [tag[0] for tag in entry["tags"]] 
        sentences.append(tags)
        for similar in entry["similars"]:
            similar_tags = [tag[0] for tag in similar["tags"]]
            sentences.append(similar_tags)
    return sentences

sentences = extract_sentences(dataset)

from gensim.models import FastText

model = FastText(sentences, vector_size=32, window=5, min_count=1, workers=4, sg=1)

model.save("fasttext_categories.model")

