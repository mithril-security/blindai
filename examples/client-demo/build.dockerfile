FROM jupyter/base-notebook AS base

# -- install blindai and dev dependencies
RUN pip install --no-cache-dir blindai \
    https://download.pytorch.org/whl/cpu/torch-1.11.0%2Bcpu-cp39-cp39-linux_x86_64.whl

# -- copy examples
COPY --chown=${NB_UID} . .

# -- remove useless folders
RUN rm -rf work && \
    rm -rf client-demo