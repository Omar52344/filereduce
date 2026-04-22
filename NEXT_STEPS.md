# Próximos pasos para completar el Hito 15

## Tareas completadas

1. **Dockerfile multi-stage** creado y configurado para Google Cloud Run.
2. **Soporte de variable PORT** en la API (escucha en el puerto definido por la variable de entorno).
3. **Dependencias de Google Cloud Storage** agregadas (feature `gcs`).
4. **Sistema de notificación con broadcast channel** implementado (falta endpoint SSE).
5. **Documentación de despliegue** creada (`DEPLOYMENT.md`).
6. **Pipeline de Cloud Build** configurado (`cloudbuild.yaml`).
7. **Actualización del estado de tareas** en `tareas.md`.

## Tareas pendientes

### 1. Configurar Google Cloud Storage (Task 15.3)
- Implementar el módulo `storage` con soporte para pre-signed URLs.
- Integrar con la API para almacenar archivos grandes en GCS.
- Configurar credenciales de servicio en Cloud Run.

### 2. Completar Webhook/SSE (Task 15.4)
- Implementar endpoint `/events/{file_id}` con Server-Sent Events.
- Enviar actualizaciones a través del canal broadcast cuando cambie el estado de una tarea.
- Actualizar el frontend para suscribirse a eventos (opcional).

### 3. Pruebas de integración E2E (Task 15.6)
- Crear pruebas que simulen el flujo completo: subida, procesamiento, descarga.
- Ejecutar en entorno de CI/CD.

### 4. Configurar métricas, logging y alertas (Task 15.7)
- Integrar con Cloud Monitoring y Cloud Logging.
- Configurar alertas para errores y latencia.

### 5. Documentación de despliegue y rollback (Task 15.8)
- Completar `DEPLOYMENT.md` con pasos de rollback.
- Documentar gestión de secretos y variables de entorno.

### 6. Prueba de carga (Task 15.9)
- Realizar prueba de carga con herramienta como `k6` o `artillery`.

### 7. Pipeline CI/CD (Task 15.10)
- Configurar GitHub Actions para construir, probar y desplegar automáticamente.

## Instrucciones inmediatas

Para probar el despliegue en Google Cloud Run:

1. Construir la imagen Docker:
   ```
   docker build -t filereduce .
   ```

2. Probar localmente:
   ```
   docker run -p 8080:8080 -e PORT=8080 filereduce
   ```

3. Desplegar en Google Cloud Run (requiere `gcloud`):
   ```
   gcloud run deploy filereduce --image gcr.io/PROJECT-ID/filereduce --platform managed --region us-central1 --allow-unauthenticated
   ```

## Notas

- Las credenciales de GCP deben gestionarse mediante Secret Manager o variables de entorno.
- El frontend Next.js se sirve junto con la API en el mismo contenedor (puerto 3000). Asegúrate de que el rewrite en `next.config.ts` apunte a la URL correcta de la API.
- Para producción, considera separar frontend y backend en servicios independientes.

## Solución de problemas

Si la compilación falla debido a errores de edición de Rust, verifica que `Cargo.toml` tenga `edition = "2021"`. Si persisten, ejecuta `cargo update` y `cargo clean`.

Para más detalles, consulta los archivos creados:
- `Dockerfile`
- `DEPLOYMENT.md`
- `cloudbuild.yaml`
- `tareas.md` (actualizado)