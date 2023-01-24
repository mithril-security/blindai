# Minimal image to build the release version of the Python client
FROM python:3.10.9-alpine3.17
WORKDIR blindai-preview
COPY client client
RUN pip install poetry 
CMD cd client \
    && poetry config pypi-token.pypi $API_TOKEN_PYPI \
    && poetry publish --build