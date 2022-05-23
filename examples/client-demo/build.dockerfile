FROM jupyter/base-notebook AS base

# -- install blindai and dev dependencies
RUN pip install --no-cache-dir blindai \
    https://download.pytorch.org/whl/cpu/torch-1.11.0%2Bcpu-cp39-cp39-linux_x86_64.whl

# -- switch to root to install libgl
USER root

# -- libgl
RUN apt-get update --yes && apt-get install --yes --no-install-recommends libglib2.0-0 libgl1 && apt-get clean && rm -rf /var/lib/apt/lists/*

# -- switch back to normal user
USER ${NB_UID}

# -- copy examples
COPY --chown=${NB_UID} . .

# -- remove useless folders
RUN rm -rf work && \
    rm -rf client-demo