> **:warning: Warning : BlindAI Preview**<br />
> This is a preview version of BlindAI named blindai-preview. It is still under development and has not yet all the features of the current BlindAI.
> You can find the current BlindAI [here](https://github.com/mithril-security/blindai).

**BlindAI** is a **fast, easy-to-use,** and **confidential inference server**, allowing you to easily and quickly deploy your AI models with privacy, **all in Python**. Thanks to the **end-to-end protection guarantees**, data owners can send private data to be analyzed by AI models, without fearing exposing their data to anyone else.

To interact with an AI model hosted on a remote secure enclave, we provide the `blindai_preview.client` API. This client will:

- check that we are talking to a genuine secure enclave with the right security features
- upload an AI model that was previously converted to ONNX
- query the model securely

## Getting Started

> **:warning: Warning**<br />
> The following instructions assume you already have access to a configured SGX enabled machine, such as one provided by Mithril Security.
> If that is not the case, you can either ask for one, or read [the Documentation](https://blindai-preview.mithrilsecurity.io/en/latest/docs/cloud-deployment).

Start by cloning the repo with --recursive to get the submodules.

> **:warning: Warning**<br />
>     If you are on an Azure machine, you should replace the .devcontainer folder by the devcontainer-azure/.devcontainer folder.
Open it in vscode and `Reopen in Container`.

Then, at the root of the project:

```
cd client
poetry install
poetry shell
```

Which creates a virtual environment and install the client sdk.


## justfile overview

You can use the justfile to do recurrent developer tasks.

- ```just run [args]``` to run the server in debug mode
- ```just release [args]```to run the server in release mode
- ```just test``` to run all the tests
- ```just doc``` to build and serve the doc


## Testing

First, generates all of the necessary onnx and inputs (npz) files with:
```
bash generate_all_onnx_and_npz.sh
```

Then ```just test``` run the client unittests, the end-to-end tests, and serve a pretty coverage report.
