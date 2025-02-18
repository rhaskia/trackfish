import tensorflow as tf
import numpy as np
from tkinter import *
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
import matplotlib.pyplot as plt

def clean(str1):
    no_whitespace = ''.join(str1.split()).lower()

    no_punc = ''.join(char for char in no_whitespace if char.isalnum())

    if no_punc.endswith("music"):
        return no_punc[:-5]
    else: 
        return no_punc

encoder = tf.saved_model.load('E:/rust/music/models/encoder16')
decoder = tf.saved_model.load('E:/rust/music/models/decoder16')

predict_fn = encoder.signatures["serving_default"]
decode = decoder.signatures["serving_default"]

with open("E:/rust/music/models/genrelist", "r") as file:
    genre_index = [clean(genre) for genre in file.readlines()]

def genre_to_vec(genre):
    index = -1

    try: 
        genre = clean(genre)
        index = genre_index.index(genre)
    except: 
        print(genre)

    encoding = tf.constant([1.0 if i == index else 0.0 for i in range(1083)], dtype=tf.float32)
    encoding = tf.reshape(encoding, (1, 1083))
    return predict_fn(encoding)["output_0"].numpy()[0]

def calculate(calc): 
    split = calc.split(" ")
    value = genre_to_vec(split.pop())
    print(value)

    while len(split) > 1:
        op = split.pop()
        next = genre_to_vec(split.pop())

        if op == "+":
            value = value + next
        elif op == "-":
            value = value - next
        elif op == "/":
            value = value / next
        elif op == "*":
            value = value * next
        else:
            return value

    return value



def to_genre(value):
    value = tf.constant(value)
    value = tf.reshape(value, (1, 16))
    decoded = decode(value)["output_0"].numpy()[0]
    indexes = decoded.argsort()[::-1][:5]
    return [genre_index[idx] for idx in indexes]

print(genre_to_vec("pop"))

m = Tk()
sv = StringVar()

w = Label(m, text='idk')
w.grid(row=1)

def callback(a, b, c):
    value = calculate(sv.get())
    print(value)
    new_value = ", ".join(to_genre(value))
    w.config(text=new_value)
    update_graph(value)
    return True

def update_graph(value):
    for rect, new_val in zip(bar_rects, value):
        rect.set_height(new_val)
    ax.set_ylim(min(value) - 0.1, max(value) + 0.1)  # Auto-scale
    canvas.draw()

sv.trace_add("write", callback)

Label(m, text='Calculation').grid(row=0)
e1 = Entry(m, textvariable=sv)
e1.grid(row=0, column=1)

fig, ax = plt.subplots(figsize=(5, 3))
bar_rects = ax.bar(range(16), np.zeros(16))  # Empty bar graph
ax.set_ylim(-1, 1)  # Set y-axis range
ax.set_xticks(range(16))
ax.set_title("Genre Vector Representation")

# Embed in Tkinter window
canvas = FigureCanvasTkAgg(fig, master=m)
canvas.get_tk_widget().grid(row=2, column=0, columnspan=2)

m.mainloop()
