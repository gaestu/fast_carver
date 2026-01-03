# Carver Enhancements v2

**Status: Implemented**
**Implemented: 2025-01**

## Summary of Changes

- Enhanced ICO validator with BMP/PNG signature verification at declared offsets
- Enhanced EML validator requiring 2+ RFC 822 headers and email patterns  
- Enhanced FB2 validator requiring FictionBook marker in first 4KB
- Added `--dry-run` mode for scan-only (no file writes)
- Added `--validate-carved` and `--remove-invalid` CLI flags (CLI ready)
- Added periodic parquet flushing during progress reporting
- Enhanced progress reporting with completion percentage
- Added `--enable-types` as alternative to `--types` for explicit inclusion

## Problem Statement

Real-world forensic image analysis revealed significant false positive rates in certain carvers:
- **ICO**: 66% false positives (5,309 files, 46 GB wasted)
- **EML**: 97% false positives (template strings, debug messages misidentified)
- **FB2**: 100% false positives (generic XML files misidentified)

Additionally, usability issues were identified:
- Parquet files not flushed on cancellation
- No dry-run mode for preview
- Limited progress visibility
- No post-carving validation option
- Coarse file type control (only `--types` include list)

## Scope

### In Scope
1. Strengthen ICO validator (verify BMP/PNG signatures at offsets)
2. Strengthen EML validator (require multiple RFC 822 headers)
3. Fix FB2 validator (require FictionBook namespace, not just `<?xml`)
4. Add `--dry-run` mode (count-only, no file writes)
5. Add `--validate-carved` flag for post-carving validation
6. Fix parquet periodic flushing and graceful shutdown
7. Enhance progress reporting (completion %, validation stats)
8. Add `--enable-types` for fine-grained type control (vs `--types`)

### Out of Scope
- New file type carvers
- GPU-specific changes
- Major architectural changes

## Design

### 1. ICO Validator Enhancement

Current problem: 4-byte signature `00 00 01 00` is too short and common.

**Solution**:
- After parsing directory entries, validate that at least one entry contains valid BMP or PNG data
- BMP signature at offset: `42 4D` (BM)
- PNG signature at offset: `89 50 4E 47` (PNG header within ICO)
- Reject if no valid image signatures found at declared offsets
- Add stricter bounds checking (max 256 entries, reasonable sizes)

### 2. EML Validator Enhancement

Current problem: `From: ` pattern matches template strings.

**Solution**:
- Require at least 2 of: `From:`, `To:`, `Subject:`, `Date:`
- Validate `From:` line contains `@` character (basic email check)
- Look for `\r\n` line endings within first 1KB
- Reject obvious non-email patterns

### 3. FB2 Validator Enhancement

Current problem: `<?xml` matches any XML file.

**Solution**:
- Already checks for `<FictionBook` tag (good!)
- Issue: scanning for tag happens incrementally, may accept before finding it
- Add: require `FictionBook` or `fictionbook` (case-insensitive) in first 4KB
- Add: validate XML namespace contains "fictionbook" if xmlns present

### 4. Dry Run Mode

Add `--dry-run` CLI flag:
- Skip all file writes (carved files and metadata)
- Run full scan and carving logic
- Report counts at end
- Useful for estimating output size

### 5. Validate Carved Flag

Add `--validate-carved` CLI flag:
- After carving, run `file` magic validation or internal validation
- Flag invalid files in metadata (add `validation_status` field)
- Optionally remove invalid files with `--remove-invalid`

### 6. Parquet Flushing

Current problem: 0-byte files on cancellation.

**Solution**:
- Flush after every N records (configurable, default 1000)
- Register signal handler to flush on SIGINT/SIGTERM
- Add `flush()` method to MetadataSink trait
- Call flush in graceful shutdown path

### 7. Enhanced Progress Reporting

Add to progress snapshot:
- `validation_pass_count` / `validation_fail_count` (if `--validate-carved`)
- Completion percentage (already calculated)
- ETA in human-readable format

### 8. Granular Type Control

Current: `--types jpeg,png` (whitelist only)

Add: `--enable-types <types>` as alternative syntax
- Clearer naming
- Same functionality as `--types`
- Keep `--types` for backward compatibility

## Impact on Docs and README

- Update CLI documentation with new flags
- Document validation behavior
- Update metadata schema for validation fields

## Expected Tests

1. ICO validator: test with valid and invalid ICO files
2. EML validator: test with real emails vs template strings
3. FB2 validator: test with FB2 vs generic XML
4. Dry run: verify no files written
5. Parquet flush: verify data survives cancellation
6. Progress: verify new fields populated

## Implementation Tasks

1. [x] Update ICO carver validation
2. [x] Update EML carver validation  
3. [x] Update FB2 carver validation
4. [x] Add dry-run CLI flag and pipeline support
5. [x] Add validate-carved flag and validation pass (CLI added, validation logic deferred)
6. [x] Fix parquet flushing
7. [x] Enhance progress reporting
8. [x] Add enable-types alias
9. [x] Update tests
10. [x] Update documentation
