# JSONL Metadata Schema (Phase 1)

Each line in `metadata/carved_files.jsonl` is a JSON object with:

- `run_id`
- `file_type`
- `path` (relative to `carved/`)
- `extension`
- `global_start`
- `global_end`
- `size`
- `md5`
- `sha256`
- `validated`
- `truncated`
- `errors`
- `pattern_id`
- `tool_version`
- `config_hash`
- `evidence_path`
- `evidence_sha256`

Example:

```json
{
  "run_id": "20250101T120000Z_00000001",
  "file_type": "jpeg",
  "path": "jpeg/jpeg_000000000400.jpg",
  "extension": "jpg",
  "global_start": 1024,
  "global_end": 1055,
  "size": 32,
  "md5": "...",
  "sha256": "...",
  "validated": true,
  "truncated": false,
  "errors": [],
  "pattern_id": "jpeg_soi",
  "tool_version": "0.1.0",
  "config_hash": "...",
  "evidence_path": "/cases/image.dd",
  "evidence_sha256": ""
}
```
