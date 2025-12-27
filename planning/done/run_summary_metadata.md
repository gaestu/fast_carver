Status: Implemented
Implemented in version: 0.1.0

# Run Summary Metadata Output

Short description: Write run-level summary metrics to metadata outputs (JSONL/CSV/Parquet).

## Problem statement
Run-level counters are only logged; they should be captured in structured metadata to support downstream reporting and auditing.

## Scope
- Add a run summary record with counts for bytes scanned, chunks processed, hits found, files carved, string spans, and artefacts extracted.
- Emit summary via JSONL/CSV/Parquet metadata sinks.
- Update docs/README to mention the new output.

## Non-goals
- Persisting start/end timestamps or wall-clock duration.
- Exposing per-file-type stats.

## Design notes
- Add a `RunSummary` struct and a new metadata sink method.
- Emit a single summary record at pipeline completion.

## Expected tests
- Metadata sink tests cover presence of the new run summary file.
- Parquet test confirms `run_summary.parquet` exists.

## Impact on docs and README
- Document the `run_summary` output in metadata docs and README.
