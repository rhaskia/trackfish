import utils
import json

files = utils.load_all()

print("loaded all files")

with_tags = [f for f in files if len(f['tags']) > 0] 
with_similars = [f for f in with_tags if len(f['similars']) > 0]
ids = [track['track_id'] for track in with_similars]

print("sorted files")
tag_min = 10

tracks = []
for track in with_similars:
    similars = []
    tags = track['tags']
    for similar_id in track['similars']:
        if similar_id[0] in ids:
            similar = with_similars[ids.index(similar_id[0])] 
            tags = [tag for tag in similar['tags'] if int(tag[1]) >= tag_min]
            if len(tags) > 0: 
                similars.append({ 'tags' : tags, 'similarity': similar_id[1] })
    if len(similars) > 0:
        tags = [tag for tag in similar['tags'] if int(tag[1]) >= tag_min]
        tracks.append({ 'tags' : tags, 'similars' :  similars })

print(len(tracks))

f = open("similars.json", 'w')
f.write(json.dumps(tracks))

