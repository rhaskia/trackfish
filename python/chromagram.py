import librosa
import librosa.display
import numpy as np
import matplotlib.pyplot as plt

y, sr = librosa.load("/home/rhaskia/Downloads/octave.wav", sr=None)  # Preserve original sample rate
tuning = librosa.estimate_tuning(y=y, sr=sr, n_fft=2048)
print(tuning)
chroma_fb = librosa.filters.chroma(sr=44100, n_fft = 2048, tuning = tuning)
np.save("chroma.npy", chroma_fb)
print(chroma_fb.shape)

chromagram = librosa.feature.chroma_stft(y=y, sr=sr)

plt.figure(figsize=(10, 5))
librosa.display.specshow(chromagram, x_axis="time", y_axis="chroma", cmap="coolwarm")

plt.colorbar(label="Intensity")
plt.title("Chromagram")
plt.xlabel("Time")
plt.ylabel("Pitch Class")

plt.savefig("chromagram.png", dpi=300, bbox_inches="tight")
plt.close()


