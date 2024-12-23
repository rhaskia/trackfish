import librosa
import numpy as np
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.preprocessing import OneHotEncoder, MultiLabelBinarizer
from sklearn.metrics import accuracy_score
from sklearn.multioutput import ClassifierChain
import eyed3
import os
import string
import logging

logging.disable()

def get_genres(file):
    audiofile = eyed3.load("E:/Music/" + file)

    if not audiofile: 
        return None

    genre_tag = audiofile.tag.genre 

    if not genre_tag: 
        return None

    genres = genre_tag.name.split('\0')

    return [clean(genre) for genre in genres]

def get_tonnetz(wav_file_path):
    y, sr = librosa.load(wav_file_path)
    tonnetz = np.array(librosa.feature.tonnetz(y=y, sr=sr))
    return tonnetz

def get_chroma_vector(wav_file_path):
    y, sr = librosa.load(wav_file_path)
    chroma = np.array(librosa.feature.chroma_stft(y=y, sr=sr))
    return chroma

def get_melspectrogram(wav_file_path):
    y, sr = librosa.load(wav_file_path, offset=0, duration=30)
    melspectrogram = np.array(librosa.feature.melspectrogram(y=y, sr=sr))
    return melspectrogram

def get_mfcc(wav_file_path):
    y, sr = librosa.load(wav_file_path, offset=0, duration=30)
    mfcc = np.array(librosa.feature.mfcc(y=y, sr=sr))
    return mfcc

def extract_features(file_path):
    # Extracting MFCC feature
    mfcc = get_mfcc(file_path)
    mfcc_mean = mfcc.mean(axis=1)
    mfcc_min = mfcc.min(axis=1)
    mfcc_max = mfcc.max(axis=1)
    mfcc_feature = np.concatenate( (mfcc_mean, mfcc_min, mfcc_max) )

    # Extracting Mel Spectrogram feature
    melspectrogram = get_melspectrogram(file_path)
    melspectrogram_mean = melspectrogram.mean(axis=1)
    melspectrogram_min = melspectrogram.min(axis=1)
    melspectrogram_max = melspectrogram.max(axis=1)
    melspectrogram_feature = np.concatenate( (melspectrogram_mean, melspectrogram_min, melspectrogram_max) )

    # Extracting chroma vector feature
    chroma = get_chroma_vector(file_path)
    chroma_mean = chroma.mean(axis=1)
    chroma_min = chroma.min(axis=1)
    chroma_max = chroma.max(axis=1)
    chroma_feature = np.concatenate( (chroma_mean, chroma_min, chroma_max) )

    # Extracting tonnetz feature
    tntz = get_tonnetz(file_path)
    tntz_mean = tntz.mean(axis=1)
    tntz_min = tntz.min(axis=1)
    tntz_max = tntz.max(axis=1)
    tntz_feature = np.concatenate( (tntz_mean, tntz_min, tntz_max) ) 

    feature = np.concatenate( (chroma_feature, melspectrogram_feature, mfcc_feature, tntz_feature) )
    return feature

# Load and preprocess data
def load_data(labeled):
    X = []  # Features
    y = []  # Labels (genres)
    
    for file in labeled: 
        file_path = os.path.join("E:/music", file)
        features = extract_features(file_path)
        genres = get_genres(file)
        if features is not None:
            for genre in genres: 
                X.append(features)
                y.append(genre)

    return np.array(X), np.array(y)

# Train a Random Forest classifier
def train_model(X_train, y_train):
    chain = RandomForestClassifier(n_estimators=100, random_state=42)
    chain.fit(X_train, y_train)
    return chain

# Predict genres for unlabeled songs
def predict_genres(model, unlabeled):
    predictions = []
    
    for file_name in unlabeled:
        file_path = os.path.join("E:/music/", file_name)
        features = extract_features(file_path)
        if features is not None:
            predicted_genre = model.predict_proba([features])[0]
            predictions.append((file_name, predicted_genre))
    
    return predictions

def clean(str1):
    no_whitespace = ''.join(str1.split()).lower()

    no_punc = ''.join(char for char in no_whitespace if char not in string.punctuation)

    if no_punc.endswith("music"):
        return no_punc[:-5]
    else: 
        return no_punc

directory = "E:/Music/"
files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f)) and f.endswith(".mp3")][:10]
labeled = []
unlabeled = []
all_genres = []

for file in files:
    genres = get_genres(file)

    if genres:
        labeled.append(file)
        for genre in genres:
            if genre not in all_genres:
                all_genres.append(genre)
    else:
        unlabeled.append(file)

print(len(all_genres))
genre_len = len(all_genres)
fit_genres = np.array([all_genres]).reshape(-1, 1)

print("loaded all data")
            
X, y = load_data(labeled)
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
print(X.shape, y.shape)

model = train_model(X_train, y_train)

sine = model.predict(extract_features("E:/Downloads/sinewave.wav"))
print(sine)

# Evaluate the model (optional)
y_pred = model.predict(X_test)
accuracy = accuracy_score(y_test, y_pred)
print(f"Accuracy: {accuracy}") 


predicted_genres = predict_genres(model, unlabeled)

# Save predictions (optional)
with open("predicted_genres.txt", "w", encoding ="utf-8") as f:
    for file_name, genres in predicted_genres:
        f.write(f"{file_name}: ")
        top_genres = [str(all_genres[i]) + ";" + str(genres[i]) for i in np.array(genres).argsort()][::-1][:4]
        f.write(",".join(top_genres))
        f.write("\n")
