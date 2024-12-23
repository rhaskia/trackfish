from sklearn.ensemble import RandomForestClassifier
import librosa
from scipy.spatial.distance import cdist
import os
import numpy as np
from scipy import stats

import sklearn as skl
import utils

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

# Create a Random Forest classifier
rf_classifier = RandomForestClassifier(n_estimators=100, random_state=42)
rf_classifier.fit(X_train, y_train) 

# Get feature importances
feature_importances = rf_classifier.feature_importances_

# Sort features by importance
sorted_indices = np.argsort(feature_importances)[::-1]

# Print feature importances
for i in sorted_indices:
    print(f"Feature {i}: {feature_importances[i]}") 

k = 20

# Select top-k features based on importance 
top_k_features = sorted_indices[:k] 
X_train_reduced = X_train[:, top_k_features] 
X_test_reduced = X_test[:, top_k_features]

# Train and evaluate the model with reduced features
# ...
