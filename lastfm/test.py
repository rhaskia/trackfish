from gensim.models import FastText

model = FastText.load("fasttext_categories.model")

print("Vector for 'Hip-Hop':", model.wv["Country"])

print(model.wv.most_similar("hardcore"))
