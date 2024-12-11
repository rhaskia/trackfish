import matplotlib.pyplot as plt
import eyed3
import os
import string

directory = "/mnt/e/Music/"
files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f))]
data = {}

def clean(str1):
    no_whitespace = ''.join(str1.split()).lower()

    no_punc = ''.join(char for char in no_whitespace if char not in string.punctuation)

    if no_punc.endswith("music"):
        return no_punc[:-5]
    else: 
        return no_punc

# minimum string alphabetically
def min(str1, str2):
    if str1.lower() < str2.lower():
        return str1
    return str2

def max(str1, str2):
    if str1.lower() > str2.lower():
        return str1
    return str2


for file in files: 
    audiofile = eyed3.load("/mnt/e/Music/" + file)

    if not audiofile: 
        continue

    genre = audiofile.tag.genre 

    if genre:
        genres = genre.name.split("\0")
        for genre1 in genres: 
            for genre2 in genres:
                genre1 = clean(genre1)
                genre2 = clean(genre2)
                firstgenre = min(genre1, genre2)
                secondgenre = max(genre1, genre2)
                key = (firstgenre, secondgenre)
                data[key] = data.get(key, 0) + 1

print(data)

data = [(k[0], k[1], v) for k, v in data.items() if k[0] != k[1]]

with open('weights.txt', 'w') as file:
    file.writelines(['' + line[0] + ',' + line[1] + ',' + str(line[2]) + '\n' for line in data])

