# filereduce

Motor de procesamiento de datos de alto rendimiento para archivos EDIFACT, XML y JSON con capacidad de consultas SQL-like.

## Características

- **Streaming de memoria constante**: Procesamiento de archivos de cualquier tamaño sin cargar todo en memoria
- **Multi-formato**: Soporte para EDIFACT, XML y JSON/JSONL
- **Motor de consultas SQL-like**:
  - SELECT con proyección de campos
  - WHERE con operadores: =, >, <, >=, <=, LIKE, IN, BETWEEN
  - Lógicos: AND, OR, NOT
  - ORDER BY con ASC/DESC
  - LIMIT para control de resultados
  - **Agregaciones**: COUNT, SUM, AVG, MIN, MAX
- **Salida**: JSONL (JSON Lines) optimizado
- **CLI completa**: Interface de línea de comandos con clap

## Instalación

```bash
cargo install --path .
```

## Uso

### Procesar archivos

```bash
# Procesar archivo EDIFACT
filereduce process input.edifact output.jsonl

# Procesar archivo XML
filereduce process input.xml output.jsonl

# Procesar archivo JSONL
filereduce process input.jsonl output.jsonl

# Especificar formato
filereduce process data.txt output.jsonl --format xml
```

### Consultas SQL

```bash
# Consulta simple
filereduce query input.jsonl "qty > 100"

# Consulta con LIKE
filereduce query input.jsonl "sku LIKE 'SKU%'" --output filtered.jsonl

# Consulta con múltiples condiciones
filereduce query input.jsonl "qty > 50 AND price < 1000" --output results.jsonl

# Consulta con agregaciones (futuro)
filereduce query input.jsonl "SELECT COUNT(*), SUM(qty), AVG(price) FROM *"
```

## Formatos soportados

### EDIFACT
- Segmentos: UNH, BGM, DTM, NAD, LIN, QTY, MOA, UNT, UNZ
- Conversión automática a JSON estructurado

### XML
- Elementos: `<record>`, `<item>`, `<row>`
- Streaming con eventos XML

### JSON/JSONL
- JSON Lines (un objeto JSON por línea)
- JSON arrays (convertidos a líneas)
- Normalización automática (elimina campos null)

## Estructura de proyecto

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
