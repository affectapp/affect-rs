#!/bin/bash
# Builds and deploys the affect server for production.
export SERVICE="affect-server"
export TAG="gcr.io/affect-app/affect-server"

# Build and push image.
docker build --tag ${TAG} . && \
  docker push ${TAG}
# gcloud builds submit \
#   --tag ${TAG} \
#   --machine-type=n1-highcpu-8 \
#    --timeout=3600s
if [ $? -ne 0 ]; then
  exit 2
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
if [ $? -ne 0 ]; then
  exit 2
fi

gcloud run services update-traffic \
  affect-server \
  --region=us-west2 \
  --to-latest