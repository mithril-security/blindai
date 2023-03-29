from pydantic import BaseModel


class WhisperParams(BaseModel):
    sample_rate: int = 16000
