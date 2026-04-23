# Próximos pasos después del Hito 15

## ✅ Hito 15: COMPLETADO

El **Hito 15 (Escalabilidad Híbrida y Orquestación Cloud)** ha sido **completado exitosamente**. Todas las tareas están implementadas y funcionando:

### Tareas completadas del Hito 15

1. **Task 15.1 – Smart Switcher en Frontend**: Lógica de decisión basada en tamaño de archivo (100 MB threshold) en `FileUpload.tsx`. Función `processWithCloud` implementada con timeout y manejo de errores. UI con barra de progreso y estados visuales para procesamiento cloud.

2. **Task 15.2 – API de Procesamiento Asíncrono**: Refactorización de `src/bin/api.rs` con sistema de colas en memoria (`HashMap<Uuid, TaskStatus>`). Workers en background con `tokio::spawn_blocking`. Endpoints:
   - `POST /upload/request` – solicitud de URL firmada para subida directa.
   - `POST /process/cloud/{file_id}` – inicia procesamiento asíncrono (operaciones `"edifact"`, `"jsonl"`, `"fra"`).
   - `GET /status/{file_id}` – consulta estado de tarea (Pending, Processing, Completed, Failed).
   - `GET /download/{file_id}` – descarga de resultados (JSONL o .fra).

3. **Task 15.3 – Gestión de Storage con Pre‑signed URLs**: Integración condicional de Google Cloud Storage (GCS) y MemoryStorage. Si la variable de entorno `GCS_BUCKET` está definida y la feature `gcs` está activa, se usa GCS; de lo contrario, MemoryStorage. Módulo GCS compila correctamente y está listo para usar con credenciales de Google Cloud. Upload y download funcionando.

4. **Task 15.4 – Webhook de Finalización y Polling**: Endpoint SSE `/events` implementado y funcionando, emite eventos de conexión y actualizaciones de tareas. Canal broadcast operativo. Frontend realiza polling cada 5 segundos para actualizar UI automáticamente. Manejo de reconexión y timeouts.

5. **Task 15.5 – Dockerfile optimizado para Google Cloud Run**: Dockerfile multi‑stage creado, soporte de variable `PORT`. Imagen ligera lista para despliegue en Cloud Run.

### Estado actual del sistema

- **Smart Switching operativo**: Archivos < 100 MB se procesan localmente con WASM; archivos ≥ 100 MB se envían automáticamente a la API cloud.
- **Procesamiento asíncrono**: Colas en memoria garantizan que tareas largas no bloqueen el event loop.
- **Comunicación en tiempo real**: SSE permite notificaciones push (opcional para frontend; actualmente se usa polling).
- **Storage escalable**: GCS integrado condicionalmente; pre‑signed URLs permiten subida/descarga directa sin pasar por el servidor.
- **Frontend actualizado**: Componente `FileUpload.tsx` refactorizado para usar el flujo asíncrono completo.

## 🚀 Próximos pasos (post‑Hito 15)

### 1. Despliegue en Google Cloud Run
- Ejecutar el Dockerfile multi‑stage en Cloud Run con las credenciales GCS configuradas.
- Configurar variables de entorno (`GCS_BUCKET`, `GOOGLE_APPLICATION_CREDENTIALS`).
- Verificar que la API responda correctamente en el dominio desplegado.

### 2. Pruebas de carga
- Validar el sistema con archivos grandes (≥ 100 MB) para verificar el Smart Switching y la escalabilidad.
- Simular múltiples usuarios concurrentes procesando archivos EDIFACT/JSONL.
- Medir tiempos de respuesta y uso de recursos.

### 3. Monitoreo y métricas
- Añadir logging estructurado (`tracing`, `opentelemetry`).
- Integrar con Cloud Monitoring y Cloud Logging.
- Configurar alertas para errores y latencia elevada.

### 4. Optimización de frontend
- Implementar SSE en el frontend para actualizaciones en tiempo real en lugar de polling.
- Mejorar manejo de errores y reintentos automáticos.
- Añadir indicadores visuales más detallados del estado de procesamiento cloud.

### 5. Caché de diccionarios
- Almacenar diccionarios generados por el scraper en un repositorio central (ej. Google Cloud Storage) para evitar re‑scraping entre instancias.
- Implementar sistema de actualizaciones periódicas para detectar nuevas versiones EDIFACT.

### 6. Pruebas de integración end‑to‑end
- Crear pruebas automatizadas que simulen el flujo completo: subida, procesamiento, descarga.
- Ejecutar en entorno de CI/CD (GitHub Actions, Cloud Build).
- Cubrir escenarios de error (timeouts, archivos corruptos, credenciales inválidas).

### 7. Pipeline CI/CD completo
- Configurar GitHub Actions para construir, probar y desplegar automáticamente en Cloud Run.
- Añadir etapas de linting, seguridad y pruebas de regresión.
- Implementar despliegue canario o blue‑green (opcional).

## 📋 Instrucciones inmediatas

Para probar el despliegue en Google Cloud Run:

1. **Construir la imagen Docker**:
   ```bash
   docker build -t filereduce .
   ```

2. **Probar localmente**:
   ```bash
   docker run -p 8080:8080 -e PORT=8080 -e GCS_BUCKET=filerecudebucket1 -e GOOGLE_APPLICATION_CREDENTIALS=/path/to/creds.json filereduce
   ```

3. **Desplegar en Google Cloud Run** (requiere `gcloud`):
   ```bash
   gcloud run deploy filereduce --image gcr.io/PROJECT-ID/filereduce --platform managed --region us-central1 --allow-unauthenticated --set-env-vars=GCS_BUCKET=filerecudebucket1
   ```

## 📝 Notas

- Las credenciales de GCP deben gestionarse mediante Secret Manager o variables de entorno en Cloud Run.
- El frontend Next.js se sirve junto con la API en el mismo contenedor (puerto 3000). Asegúrate de que el rewrite en `next.config.ts` apunte a la URL correcta de la API.
- Para producción, considera separar frontend y backend en servicios independientes para mayor escalabilidad.

## 🔧 Solución de problemas

- Si la compilación falla debido a errores de edición de Rust, verifica que `Cargo.toml` tenga `edition = "2021"`. Si persisten, ejecuta `cargo update` y `cargo clean`.
- Para errores de async/await, revisa que todas las futures sean `Send` y no capturen referencias que crucen awaits.
- Si el módulo GCS no compila, asegúrate de que la feature `gcs` esté activada y las dependencias `google-cloud-storage` estén correctamente especificadas.

Para más detalles, consulta los archivos creados:
- `Dockerfile`
- `DEPLOYMENT.md`
- `cloudbuild.yaml`
- `tareas.md` (actualizado con el estado completo del proyecto)
