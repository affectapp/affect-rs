gcloud builds submit --tag gcr.io/affect-app/affect-server --timeout=3600s
gcloud run deploy affect-server --image gcr.io/affect-app/affect-server
