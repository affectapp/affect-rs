#!/bin/bash
# Builds and deploys the affect server for production.
export SERVICE="affect-server"
export TAG="gcr.io/affect-app/affect-server"

# Build image.
docker build --tag ${TAG} .
if [ $? -ne 0 ]; then
  kill 0
fi

# Push image.
docker push ${TAG}

# gcloud builds submit \
#   --tag ${TAG} \
#   --machine-type=n1-highcpu-8 \
#    --timeout=3600s

if [ $? -ne 0 ]; then
  kill 0
fi

# Run image on cloud.
gcloud beta run deploy ${SERVICE} \
  --image ${TAG} \
  --update-secrets=CONFIG=server-config:latest \
  --execution-environment=gen1 \
  --allow-unauthenticated \
  --cpu-throttling \
  --cpu=1 \
  --region=us-west2