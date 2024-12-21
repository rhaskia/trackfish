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

genre_len = len(genre_index)
one_hot_encodings = []
for song in all_lists:
    encoding = [1 if i in song else 0 for i in range(0, genre_len)]
    one_hot_encodings.append(encoding)

x = np.array(one_hot_encodings)
x_train, x_test = x[:1400], x[1400:]
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
latent_dim = 128
autoencoder = Autoencoder(latent_dim, shape)

autoencoder.compile(optimizer='adam', loss=losses.MeanSquaredError())

autoencoder.fit(x_train, x_train,
                epochs=500,
                shuffle=True,
                validation_data=(x_test, x_test))

encoded_imgs = autoencoder.encoder(x_test).numpy()
decoded_imgs = autoencoder.decoder(encoded_imgs).numpy()

for j in range(30):
    print([genre_index[i] for i in all_lists[1400 + j]], [genre_index[i] for i in range(genre_len) if decoded_imgs[j][i] > 0.05][:10])
