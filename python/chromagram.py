import librosa
import librosa.display
import numpy as np
import matplotlib.pyplot as plt

chroma_fb = librosa.filters.chroma(sr=44100, n_fft = 2048)
print(chroma_fb.shape)
np.save("chroma.npy", chroma_fb)

y, sr = librosa.load("/home/rhaskia/Downloads/octave.wav", sr=None)  # Preserve original sample rate

chromagram = librosa.feature.chroma_stft(y=y, sr=sr)

plt.figure(figsize=(10, 5))
librosa.display.specshow(chromagram, x_axis="time", y_axis="chroma", cmap="coolwarm")

plt.colorbar(label="Intensity")
plt.title("Chromagram")
plt.xlabel("Time")
plt.ylabel("Pitch Class")

plt.savefig("chromagram.png", dpi=300, bbox_inches="tight")
plt.close()


