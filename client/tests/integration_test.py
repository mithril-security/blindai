from pathlib import Path
import blindai_preview

# response = client_v2.upload_model(model=model_path)
# run_response = client_v2.run_model(model_id=response.model_id, input_tensors=inputs, sign=False)
# client_v2.delete_model(model_id = response.model_id)


def test_connect():
    client = blindai_preview.connect(
        addr="localhost", hazmat_http_on_unattested_port=True
    )


# TODO: make a real test from this :
# def test_connect_custom_manifest():
#     client = blindai_preview.connect(addr="localhost", hazmat_manifest_path = Path("/workspaces/blindai-preview/manifest.dev.toml"))
