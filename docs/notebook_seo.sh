#!/bin/sh
set -e  # Exit if any command fails

sed -i '/<head>/a <meta name="description" content="Quick Tour of BlindAI API: Open-source Python library for private AI model access and Whisper model audio transcription.">' $READTHEDOCS_OUTPUT/html/docs/getting-started/quick-tour/index.html
sed -i '/<head>/a <meta name="description" content="Secure AI model deployment with BlindAI for private Covid-19 diagnosis. Data privacy with SGX enclaves.">' $READTHEDOCS_OUTPUT/html/docs/how-to-guides/covid_net_confidential/index.html
sed -i '/<head>/a <meta name="description" content="Transcribe audio privately with Whisper and BlindAI API: Install Python library, transcribe securely with hardware attestation.">' $READTHEDOCS_OUTPUT/html/docs/tutorials/api/whisper_tutorial/index.html
sed -i '/<head>/a <meta name="description" content="Upload and test ML models securely on BlindAI server: Ensure user data privacy and security during AI model deployment.">' $READTHEDOCS_OUTPUT/html/docs/tutorials/core/uploading_models/index.html