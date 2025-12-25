# Configuration

The default config is `config/default.yml`.

## Top-level fields

- `run_id` (string): optional; if empty, a timestamp-based ID is generated.
- `overlap_bytes` (u64): overlap between chunks.
- `enable_string_scan` (bool): not used in Phase 1.
- `file_types` (list): enabled file types and patterns.

## File type configuration

Each entry in `file_types` contains:

- `id`: identifier (e.g. `jpeg`, `png`, `gif`)
- `extensions`: list of output extensions
- `header_patterns`: signature patterns used by the scanner
- `footer_patterns`: reserved for future use
- `max_size`: maximum carve size in bytes
- `min_size`: minimum carve size in bytes
- `validator`: logical handler name

## Example

```yaml
run_id: ""
overlap_bytes: 65536
enable_string_scan: false
file_types:
  - id: "jpeg"
    extensions: ["jpg", "jpeg"]
    header_patterns:
      - id: "jpeg_soi"
        hex: "FFD8FF"
    footer_patterns: []
    max_size: 104857600
    min_size: 16
    validator: "jpeg"
```
