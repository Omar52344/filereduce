resumen de lo que se quiere para filereduce:

revisar el manejador o procesador de elemento de filereduce para edifact ya que este espera siempre unas secciones edifact quemadas, seria ideal tener una opcion de que se puedan automaticamente
generar secciones nuevas del edifact sin necesidad de quemarlas en el codigo y que tambien el json de salida tambien sea dinamico, es decir tener un diccionario de traduccion de etiquetas edifact autoalimentable y dinamico (se podria crear un json de traducciones donde se defina "valueEdifact:"","valueJon:""") este json seria un estatico que se modifica a demanda por otro servicios

y podamos usar ese json como un libreria dinamica que se pueda alimentar de forma diferente sea manual o por otro servicio que traduzca con ia y identifique etiquetas nuevas previamente para poderlas traducir antes de hacer la conversion del archivo para con ellos automatizar el proceso de agregar etiquetas nuevas


el metodo de import export edifact se debe pensar para exponer como un metodo
que se va a volver wasm para poderlo usar en la web, comtemplando esta librearia JSOn de etiquetas de traduccion


desarrollar un front drag and drop que consuma el wasm o el enpoint rest o que arquitectura manejar para consumir estos servicios de compresion seria la mejor para poder usar ese dinamismo, 



🗺️ Roadmap de FileReduce: Del Motor al SaaS
🚀 Hito 1: El Motor Dinámico (Rust Core Refactor)
Objetivo: Eliminar el código "quemado" y permitir que la lógica dependa de metadatos externos.

Task 1.1: Definición del Schema de Traducción. Diseñar el translations.json que soporte segmentos, sub-elementos y nombres legibles (Labels).

Task 1.2: Registro de Mapeo (Registry Pattern). Crear un HashMap global en Rust que cargue el JSON y sirva como fuente de verdad para el parser.

Task 1.3: Refactorización del Parser EDIFACT. Reemplazar los match estáticos por una función de búsqueda (lookup) que consulte el Registro.

Task 1.4: Manejador de "Unknown Tags". Implementar un sistema de eventos que capture segmentos no encontrados y los guarde en un log para posterior análisis.

Task 1.5: Unit Testing Dinámico. Crear tests donde se pase un archivo EDIFACT y un JSON de mapeo diferente para asegurar que el output cambie sin tocar el binario.

🌐 Hito 2: Portabilidad y WebAssembly (WASM)
Objetivo: Llevar la potencia de Rust al navegador para el Demo Web.

Task 2.1: Integración de wasm-bindgen. Exponer los métodos de conversión (process_edifact) y compresión (filereducelib) para JavaScript.

Task 2.2: Optimización de Binario WASM. Configurar wasm-opt para reducir el tamaño del paquete y asegurar una carga rápida en la web.

Task 2.3: Bridge de Datos. Implementar la transferencia eficiente de grandes archivos (Uint8Array) entre JS y Rust evitando copias innecesarias de memoria.

Task 2.4: Wrapper de Diccionario en Web. Crear la lógica para que el WASM pueda recibir el JSON de traducciones actualizado desde una URL externa.

🎨 Hito 3: Frontend de Impacto (Next.js + Drag & Drop)
Objetivo: Crear el "Efecto Wow" para el cliente con valor inmediato visible.

Task 3.1: Interface "Dropzone". Desarrollar el área de carga de archivos con feedback visual de progreso.

Task 3.2: Web Worker Orchestration. Configurar un Worker de JS para que el WASM procese archivos pesados en un hilo separado (sin congelar la UI).

Task 3.3: Data Grid Legible (TanStack Table). Renderizar el JSON resultante en una tabla con nombres humanos (ej: "Número de Orden" en lugar de "BGM+102").

Task 3.4: Widget de Ahorro .fra. Implementar un comparador visual que muestre la reducción de tamaño entre el original y el comprimido.

Task 3.5: Descarga Local. Habilitar la descarga del archivo .jsonl y el backup .fra generado localmente.

🧠 Hito 4: Inteligencia y Escalabilidad (The Cloud Brain)
Objetivo: Automatizar el crecimiento del diccionario y la persistencia.

Task 4.1: API de Reporte de Etiquetas. Crear un endpoint (FastAPI o Rust Axum) que reciba las etiquetas desconocidas enviadas por los clientes.

Task 4.2: Integración con IA (Gemini API). Desarrollar el worker que toma etiquetas desconocidas, las analiza con el manual de EDIFACT y sugiere la traducción al administrador.

Task 4.3: Global Dictionary Sync. Implementar un sistema de caché (Redis o similar) para que todos los clientes reciban las nuevas traducciones en tiempo real.

Task 4.4: Conector SQL Server (Pro). Desarrollar el módulo que toma el output del WASM y genera los scripts de inserción optimizados para la base de datos del cliente.


