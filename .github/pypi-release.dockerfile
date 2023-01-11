# Minimal image to build the release version of the sgx enclave
FROM python:3.10.9-alpine3.17
WORKDIR blindai-preview
COPY client client
RUN pip install poetry && cd client && poetry config repositories.testpypi https://test.pypi.org/legacy/ && \
    poetry config pypi-token.testpypi $API_TOKEN_PYPI && poetry publish --build --repository testpypi