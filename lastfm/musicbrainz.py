import json

file = open("E:/Downloads/recording/mbdump/recording", encoding="utf8")
tracks = [json.loads(line.strip()) for line in file.readlines()]

i = 0
total = 0
max = 0
for track in tracks:
    if len(track['genres']) > 0:
        i += 1
        #print(track['title'], ", ", track['artist-credit'][0]['artist']['name'], track['genres'])
        total += len(track['genres'])
        if len(track['genres']) > max:
            max = len(track['genres'])
print(len(tracks))
print(i)
print(total / i)
print(max)
