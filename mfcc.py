import librosa
from scipy.spatial.distance import cdist
import os
import numpy as np
from scipy import stats

def find_similar_songs(query_song_mfcc, song_mfccs, n_neighbors=5):
    distances = cdist(query_song_mfcc.reshape(1, -1), song_mfccs, metric='euclidean')
    nearest_neighbor_indices = np.argsort(distances)[0, 1:n_neighbors+1]  # Exclude the query song itself
    return nearest_neighbor_indices

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
    data = np.hstack((kurt,max,mean,med,min,skew,std))
    
    return data

song_mfccs = [song_mfcc(song) for song in files]

# Find similar songs to the first song in the dataset
for j in range(600, 605):
    query_song_mfcc = song_mfccs[j] 
    similar_song_indices = find_similar_songs(query_song_mfcc, song_mfccs)

    # Print indices of similar songs
    print(files[j])
    for song in similar_song_indices:
        print(files[song])
