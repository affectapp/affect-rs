SERVICE="affect-server"
TAG="gcr.io/affect-app/affect-server"

gcloud builds submit \
  --tag ${TAG} \
  --machine-type=n1-highcpu-8 \
   --timeout=3600s

gcloud run deploy ${SERVICE} \
  --image ${TAG} \
  --update-secrets=CONFIG=server-config:latest
