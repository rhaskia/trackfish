import eyed3
import os
import string
import logging

logging.disable()

directory = "E:/Music/"
files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f)) and f.endswith(".mp3")]
all_genres = []
data = {}

def clean(str1):
    no_whitespace = ''.join(str1.split()).lower()

    no_punc = ''.join(char for char in no_whitespace if char not in string.punctuation)

    if no_punc.endswith("music"):
        return no_punc[:-5]
    else: 
        return no_punc

for file in files: 
    audiofile = eyed3.load("E:/Music/" + file)

    if not audiofile: 
        print("file has no tags:" + file)
        continue

    genre = audiofile.tag.genre 

    if genre:
        genres = [clean(g) for g in genre.name.split("\0")]
        for genre in genres:
            old = data.get(genre)
            if not old:
                data[genre] =[]
            data[genre] += [file]

        all_genres += genres
    else:
        print(file + " has no genre tag")
        new_genres = input("> ")
        audiofile.tag.genre = new_genres.replace(';', '\0')
        audiofile.tag.save()

for (genre, songs) in data.items():
    if len(songs) == 1:
        print(genre + " has only one song")
