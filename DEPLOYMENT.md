# Despliegue en Google Cloud Run

Este documento describe el proceso para desplegar FileReduce en Google Cloud Run.

## Prerrequisitos

- Cuenta de Google Cloud con facturación habilitada.
- Google Cloud SDK (`gcloud`) instalado y configurado.
- Docker instalado localmente (para builds locales).

## Configuración del proyecto

1. Crear un nuevo proyecto en Google Cloud Console o usar uno existente.
2. Habilitar las APIs necesarias:
   - Cloud Run API
   - Cloud Build API
   - Container Registry API
   - Cloud Storage API (para almacenamiento de archivos)

3. Configurar un bucket de Cloud Storage para caché de archivos:
   ```bash
   gcloud storage buckets create gs://filereduce-cache --location=us-central1
   ```

4. Crear una cuenta de servicio con permisos adecuados:
   ```bash
   gcloud iam service-accounts create filereduce-sa \
     --display-name="FileReduce Service Account"
   gcloud projects add-iam-policy-binding $PROJECT_ID \
     --member="serviceAccount:filereduce-sa@$PROJECT_ID.iam.gserviceaccount.com" \
     --role="roles/storage.objectAdmin"
   gcloud projects add-iam-policy-binding $PROJECT_ID \
     --member="serviceAccount:filereduce-sa@$PROJECT_ID.iam.gserviceaccount.com" \
     --role="roles/run.invoker"
   ```

5. Generar una clave de cuenta de servicio y guardarla como secreto:
   ```bash
   gcloud iam service-accounts keys create credentials.json \
     --iam-account=filereduce-sa@$PROJECT_ID.iam.gserviceaccount.com
   ```

## Construcción de la imagen Docker

El proyecto incluye un Dockerfile multi-stage optimizado para Cloud Run.

Para construir localmente:

```bash
docker build -t gcr.io/$PROJECT_ID/filereduce:latest .
```

## Despliegue en Cloud Run

### Opción 1: Usar Cloud Build

Crear un archivo `cloudbuild.yaml`:

```yaml
steps:
  - name: 'gcr.io/cloud-builders/docker'
    args: ['build', '-t', 'gcr.io/$PROJECT_ID/filereduce:$COMMIT_SHA', '.']
  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', 'gcr.io/$PROJECT_ID/filereduce:$COMMIT_SHA']
  - name: 'gcr.io/google.com/cloudsdktool/cloud-sdk'
    entrypoint: gcloud
    args:
      - 'run'
      - 'deploy'
      - 'filereduce'
      - '--image'
      - 'gcr.io/$PROJECT_ID/filereduce:$COMMIT_SHA'
      - '--region'
      - 'us-central1'
      - '--platform'
      - 'managed'
      - '--allow-unauthenticated'
      - '--set-env-vars'
      - 'GOOGLE_APPLICATION_CREDENTIALS=/app/credentials.json'
      - 'PORT=8080'
    secretEnv: ['GOOGLE_APPLICATION_CREDENTIALS_DATA']
secrets:
  - kmsKeyName: projects/$PROJECT_ID/locations/global/keyRings/my-key-ring/cryptoKeys/my-key
    secretEnv:
      GOOGLE_APPLICATION_CREDENTIALS_DATA: 'encrypted_credentials'
```

### Opción 2: Despliegue manual

```bash
# Subir imagen a Container Registry
docker push gcr.io/$PROJECT_ID/filereduce:latest

# Desplegar en Cloud Run
gcloud run deploy filereduce \
  --image gcr.io/$PROJECT_ID/filereduce:latest \
  --region us-central1 \
  --platform managed \
  --allow-unauthenticated \
  --set-env-vars PORT=8080 \
  --service-account filereduce-sa@$PROJECT_ID.iam.gserviceaccount.com
```

## Variables de entorno

| Variable | Descripción | Valor por defecto |
|----------|-------------|-------------------|
| `PORT` | Puerto donde escucha la API | 8080 |
| `GOOGLE_APPLICATION_CREDENTIALS` | Ruta a las credenciales de servicio | `/app/credentials.json` |
| `GCS_BUCKET` | Nombre del bucket de Cloud Storage | `filereduce-cache` |
| `RUST_LOG` | Nivel de logging de Rust | `info` |

## Almacenamiento en Cloud Storage

FileReduce puede usar Google Cloud Storage para almacenar archivos subidos y resultados. Para habilitarlo:

1. Configurar la variable de entorno `GCS_BUCKET`.
2. Montar las credenciales de servicio como volumen en Cloud Run.
3. La API generará URLs pre-firmadas para acceso directo a los archivos.

## Métricas y Logging

Cloud Run integra automáticamente con Cloud Monitoring y Cloud Logging. Para métricas personalizadas:

- Los logs de la aplicación se envían a Cloud Logging.
- Las métricas de solicitudes, latencia y errores están disponibles en Cloud Monitoring.
- Configurar alertas para errores HTTP 5xx o alta latencia.

## Pruebas de integración E2E

Ejecutar las pruebas de integración localmente:

```bash
cargo test --features api -- --test-threads=1
```

Para pruebas en el entorno de Cloud Run, usar un script de post-despliegue que verifique los endpoints de salud y funcionalidad básica.

## Rollback

Para revertir a una versión anterior:

```bash
gcloud run revisions list --service filereduce --region us-central1
gcloud run deploy filereduce --image gcr.io/$PROJECT_ID/filereduce:<REVISION_SHA>
```

## Costos estimados

- Cloud Run: según uso de CPU y memoria, tráfico.
- Cloud Storage: según almacenamiento y operaciones.
- Cloud Build: según tiempo de construcción.

Consulte la [calculadora de precios de Google Cloud](https://cloud.google.com/products/calculator).

## Solución de problemas

- **Error "Permission denied"**: Verificar permisos de la cuenta de servicio.
- **La API no responde**: Verificar logs en Cloud Logging.
- **Archivos no persisten**: Cloud Run tiene sistema de archivos efímero; usar Cloud Storage para persistencia.
- **Frontend no carga**: Verificar que el puerto 8080 esté expuesto y que el rewrite de Next.js apunte a la URL correcta.