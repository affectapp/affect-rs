#!/bin/bash
# Builds and deploys the affect server for production.
export SERVICE="affect-server"
export TAG="gcr.io/affect-app/affect-server"

# Build image on cloud.
gcloud builds submit \
  --tag ${TAG} \
  --machine-type=n1-highcpu-8 \
   --timeout=3600s

# Run image on cloud.
gcloud run deploy ${SERVICE} \
  --image ${TAG} \
  --update-secrets=CONFIG=server-config:latest
