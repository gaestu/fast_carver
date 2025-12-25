# CSV Metadata Schema (Phase 2)

CSV output is enabled with `--metadata-backend csv`.

## carved_files.csv

Columns:

- `run_id`
- `file_type`
- `path`
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

## string_artefacts.csv

Columns:

- `run_id`
- `artefact_kind`
- `content`
- `encoding`
- `global_start`
- `global_end`
- `tool_version`
- `config_hash`
- `evidence_path`
- `evidence_sha256`

## browser_history.csv

Columns:

- `run_id`
- `browser`
- `profile`
- `url`
- `title`
- `visit_time`
- `visit_source`
- `source_file`
- `tool_version`
- `config_hash`
- `evidence_path`
- `evidence_sha256`
