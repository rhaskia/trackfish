import tensorflow as tf
import numpy as np
encoder = tf.saved_model.load('models/encoder16')
decoder = tf.saved_model.load('models/decoder16')
print(encoder.signatures.keys())
print(decoder.signatures.keys())

for var in decoder.variables:
    print(f"Name: {var.name}, Shape: {var.shape}")

for var in encoder.variables:
    print(f"Name: {var.name}, Shape: {var.shape}")
    #np.save(f'encoder_{var.name}.npy', var.numpy()) 

np.save('models/decoder_weights.npy', decoder.variables[0].numpy()) 
np.save('models/decoder_biases.npy', decoder.variables[1].numpy()) 
np.save('models/encoder_weights.npy', encoder.variables[0].numpy()) 
np.save('models/encoder_biases.npy', encoder.variables[1].numpy()) 
