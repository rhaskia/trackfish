import os

#import IPython.display as ipd
import numpy as np
from scipy import stats
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import sklearn as skl
import sklearn.utils, sklearn.preprocessing, sklearn.decomposition, sklearn.svm
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import StandardScaler
from sklearn.svm import SVC
from sklearn.metrics import accuracy_score, hamming_loss, jaccard_score
import librosa
import librosa.display
from sklearn.multiclass import OneVsRestClassifier
from sklearn.model_selection import GridSearchCV


directory = "E:/music/"
files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f)) and f.endswith(".mp3")]

def song_mfcc(file):
    y, sr = librosa.load(directory + file, offset=0, duration=30)
    mfcc = np.array(librosa.feature.mfcc(y=y, sr=sr))

    mean = np.mean(mfcc, axis=1)
    std = np.std(mfcc, axis=1)
    skew = stats.skew(mfcc, axis=1)  
    kurt = stats.kurtosis(mfcc, axis=1)  
    med = np.median(mfcc, axis=1)
    min = np.min(mfcc, axis=1)
    max = np.max(mfcc, axis=1)
    data = np.vstack((mean,std,skew,kurt,med,min,max)).reshape(1, 140)
    data = np.vstack((kurt,max,mean,med,min,skew,std)).reshape(1, 140)
    
    return data

import utils

plt.rcParams['figure.figsize'] = (17, 5)

# Directory where mp3 are stored.
AUDIO_DIR = os.environ.get('AUDIO_DIR')

# Load metadata and features.
tracks = utils.load('data/fma_metadata/tracks.csv')
genres = utils.load('data/fma_metadata/genres.csv')
features = utils.load('data/fma_metadata/features.csv')
echonest = utils.load('data/fma_metadata/echonest.csv')

np.testing.assert_array_equal(features.index, tracks.index)
assert echonest.index.isin(tracks.index).all()

print(tracks.shape, genres.shape, features.shape, echonest.shape)

small = tracks['set', 'subset'] <= 'medium'

train = tracks['set', 'split'] == 'training'
val = tracks['set', 'split'] == 'validation'
test = tracks['set', 'split'] == 'test'

y_train = tracks.loc[small & train, ('track', 'genre_top')]
y_test = tracks.loc[small & test, ('track', 'genre_top')]
X_train = features.loc[small & train, 'mfcc']
X_test = features.loc[small & test, 'mfcc']

print('{} training examples, {} testing examples'.format(y_train.size, y_test.size))
print('{} features, {} classes'.format(X_train.shape[1], np.unique(y_train).size))

scaler = skl.preprocessing.StandardScaler(copy=False)
scaler.fit_transform(X_train)
scaler.transform(X_test)

clf = skl.svm.SVC(C=1000, gamma="auto", kernel="rbf")
clf.fit(X_train, y_train)
score = clf.score(X_test, y_test)
print('Accuracy: {:.2%}'.format(score))


from sklearn.metrics import accuracy_score
y_pred = clf.predict(X_train)
print(accuracy_score(y_train, y_pred))

for i in range(300, 310):
    data = song_mfcc(files[i])
    X_new_scaled = scaler.transform(data)

    y_predicted = clf.predict(X_new_scaled)

    print("Predicted labels:", y_predicted)
    print(files[i])
