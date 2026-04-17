 🧪 Hito 7: Pruebas Funcionales y Validación de Usuario

Objetivo: Realizar pruebas exhaustivas de todos los casos de uso implementados para validar que el sistema funciona correctamente en producción. Estas pruebas deben ser ejecutadas por el usuario final para garantizar la calidad y usabilidad del sistema.

**Estado Hito 7: ⏳ PENDIENTE (Ejecución por Usuario)**
- ⏳ Task 7.1: Pruebas de Conversión EDIFACT → JSONL (Ruta principal `/`)
- ⏳ Task 7.2: Pruebas de Compresión Opcional .fra en Flujo EDIFACT
- ⏳ Task 7.3: Pruebas de Ruta Especializada de Compresión (`/compression`)
- ⏳ Task 7.4: Pruebas del Sistema Zero-Config de Detección de Versión
- ⏳ Task 7.5: Pruebas de API REST Endpoints
- ⏳ Task 7.6: Pruebas de Interfaz de Usuario y Navegación
- ⏳ Task 7.7: Pruebas de Modos de Procesamiento (WASM vs API)
- ⏳ Task 7.8: Pruebas de Dashboard y Métricas
- ⏳ Task 7.9: Pruebas de Descargas
- ⏳ Task 7.10: Pruebas de Validación de Errores y Casos Límite

### Detalles de las Pruebas

#### Task 7.1: Pruebas de Conversión EDIFACT → JSONL
1. Subir archivo `.edi` o `.edifact` de ejemplo (`test.edi`)
2. Procesar en modo **Local (WASM)** y verificar:
   - Worker WASM carga correctamente
   - Conversión completa sin errores
   - DataGrid muestra datos convertidos
   - Labels semánticos visibles (no códigos EDIFACT crudos)
3. Procesar en modo **Backend (API)** y verificar:
   - API responde correctamente (backend ejecutando en `localhost:8080`)
   - Resultados consistentes con modo local
4. Descargar JSONL resultante y verificar formato válido
5. Descargar CSV y verificar conversión básica

#### Task 7.2: Pruebas de Compresión Opcional .fra en Flujo EDIFACT
1. En ruta principal (`/`), activar checkbox "Also compress to .fra"
2. Subir archivo EDIFACT y procesar
3. Verificar que se generan **dos archivos** para descarga:
   - JSONL (conversión directa)
   - .fra (compresión del JSONL)
4. Descargar archivo `.fra` y verificar que es binario (no texto)
5. Opcional: Usar archivo `.fra` generado en ruta `/compression` para descompresión

#### Task 7.3: Pruebas de Ruta Especializada de Compresión (`/compression`)
1. **Compresión JSONL → .fra**:
   - Subir archivo `.jsonl` o `.json`
   - Verificar que sistema detecta operación "Compress to .fra"
   - Procesar y descargar archivo `.fra`
   - Verificar reducción de tamaño (>90% típicamente)
2. **Descompresión .fra → JSONL**:
   - Subir archivo `.fra` (generado previamente)
   - Verificar que sistema detecta operación "Decompress to JSONL"
   - Procesar y descargar JSONL
   - Verificar que JSONL es idéntico al original (pérdida cero)
3. Probar ambos modos: **Local (WASM)** y **Backend (API)**

#### Task 7.4: Pruebas del Sistema Zero-Config de Detección de Versión
1. Usar archivo EDIFACT con versión `D96A` (ej. `test.edi`)
2. Verificar que sistema detecta automáticamente versión del segmento UNH
3. Confirmar que carga diccionario `standards/D96A.json`
4. **Prueba de scraping automático** (si no existe diccionario):
   - Eliminar `standards/D96A.json`
   - Procesar archivo EDIFACT
   - Verificar que sistema genera diccionario automáticamente
   - Confirmar que scraping funciona (requiere conexión a internet)

#### Task 7.5: Pruebas de API REST Endpoints
1. **Iniciar backend**: `cargo run --bin api --features api`
2. Probar cada endpoint con `curl` o Postman:
   - `POST /process/edifact` - Conversión EDIFACT → JSONL
   - `POST /process/jsonl` - Compresión JSONL → .fra
   - `POST /decompress/fra` - Descompresión .fra → JSONL
   - `POST /convert/json-to-edi` - Reconstrucción EDIFACT (opcional)
3. Verificar respuestas HTTP correctas (200 OK) y tipos MIME apropiados
4. Probar con archivos de diferentes tamaños

#### Task 7.6: Pruebas de Interfaz de Usuario y Navegación
1. Navegar entre rutas `/` y `/compression`
2. Verificar que navegación activa se resalta correctamente
3. Probar **Drag & Drop** con archivos válidos
4. Probar **click para seleccionar** archivos
5. Validar que sistema rechaza tipos de archivo no soportados
6. Verificar mensajes de error claros y útiles

#### Task 7.7: Pruebas de Modos de Procesamiento
1. **Modo Local (WASM)**:
   - Verificar que worker carga (mensaje "Worker ready")
   - Procesar archivo pequeño (WASM activo)
   - Verificar que no se requiere backend
2. **Modo Backend (API)**:
   - Cambiar toggle a "Backend (API)"
   - Procesar archivo (requiere backend ejecutándose)
   - Verificar comunicación con `localhost:8080`
3. **Fallback automático**: Si backend no disponible y modo API seleccionado, verificar mensaje de error apropiado

#### Task 7.8: Pruebas de Dashboard y Métricas
1. Procesar archivo EDIFACT y verificar Dashboard muestra:
   - Tamaño original vs procesado
   - Porcentaje de ahorro (para compresión)
   - Barra de progreso visual
   - Costos proyectados en la nube (mensual/anual)
2. Verificar cálculos correctos:
   - Ahorro porcentual = `(original - procesado) / original * 100`
   - Costos basados en $0.023/GB/mes (AWS S3 Standard)
3. Probar con diferentes tamaños de archivo

#### Task 7.9: Pruebas de Descargas
1. **Ruta principal (`/`)**:
   - Descargar JSONL (después de conversión EDIFACT)
   - Descargar CSV (exportación básica)
   - Descargar .fra (si checkbox activado)
2. **Ruta `/compression`**:
   - Descargar .fra (después de compresión JSONL)
   - Descargar JSONL (después de descompresión .fra)
3. Verificar que nombres de archivo son apropiados:
   - Mantienen nombre original (sin extensión)
   - Agregan extensión correcta (.jsonl, .csv, .fra)
4. Verificar que descargas funcionan en diferentes navegadores

#### Task 7.10: Pruebas de Validación de Errores y Casos Límite
1. **Archivos no soportados**: Subir `.txt`, `.pdf`, `.docx` - debe rechazar
2. **Archivos corruptos**: Subir EDIFACT mal formado - debe mostrar error claro
3. **Archivos vacíos**: Subir archivo vacío - manejo apropiado
4. **Archivos muy grandes**: Probar límites prácticos (depende de memoria navegador)
5. **Conexión internet**: Para scraping automático, probar sin conexión
6. **Backend no disponible**: En modo API sin backend ejecutándose
7. **Worker WASM falla**: Simular error de carga WASM

### Criterios de Aceptación
- ✅ Todas las pruebas ejecutadas exitosamente
- ✅ No hay regresiones en funcionalidad existente
- ✅ Interfaz de usuario responde adecuadamente
- ✅ Sistema maneja errores con mensajes claros
- ✅ Rendimiento aceptable para casos de uso típicos
- ✅ Documentación de problemas encontrados y soluciones

### 📋 Resultados de las Pruebas y Mejoras Identificadas

Las pruebas funcionales fueron ejecutadas por el usuario final y se identificaron 10 áreas de mejora crítica. Estos hallazgos han sido formalizados en el roadmap principal (`tareas.md`) como hitos 7-14:

1. **Internacionalización (i18n)** - Hito 7
2. **Páginas de Contenido (FAQs/About)** - Hito 8  
3. **Header Responsive** - Hito 9
4. **Validación de Archivos por Ruta** - Hito 10
5. **Mejoras UX (Botón Remove)** - Hito 11
6. **Arquitectura Serverless** - Hito 12
7. **Generador de Archivos de Prueba** - Hito 13
8. **Scraper Completo de Todas las Versiones** - Hito 14

*Actualizado: 2026‑04‑16*