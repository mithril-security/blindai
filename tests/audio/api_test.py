from blindai_preview import core, api
import os

# blindai code
if os.environ.get("BLINDAI_SIMULATION_MODE") == "true":
    connection = core.connect(
        addr="localhost", hazmat_http_on_unattested_port=True, simulation_mode=True
    )
else:
    connection = core.connect(addr="localhost", hazmat_http_on_unattested_port=True)

# Download audio file
output = os.path.join(os.path.dirname(__file__), "taunt.wav")
os.system(f"wget https://www2.cs.uic.edu/~i101/SoundFiles/taunt.wav -O {output}")
output = os.path.abspath(output)

# Transcribe audio file with BlindAI SGX
transcript = api.Audio.transcribe(file=output, connection=connection)
assert transcript == " Now go away, or I shall taunt you a second timer!"

try:
    api.Audio.transcribe(file=output, tee="sev")
except Exception as e:
    assert str(e) == "tee must be one of ['sgx', 'nitro']"
