import os
import wave
import ffmpeg
from typing import Union
import subprocess
import multiprocessing
import numpy as np
import cbor2 as cbor
import tempfile

"""
This is how I would normally do it:
1. We can use a different protocol say `gRPC` or `HTTP` to stream the wave file.

    Using this structure:
        {
            "nframes": int,
            "sampwidth": int,
            "framerate": int,
            "nchannels": int,
            "comptype": str,
            "compname": str,
            "frames": bytes,
        }

2. Then, we can use multiprocessing to read the file and send it to the other process

3. The other process can then use ffmpeg to convert the bytes to a format that we can use
"""

# # This part can be ignored when using a different protocol.


def load_audio(data: Union[str, bytes]) -> np.array:
    def read_file_to_bytes(data, conn):
        # Seen as serialized bytes.
        # This is the most important part; sending CBOR-seralized data to the ffmpeg_reader process
        conn.send(data)

    def ffmpeg_reader(conn, queue):
        output = conn.recv()
        # Unpickle the data
        output = cbor.loads(output)

        temp_file = os.path.join(tempfile.gettempdir(), "temp.wav")

        with wave.open(temp_file, "wb") as f:
            f.setparams(
                (
                    output["nchannels"],
                    output["sampwidth"],
                    output["framerate"],
                    output["nframes"],
                    output["comptype"],
                    output["compname"],
                )
            )
            f.writeframes(output["frames"])
        out, _ = (
            ffmpeg.input(temp_file)
            .output("-", format="s16le", acodec="pcm_s16le", ac=1, ar=16000)
            .run(cmd=["ffmpeg", "-nostdin"], capture_stdout=True, capture_stderr=True)
        )
        out = np.frombuffer(out, np.int16).flatten().astype(np.float32) / 32768.0

        queue.put(out)

    conn1, conn2 = multiprocessing.Pipe(True)
    queue: multiprocessing.Queue = multiprocessing.Queue()

    # Transform data into bytes if it is a string
    if isinstance(data, str):
        with wave.open(data, "rb") as f:
            out = dict(
                nframes=f.getnframes(),
                sampwidth=f.getsampwidth(),
                framerate=f.getframerate(),
                nchannels=f.getnchannels(),
                comptype=f.getcomptype(),
                compname=f.getcompname(),
                frames=f.readframes(f.getnframes()),
            )
            data = cbor.dumps(out)

    read = multiprocessing.Process(target=read_file_to_bytes, args=(data, conn1))
    reader = multiprocessing.Process(target=ffmpeg_reader, args=(conn2, queue))
    read.start()
    reader.start()

    return queue.get()
