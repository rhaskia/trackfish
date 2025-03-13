import os
import librosa
import numpy as np
from scipy.spatial.distance import cosine
from joblib import Parallel, delayed
from sklearn.decomposition import PCA

def extract_mfcc(audio_file, n_mfcc=13):
    y, sr = librosa.load(audio_file, sr=None)
    
    mfcc = librosa.feature.mfcc(y=y, sr=sr, n_mfcc=n_mfcc)
    
    mfcc_mean = np.mean(mfcc, axis=1)
    
    return mfcc_mean

def extract_chroma(audio_file, n_chroma=12):
    y, sr = librosa.load(audio_file, sr=None)
    
    chroma = librosa.feature.chroma_stft(y=y, sr=sr, n_chroma=n_chroma)
    
    chroma_mean = np.mean(chroma, axis=1)
    return chroma_mean

def compare_audio_files(file1, file2, feature_type='both'):
    if feature_type == 'mfcc':
        feature1 = extract_mfcc(file1)
        feature2 = extract_mfcc(file2)
    if feature_type == 'chroma':
        feature1 = extract_chroma(file1)
        feature2 = extract_chroma(file2)
    if feature_type == 'both':
        feature1 = np.hstack([extract_mfcc(file1), extract_chroma(file1)])
        feature2 = np.hstack([extract_mfcc(file2), extract_chroma(file2)])
    
    similarity = 1 - cosine(feature1, feature2)
    
    return similarity


def compare_directory_to_index(audio_files, index, feature_type='chroma'):
    if index < 0 or index >= len(audio_files):
        raise ValueError(f"Index {index} is out of range. Directory contains {len(audio_files)} files.")
    
    index_file = os.path.join(directory, audio_files[index])
    
    similarities = Parallel(n_jobs=-1)(
        delayed(compare_audio_files)(index_file, os.path.join(directory, file)) 
        for file in audio_files
    )
    
    similarity_dict = {audio_files[i]: similarities[i] for i in range(len(audio_files))}
    
    return similarity_dict

directory = 'E:/Music' 
audio_files = [f for f in os.listdir(directory) if f.endswith('.mp3') or f.endswith('.wav')]
song = ""
index = audio_files.index(song)  
print(index)

similarities = list(compare_directory_to_index(audio_files, index, feature_type="chroma").items())
similarities = [[row[0], row[1][0], row[1][1]] for row in list(enumerate(similarities))]
print(similarities)
similarities = sorted(
    similarities, 
    key=lambda x: x[2]
)
similarities.reverse()

print("Similarities with the chosen reference file:")
for similar in similarities[:10]:
    print(f"Index {similar[0]}: {similar[1]}, {similar[2]:.4f}")

