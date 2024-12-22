import eyed3
import os
import string
import logging
import numpy as np
import pandas as pd
import tensorflow as tf
from tensorflow.keras.layers import Input, Dense
from tensorflow.keras.models import Model
from tensorflow.keras.regularizers import l1

from sklearn.metrics import accuracy_score, precision_score, recall_score
from sklearn.model_selection import train_test_split
from tensorflow.keras import layers, losses
from tensorflow.keras.datasets import fashion_mnist
from tensorflow.keras.models import Model
import utils
import random

logging.disable()

directory = "E:/Music/"
files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f))]
data = {}

def clean(str1):
    no_whitespace = ''.join(str1.split()).lower()

    no_punc = ''.join(char for char in no_whitespace if char not in string.punctuation)

    if no_punc.endswith("music"):
        return no_punc[:-5]
    else: 
        return no_punc

genre_index = []
all_lists = []

for file in files: 
    audiofile = eyed3.load("E:/Music/" + file)

    if not audiofile: 
        continue

    genre = audiofile.tag.genre 

    if genre:
        genres = [clean(g) for g in genre.name.split("\0")]
        encoding = []
            
        for genre1 in genres: 
            try:
                index = genre_index.index(genre1) 
                encoding.append(index)
            except ValueError:
                encoding.append(len(genre_index))
                genre_index.append(genre1)

        all_lists.append(encoding)

tracks = utils.load('data/fma_metadata/tracks.csv')
genre_csv = utils.load('data/fma_metadata/genres.csv')

train = tracks['set', 'split'] == 'training'
val = tracks['set', 'split'] == 'validation'
test = tracks['set', 'split'] == 'test'

small = tracks['set', 'subset'] <= 'medium'
extra_genres = tracks.loc[small & train, ('track', 'genres')]
print(extra_genres.shape)

for genre in extra_genres:
    genres = [clean(genre_csv['title'][g]) for g in genre]
    encoding = []
        
    for genre1 in genres: 
        try:
            index = genre_index.index(genre1) 
            encoding.append(index)
        except ValueError:
            encoding.append(len(genre_index))
            genre_index.append(genre1)

    all_lists.append(encoding)

random.shuffle(all_lists)

genre_len = len(genre_index)
print(genre_len)
one_hot_encodings = []
for song in all_lists:
    encoding = [1 if i in song else 0 for i in range(0, genre_len)]
    one_hot_encodings.append(encoding)

x = np.array(one_hot_encodings)
split = int((4 * len(x)) // 5)
print(split, len(x))
x_train, x_test = x[:split], x[split:]
x_train = x_train.astype('float32')
x_test = x_test.astype('float32')

class Autoencoder(Model):
  def __init__(self, latent_dim, shape):
    super(Autoencoder, self).__init__()
    self.latent_dim = latent_dim
    self.shape = shape
    self.encoder = tf.keras.Sequential([
      layers.Flatten(),
      layers.Dense(latent_dim, activation='relu'),
    ])
    self.decoder = tf.keras.Sequential([
      layers.Dense(tf.math.reduce_prod(shape).numpy(), activation='sigmoid'),
      layers.Reshape(shape)
    ])

  def call(self, x):
    encoded = self.encoder(x)
    decoded = self.decoder(encoded)
    return decoded


shape = x_test.shape[1:]
latent_dim = 4
autoencoder = Autoencoder(latent_dim, shape)

autoencoder.compile(optimizer='adam', loss=losses.MeanSquaredError())

autoencoder.fit(x_train, x_train,
                epochs=100,
                shuffle=True,
                validation_data=(x_test, x_test))

encoded_imgs = autoencoder.encoder(x_test).numpy()
decoded_imgs = autoencoder.decoder(encoded_imgs).numpy()

for j in range(30):
    for genre in decoded_imgs[j].argsort()[::-1][:10]:
        print(genre_index[genre], end=",")
    print([genre_index[g] for g in all_lists[split + j]])

# pop_index = genre_index.index("pop") 
# rap_index = genre_index.index("rock") 
# country_index = genre_index.index("country") 
# pop = [1 if i == pop_index else 0 for i in range(0, genre_len)]
# rap = [1 if i == rap_index else 0 for i in range(0, genre_len)]
# country = [1 if i == country_index else 0 for i in range(0, genre_len)]
# encoded_imgs = autoencoder.encoder(np.array([pop, rap, country])).numpy()
# added = np.array([(1.0 / encoded_imgs[0])]).reshape(1, 128)
# decoded_imgs = autoencoder.decoder(added).numpy()
# print(decoded_imgs.shape)
#
# for j in range(1):
#     for genre in decoded_imgs[j].argsort()[::-1][:10]:
#         print(genre_index[genre], end=", ")
