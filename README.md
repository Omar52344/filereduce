# FileReduce

Procesador de archivos de gran volumen (EDIFACT, XML, JSONL) optimizado para alto rendimiento y conversión a estructuras de datos modernas. Diseñado para integración directa con bases de datos SQL Server y flujos de trabajo de datos masivos.

## 🚀 Instalación

Para instalar la herramienta globalmente en tu sistema desde el código fuente:

```bash
# Navega al directorio del proyecto
cd filereduce

# Instala el binario en tu PATH
cargo install --path .
```

*Asegúrate de tener Rust instalado y el directorio de binarios de cargo en tu PATH.*

---

## 🛠️ Uso y Comandos

### 1. Ingesta a SQL Server (`insert`)

Carga archivos EDIFACT directamente a tu base de datos utilizando un procedimiento almacenado configurable. Soporta cargas masivas (batching) y transacciones.

**Configuración (`config.yaml`)**
```yaml
ingest:
  connection_string: "server=tcp:myserver.database.windows.net,1433;database=myDB;user=user;password=pass;encrypt=true;trustServerCertificate=true;"
  procedure_name: "sp_EDI_Ingresar_Batch_Orders"
  json_param: "@JsonBatch"
  batch_size: 1000
```

#### Windows (PowerShell)
```powershell
filereduce insert --config config.yaml input.edifact
```

#### Linux / macOS (Bash)
```bash
filereduce insert --config config.yaml input.edifact
```

---

### 2. Procesamiento a Archivo (`process`)

Convierte archivos EDIFACT a formato JSONL (JSON Lines) para análisis local o ingestión en otros sistemas (BigQuery, etc.).

#### Windows (PowerShell)
```powershell
# Proceso simple
filereduce process input.edifact output.jsonl -f edifact

# Con filtro de consulta (SQL-like)
filereduce process input.edifact output_filtered.jsonl -f edifact -q "doc_type = 'ORDERS' AND qty > 100"
```

#### Linux / macOS (Bash)
```bash
# Proceso simple
filereduce process input.edifact output.jsonl -f edifact

# Con filtro de consulta (SQL-like)
filereduce process input.edifact output_filtered.jsonl -f edifact -q "doc_type = 'ORDERS' AND qty > 100"
```

---

### 3. Conversión de Formatos (`convert`)

Utilidad rápida para transformar entre formatos soportados.

#### Windows (PowerShell)
```powershell
filereduce convert input.xml output.json --from xml --to json
```

#### Linux / macOS (Bash)
```bash
filereduce convert input.xml output.json --from xml --to json
```

---

## 🏗️ Desarrollo

Para ejecutar pruebas de integración y verificar el funcionamiento localmente:

**Windows / Linux / macOS**
```bash
cargo test
```

```
filereduce/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── cli.rs            # Argumentos de línea de comandos
│   ├── processor.rs       # Procesador multi-formato
│   ├── error.rs          # Manejo de errores
│   ├── parser/           # Parsers específicos
│   │   ├── edifact.rs  # Parser EDIFACT
│   │   ├── xml.rs       # Parser XML
│   │   └── json.rs      # Parser JSON
│   └── model/           # Modelos de datos
└── engine_filereduce/
    ├── src/
    │   ├── query/        # Motor de consultas
    │   │   ├── ast.rs         # AST de consultas
    │   │   ├── parser.rs     # Parser SQL
    │   │   ├── aggregation.rs # Agregaciones
    │   │   └── lexer.rs      # Lexer de consultas
    │   ├── executor/      # Ejecutor de consultas
    │   ├── row.rs         # Modelo de fila
    │   └── reader/        # Lectura de datos
    ├── tests/            # Tests integrales y benchmarks
    └── benches/          # Benchmarks de rendimiento
```

## Examples

### EDIFACT a JSONL

```bash
cargo run -- process tests/fixtures/sample.edifact output.jsonl
```

Salida:
```json
{"doc_type":"","number":"ORDER001","buyer":"BUYER001","seller":"SELLER001","lines":[{"sku":"SKU001","qty":10.0,"amount":100.0},{"sku":"SKU002","qty":20.0,"amount":200.0},{"sku":"SKU003","qty":15.0,"amount":150.0}]}
```

### XML a JSONL

```bash
cargo run -- process tests/fixtures/sample.xml output.jsonl
```

### Consultas complejas

```sql
-- Filtrar por cantidad y ordenar
qty > 10 ORDER BY qty DESC

-- Búsqueda con patrón
sku LIKE 'SKU%'

-- Rango numérico
qty BETWEEN 1 AND 100

-- Múltiples condiciones
qty > 50 AND (sku = 'SKU001' OR sku = 'SKU002')
```

## Benchmarks

Ejecutar benchmarks para medir rendimiento:

```bash
# Benchmarks de procesamiento
cargo bench --bench processing_bench

# Benchmarks del motor de consultas
cargo bench --bench query_bench
```

## Optimizaciones

- **Streaming puro**: Sin acumulación de datos en memoria
- **BufRead/BufWriter**: Buffered I/O para eficiencia
- **Regex caching**: LIKE expressions cacheadas
- **Lazy evaluation**: Procesamiento a demanda
- **Zero-copy**: Cuando sea posible, evitando copias innecesarias

## Tests

```bash
# Ejecutar todos los tests
cargo test

# Tests integrales
cargo test --test integration_tests

# Tests del motor de consultas
cargo test --package engine_filereduce
```

## Rendimiento

Procesamiento típico:
- **Archivos pequeños** (<1MB): <100ms
- **Archivos medianos** (1-10MB): <1s
- **Archivos grandes** (10-100MB): <10s
- **Archivos masivos** (>100MB): ~1MB/s

## Roadmap

- [ ] Parser SQL completo con JOIN
- [ ] Agregaciones en CLI
- [ ] Paralelización de procesamiento
- [ ] Compresión de salida
- [ ] Soporte para CSV
- [ ] Soporte para otros formatos EDI
- [ ] Streaming HTTP
- [ ] Filtros dinámicos en tiempo real

## Contribuir

1. Fork del repositorio
2. Crear branch para feature
3. Hacer commits con mensajes claros
4. Pull request con descripción detallada

## License

MIT OR Apache-2.0

## Autor

Omar Fernando Jaramillo
