FROM jupyter/scipy-notebook AS base

# -- install blindai and dev dependencies
RUN pip install blindai \
    onnxruntime \
    pillow \
    numpy \
    torch \
    pandas \
    transformers \
    opencv-python

# -- copy examples
COPY . .

# -- fix potential write/read access errors
USER root
RUN chown -R ${NB_UID} ${HOME}
USER ${NB_UID}

# -- remove useless folders
RUN rm -rf work
RUN rm -rf client-demo