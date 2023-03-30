from blindai import api
import os

# Download audio file
output = os.path.join(os.path.dirname(__file__), "taunt.wav")
os.system(f"wget https://www2.cs.uic.edu/~i101/SoundFiles/taunt.wav -O {output}")
output = os.path.abspath(output)


# Transcribe audio file with BlindAI Nitro
transcript = api.Audio.transcribe(file=output, tee="nitro")
assert transcript == " Now go away, or I shall taunt you a second timer!"
