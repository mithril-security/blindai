import whisper
from typing import Optional, Union
from .utils import fetch_whisper_tiny_20_tokens
from .client import BlindAiConnection, connect
from transformers import WhisperProcessor
import torch
from ._preprocess_audio import load_audio

DEFAULT_BLINDAI_ADDR = "4.246.205.63"
DEFAULT_WHISPER_MODEL = "tiny.en"
DEFAULT_TEE_OPTIONS = ["sgx", "nitro"]
DEFAULT_TEE = "sgx"
DEFAULT_TRANSFORMER = f"openai/whisper-{DEFAULT_WHISPER_MODEL}"
DEFAULT_MODEL_HASH = "ff63656d9b09514efbb38b4b69324280a86b55df5e3a2268cb79e812d8c7b863"


def _preprocess_audio(file: Union[str, bytes]) -> torch.Tensor:
    """
    Preprocess audio file to be used with Whisper model.

    Args:
        file: str
            Audio file to preprocess

    Returns:
        torch.Tensor:
            The preprocessed audio file
    """
    # Load audio file
    audio = load_audio(file).flatten()

    # Pad or trim ndarray to [80, 3000]
    audio = whisper.pad_or_trim(audio)

    # Convert loaded audio to log_mel_spectrogram
    return whisper.log_mel_spectrogram(audio).unsqueeze(0)


def _get_connection(
    connection: Optional["BlindAiConnection"], tee: Optional[str]
) -> "BlindAiConnection":
    """
    Get the BlindAI connection object.

    Args:
        connection: Optional[BlindAiConnection]
            The BlindAI connection object

    Returns:
        BlindAiConnection:
            The BlindAI connection object
    """
    if connection is None:
        connection = connect(
            DEFAULT_BLINDAI_ADDR,
            hazmat_http_on_unattested_port=True,
            use_cloud_manifest=True,
        )

    return connection


class Audio:
    @classmethod
    def transcribe(
        cls,
        file: Union[str, bytes],
        model: str = DEFAULT_WHISPER_MODEL,
        connection: Optional["BlindAiConnection"] = None,
        tee: Optional[str] = DEFAULT_TEE,
    ) -> str:
        """
        BlindAI Whisper API which converts speech to text based on the model passed.

        Args:

            file: str, bytes
                Audio file to transcribe. It may also receive serialized bytes of wave file.
            model: str
                The Whisper model. Defaults to "medium".
            connection: Optional[BlindAiConnection]
                The BlindAI connection object. Defaults to None.
            tee: Optional[str]
                The Trusted Execution Environment to use. Defaults to "sgx". Unused, at the moment.
        Returns:
            Dict:
                The transcription object containing, text and the tokens
        """

        # Check which TEE to use
        if tee not in DEFAULT_TEE_OPTIONS:
            raise ValueError(f"tee must be one of {DEFAULT_TEE_OPTIONS}")

        # Preprocess audio file
        input_mel = _preprocess_audio(file)

        # Get BlindAI connection object
        with _get_connection(connection, tee) as conn:
            # Run ONNX model with `input_array` on BlindAI server
            res = conn.run_model(model_hash=DEFAULT_MODEL_HASH, input_tensors=input_mel)

            # Convert each output BlindAI Tensor object into PyTorch Tensor
            res = [t.as_torch() for t in res.output]  # type: ignore

            # Load the Whisper Transformer object.
            tokenizer = WhisperProcessor.from_pretrained(DEFAULT_TRANSFORMER)

            # Extract tokens from result
            tokens = res[0][0].numpy()  # type: ignore

            # Use transform to decode tokens
            text = tokenizer.batch_decode(tokens, skip_special_tokens=True).pop()

            return text
