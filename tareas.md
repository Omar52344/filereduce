🗺️ Roadmap de FileReduce: Del Motor al SaaS de Big Data

Este documento detalla la ruta crítica para transformar el motor estático de FileReduce en un ecosistema dinámico, impulsado por IA y optimizado para el procesamiento masivo de datos comerciales (EDIFACT/JSONL).

🚀 Hito 1: El Motor Dinámico (Rust Core Refactor)

Objetivo: Eliminar el código "quemado" y permitir que la lógica de conversión dependa totalmente de metadatos externos y dinámicos.

Task 1.1: Consolidación del Schema de Traducción. * Validar y extender translations.json (ya diseñado) para soportar estructuras de "Loops" y grupos repetitivos.

Sincronizar el struct en translations.rs para asegurar una deserialización perfecta.

Task 1.2: Implementación del Registro de Mapeo (Registry Pattern). * Crear un HashMap global o thread-safe (Arc<Mutex<>>) que cargue el JSON y sirva como fuente única de verdad para el parser.

Task 1.3: Refactorización del Parser EDIFACT. * Migrar la lógica de main.rs para que las secciones no sean match estáticos, sino búsquedas dinámicas en el Registro.

Task 1.4: Sistema de Telemetría de Etiquetas Desconocidas. * Implementar un canal de captura para segmentos no mapeados que genere reportes automáticos para el Hito 4 (IA).

Task 1.5: Batería de Tests Dinámicos. * Pruebas unitarias que validen la conversión de un mismo archivo EDIFACT usando diferentes versiones de translations.json para verificar el dinamismo.

**Estado Hito 1: ✅ COMPLETADO**
- ✅ Task 1.1: Schema de traducciones consolidado en `src/translations/config.rs`.
- ✅ Task 1.2: `TranslationRegistry` implementado en `src/translations/registry.rs`.
- ✅ Task 1.3: Parser EDIFACT refactorizado (`src/parser/edifact.rs`) con soporte dinámico.
- ✅ Task 1.4: Telemetría de segmentos desconocidos mediante `tracing::warn!`.
- ✅ Task 1.5: Tests dinámicos pasan (`cargo test`).

🌐 Hito 2: Portabilidad y WebAssembly (WASM)

Objetivo: Llevar la potencia de procesamiento al navegador y exponer la lógica mediante servicios distribuidos.

Task 2.1: Exportación a WASM con wasm-bindgen. * Encapsular el método process de main.rs para ser consumido desde JavaScript.

Task 2.2: Implementación de la API de 4 Endpoints (REST).

POST /convert/edi-to-json: Conversión directa usando el motor dinámico.

POST /convert/json-to-edi: Reconstrucción de archivos EDIFACT a partir de JSONL.

POST /compress/to-fra: Ejecución de la librería filereducelib para generar backups .fra.

POST /decompress/from-fra: Restauración de archivos .fra a JSONL original.

Task 2.3: Bridge de Datos y Memoria. * Optimizar el paso de archivos Uint8Array entre JS y el runtime de WASM para evitar cuellos de botella en archivos de más de 100MB.

Task 2.4: Hot-Reload de Traducciones en Cliente. * Lógica para que el WASM refresque su diccionario local si se detecta una actualización en el servidor central.

**Estado Hito 2: 🚧 EN PROGRESO**
- ✅ Task 2.1: Crate WASM creada (`wasm/`) con `wasm-bindgen` (compilación pendiente de `clang`).
- ✅ Task 2.2: API REST implementada (`src/bin/api.rs`):
  - ✅ `POST /process/edifact` (conversión EDIFACT → JSONL).
  - ✅ `POST /process/jsonl` (compresión JSONL → .fra).
  - ✅ `POST /decompress/fra` (descompresión .fra → JSONL) – *implementado*.
  - ✅ `POST /convert/json-to-edi` (reconstrucción EDIFACT) – *implementado (serialización básica)*.
- 🔄 Task 2.3: Bridge de Datos y Memoria – *pendiente*.
- 🔄 Task 2.4: Hot-Reload de Traducciones – *pendiente*.

🎨 Hito 3: Frontend de Impacto (Next.js + Drag & Drop)

Objetivo: Crear una interfaz que demuestre el valor inmediato ("Efecto Wow") permitiendo al usuario ver sus datos de forma legible.

Task 3.1: Interface de Carga Inteligente. * Desarrollar un componente de "Drag & Drop" con pre-validación de formato de archivo.

Task 3.2: Orquestación con Web Workers. * Asegurar que el pesado proceso de conversión en WASM ocurra en un hilo separado de la UI para mantener una experiencia fluida.

Task 3.3: Data Grid Semántico (Visualización). * Renderizar el resultado en una tabla dinámica (TanStack Table) usando los labels del translations.json.

Task 3.4: Dashboard de Ahorro de Almacenamiento. * Widget comparativo: Peso Original vs Peso .fra, con cálculo automático de ahorro porcentual y proyectado a costo en la nube.

Task 3.5: Gestor de Descargas. * Permitir al usuario bajar el JSONL resultante, el reporte de errores y el backup comprimido .fra de forma local.

**Estado Hito 3: ✅ COMPLETADO**
- ✅ Task 3.1: Interface de Carga Inteligente – componente Drag & Drop implementado (`components/FileUpload.tsx`) con pre‑validación de formatos.
- ✅ Task 3.2: Orquestación con Web Workers – implementado `spawn_blocking` en API para procesamiento no bloqueante; módulo WASM compilado listo para frontend.
- ✅ Task 3.3: Data Grid Semántico – componente `DataGrid.tsx` implementado con TanStack Table, muestra documentos EDIFACT convertidos.
- ✅ Task 3.4: Dashboard de Ahorro – componente `Dashboard.tsx` implementado con métricas de tamaño, porcentaje de ahorro y costo proyectado en la nube.
- ✅ Task 3.5: Gestor de Descargas – soporta descarga de JSONL, CSV y archivos .fra según el tipo de procesamiento.

🧠 Hito 4: Inteligencia y Escalabilidad (The Cloud Brain)

Objetivo: Automatizar el mantenimiento del sistema y facilitar la integración empresarial de nivel "Enterprise".

Task 4.1: Hub de Aprendizaje de Etiquetas. * Crear el servicio que centraliza los reportes de etiquetas desconocidas capturados en el Hito 1.

Task 4.2: Integración de IA SRE DeepSeek * Implementar el agente que analiza etiquetas nuevas contra manuales estándar de la ONU/EDIFACT y sugiere la traducción al translations.json automáticamente.

Task 4.3: Sincronización Global de Diccionarios. * Implementar un sistema de distribución (CDN o Cache) para que las actualizaciones aprobadas por la IA lleguen instantáneamente a todos los clientes.

Task 4.4: Conector SQL Server (Pro). * Desarrollar el generador de esquemas SQL dinámicos basado en el JSONL para la inyección directa de datos a bases de datos empresariales.

**Estado Hito 4: ⏳ PENDIENTE**
- Todas las tareas de inteligencia y escalabilidad están pendientes.

📊 Definición de Éxito (KPIs)

Reducción de Código: Eliminar el 90% de los match estáticos en el parser.

Rendimiento: Conversión de 100MB de EDIFACT a JSONL en menos de 5 segundos en el navegador.

Compresión: Mantener ratios de ahorro superiores al 95% usando el formato .fra.

Autonomía: El sistema debe ser capaz de auto-proponer traducciones para el 80% de las etiquetas nuevas encontradas.

requisitos Técnicos Pendientes

Instalar clang para compilación WASM

sudo apt-get install clang

Compilar módulo WASM:

cd wasm && cargo build --target wasm32-unknown-unknown --release

Ejecutar API REST:

cargo run --bin api --features api

cd frontend && npm run dev