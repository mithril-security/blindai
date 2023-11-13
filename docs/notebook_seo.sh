#!/bin/sh
set -e  # Exit if any command fails

sed -i '/<head>/a <meta name="description" content="Quick Tour of BlindAI API: Open-source Python library for private AI model access and Whisper model audio transcription.">' $READTHEDOCS_OUTPUT/html/docs/getting-started/quick-tour/index.html