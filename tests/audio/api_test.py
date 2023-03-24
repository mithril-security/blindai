from blindai_preview import core, api
import os

# blindai code
if os.environ.get("BLINDAI_SIMULATION_MODE") == "true":
    connection = core.connect(
        addr="localhost", hazmat_http_on_unattested_port=True, simulation_mode=True
    )
else:
    connection = core.connect(addr="localhost", hazmat_http_on_unattested_port=True)

response = connection.upload_model(model="whisper.onnx")

# Download audio file
output = os.path.join(os.path.dirname(__file__), "taunt.wav")
os.system(f"wget https://www2.cs.uic.edu/~i101/SoundFiles/taunt.wav -O {output}")

# Transcribe audio file
transcript = api.Audio.transcribe(file=output)
assert transcript == " Now go away, or I shall taunt you a second timer!"
