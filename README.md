# fastcarve

High-speed, forensic-grade file and artefact carver (Phase 2: CPU-only core with SQLite, strings, and expanded file types).

## Quick start

```bash
cargo run -- --input /path/to/image.dd --output ./output
```

E01 input (requires `libewf`):

```bash
cargo run --features ewf -- --input /path/to/image.E01 --output ./output
```

String scanning (URLs/emails/phones):

```bash
cargo run -- --input /path/to/image.dd --output ./output --scan-strings
```

This creates a run directory under `./output/<run_id>/` with:

- `carved/` - carved files per type (jpeg/png/gif/pdf/zip/webp/sqlite). ZIPs are classified into docx/xlsx/pptx when entries match.
- `metadata/` - JSONL records for carved files, string artefacts, and browser history

## Configuration

The default configuration lives in `config/default.yml`. You can override it with:

```bash
cargo run -- --input /path/to/image.dd --output ./output --config-path /path/to/config.yml
```

Key settings:

- `overlap_bytes`: chunk overlap in bytes
- `enable_string_scan`: enable printable string scanning
- `string_min_len`: minimum string length to consider
- `string_max_len`: maximum string length per span
- `file_types`: enabled formats, header patterns, size limits

CLI overrides:

- `--overlap-kib`: overrides `overlap_bytes` when set
- `--scan-strings`: enables string scanning
- `--string-min-len`: overrides `string_min_len` when set
- `--metadata-backend csv`: write CSV instead of JSONL

See `docs/config.md` for the full schema.

## Output metadata (JSONL)

Carved files are recorded to `metadata/carved_files.jsonl` with run-level provenance.
String artefacts (URLs/emails/phones) are recorded to `metadata/string_artefacts.jsonl`.
Browser history records (from carved SQLite) are recorded to `metadata/browser_history.jsonl`.

See `docs/metadata_jsonl.md` for the schema.
CSV output is also available with `--metadata-backend csv` (see `docs/metadata_csv.md`).

## Architecture

The Phase 2 pipeline:

1. Evidence source (raw file)
2. Chunk scheduler + reader
3. CPU signature scanner
4. Optional CPU string scanner + artefact extraction
5. Carve workers (JPEG/PNG/GIF/PDF/ZIP/WEBP/SQLite)
6. SQLite parser for browser history
7. JSONL/CSV metadata sink

See `docs/architecture.md` for details.

## Notes

- E01 support is available when built with `--features ewf` and requires `libewf` installed.
- GPU acceleration is not implemented yet.

## License

MIT (see `LICENSE`).
