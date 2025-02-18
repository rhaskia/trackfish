import json
import pathlib

def load_track(path): 
    with open(path,'r') as file:
        return json.load(file)
    return None

def load_all(): 
    files = []
    for a in range(65, 91):
        for b in range(65, 91):
            for c in range(65, 91):
                path = f"./{chr(a)}/{chr(b)}/{chr(c)}"
                try: 
                    for item in pathlib.Path(path).iterdir():
                        files.append(load_track(item))
                except: 
                    return files
    return files
