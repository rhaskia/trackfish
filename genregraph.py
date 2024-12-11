import matplotlib.pyplot as plt
import eyed3
import os
import graph_tool.all as gt

directory = "E:/music"
files = [f for f in os.listdir(directory) if os.path.isfile(os.path.join(directory, f))]
data = {}

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
    audiofile = eyed3.load("E:/music/" + file)

    if not audiofile: 
        continue

    genre = audiofile.tag.genre 

    if genre:
        genres = genre.name.split("\0")
        for genre1 in genres: 
            for genre2 in genres:
                firstgenre = min(genre1, genre2)
                secondgenre = max(genre1, genre2)
                key = (firstgenre, secondgenre)
                data[key] = data.get(key, 0) + 1

print(data)

data = [(k[0], k[1], v) for k, v in data.items() if v > 10 and k[0] != k[1]]


def create_graph_tool_graph(node_weights):
    g = gt.Graph(directed=False)  # Create an undirected graph

    # Create a property map for edge weights
    eweight = g.new_edge_property("float")

    # Add vertices to the graph
    vertices = set()
    for source, target, weight in node_weights:
        vertices.add(source)
        vertices.add(target)
    vertex_map = {v: g.add_vertex() for v in vertices}

    # Add edges with weights
    for source, target, weight in node_weights:
        source_vertex = vertex_map[source]
        target_vertex = vertex_map[target]
        e = g.add_edge(source_vertex, target_vertex)
        eweight[e] = weight

    return g, eweight

g, eweight = create_graph_tool_graph(data)

# Visualize the graph (basic example)
pos = gt.sfdp_layout(g) 
gt.graph_draw(g, pos=pos, edge_pen_width=eweight) 

# You can now use graph-tool's features to analyze and visualize the graph
# (e.g., community detection, shortest paths, centrality measures)
