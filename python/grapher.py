import csv
import os
from graph_tool.all import *

g = Graph(directed=False)  # Create an undirected graph
v_name = g.new_vertex_property("string")
vertices = {}

def get_vert(g, d, vert): 
    if vert not in d:
        d[vert] = g.add_vertex()
        print(vert, d[vert])
    return d[vert]

with open("weights.txt", 'r') as csvfile:
    reader = csv.reader(csvfile)
    next(reader)  # Skip the header row if present

    # Create edge properties
    e_weight = g.new_edge_property("float") 

    for row in reader:
        source, target, weight = row
        if int(weight) < 3: continue

        v1 = get_vert(g, vertices, source)
        v_name[v1] = source
        v2 = get_vert(g, vertices, target)
        v_name[v2] = target

        g.add_edge(v1, v2)
        e_weight[g.edge(v1, v2)] = 1 / float(weight)

state = minimize_blockmodel_dl(g)
vb, eb = betweenness(g)

# Define a function to display vertex name on hover
def on_hover_vertex(v_index):
    if v_index is not None:
        v_name = vertex_names[v_index]
        v_inspector.set_text(v_name)
    else:
        v_inspector.set_text("")

pos = sfdp_layout(g, eweight=e_weight, C=0.3,r=1,p=3)

state.draw(output="graph.pdf", pos=pos, vertex_text=g.vertex_index)

# Visualize the graph (basic example)
# graph_draw(g, vertex_text=g.vertex_index,
#            vertex_fill_color=prop_to_size(vb, 0, 1, power=.1), vertex_size=prop_to_size(vb, 3, 12, power=.2), vorder=vb, output="graph-sbm.pdf")

