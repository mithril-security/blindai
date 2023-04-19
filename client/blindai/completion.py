from typing import Optional
from pydantic import BaseModel

from .nitro_client import BlindAiNitroConnection


class PredictionMsg(BaseModel):
    input_text: str


NITRO_BLINDAI_ADDR = f"nitro.mithrilsecurity.io"
DEFAULT_TEE_OPTIONS = ["nitro"]
DEFAULT_TEE = "nitro"


class Completion:
    @classmethod
    def create(
        cls,
        prompt: str,
        connection: Optional[BlindAiNitroConnection] = None,
        tee: Optional[str] = DEFAULT_TEE,
    ) -> str:
        prompt = f"<human>: {prompt}\n<bot>: "

        if tee == "nitro":
            if connection is None:
                connection = BlindAiNitroConnection(NITRO_BLINDAI_ADDR, debug_mode=True)
            
            with connection as req:
                res = req.api(
                    "post",
                    "/open-chat-kit/predict",
                    data=PredictionMsg(input_text=prompt).json(),
                )
            
            return res
        else:
            raise ValueError(f"tee must be one of {DEFAULT_TEE_OPTIONS}")