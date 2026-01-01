# Archive Carving (TAR, GZIP, BZIP2, XZ)

Status: Planned

## Problem Statement

The golden image contains compressed archives (GZIP, BZIP2, XZ) and TAR archives that are not currently carved. These formats are ubiquitous on Unix/Linux systems and commonly encountered in forensic investigations.

## Scope

- Add signature patterns and config entries for GZIP, BZIP2, XZ, and TAR.
- Implement carve handlers with best-effort size detection.
- Wire handlers into the carve registry.
- Add unit tests.
- Update documentation.

## Non-Goals

- Decompression or extraction of contents.
- Validation of compressed data integrity.
- Nested archive handling (e.g., tar.gz as two separate files).
- Recovery of truncated/corrupted compressed streams.

---

## File Format Details

### GZIP (.gz)

**Signature:**
- `1F 8B` - GZIP magic number
- Byte 2: Compression method (08 = deflate)
- Byte 3: Flags (FTEXT, FHCRC, FEXTRA, FNAME, FCOMMENT)

**Size Detection Strategy:**
1. GZIP has **no reliable size in header**.
2. Options:
   - **Footer-based:** Last 8 bytes of GZIP contain CRC32 + uncompressed size (LE u32).
   - **Streaming:** Decompress/scan deflate stream until BFINAL block.
   - **Heuristic:** Look for next valid file header or max_size.

3. Recommended approach for carving:
   - Scan for GZIP footer signature pattern is not reliable.
   - Use **deflate stream parsing**: track block boundaries until final block.
   - Alternative: Use max_size limit and validate by attempting decompress.

**Complexity:** High - deflate stream parsing or heuristic required.

**Config Entry:**
```yaml
- id: "gzip"
  extensions: ["gz"]
  header_patterns:
    - id: "gzip_header"
      hex: "1F8B08"          # Magic + deflate method
  max_size: 1073741824       # 1 GiB
  min_size: 18               # Minimal header + footer
  validator: "gzip"
```

### BZIP2 (.bz2)

**Signature:**
- `42 5A 68` ("BZh") + block size digit ('1'-'9')
- E.g., `42 5A 68 39` = "BZh9" (900KB block size)

**Size Detection Strategy:**
1. BZIP2 streams consist of blocks ending with specific magic.
2. Stream end marker: `17 72 45 38 50 90` (48 bits, bit-aligned!)
3. Problem: Bit-alignment makes reliable detection difficult.
4. Options:
   - **Full stream parsing:** Parse bzip2 blocks until end marker.
   - **Heuristic:** Use max_size, validate with decompressor.
   - **Footer search:** Look for end marker (may have false positives).

**Complexity:** High - bit-aligned markers make this challenging.

**Config Entry:**
```yaml
- id: "bzip2"
  extensions: ["bz2"]
  header_patterns:
    - id: "bzip2_header"
      hex: "425A68"          # "BZh"
  max_size: 1073741824       # 1 GiB
  min_size: 14               # Minimal stream
  validator: "bzip2"
```

### XZ (.xz)

**Signature:**
- `FD 37 7A 58 5A 00` - XZ magic (6 bytes)

**Size Detection Strategy:**
1. XZ has a **stream footer** at the end:
   - Last 2 bytes: `59 5A` ("YZ") - footer magic
   - Bytes -12 to -3: CRC32, backward size, stream flags
2. XZ stream structure:
   - Header (12 bytes): magic + flags + CRC
   - Blocks with compressed data
   - Index
   - Footer (12 bytes): CRC + backward size + flags + magic

3. Approach:
   - Cannot easily find footer without scanning.
   - **Parse forward:** Read blocks, each block has header with sizes.
   - Block header: first byte indicates header size, contains compressed/uncompressed sizes.
   - After all blocks, read Index and Footer.

**Complexity:** Medium - block structure is parseable.

**Config Entry:**
```yaml
- id: "xz"
  extensions: ["xz"]
  header_patterns:
    - id: "xz_header"
      hex: "FD377A585A00"    # XZ magic
  max_size: 1073741824       # 1 GiB
  min_size: 32               # Header + minimal block + footer
  validator: "xz"
```

### TAR (.tar)

**Signature:**
- TAR has **no magic number at offset 0**.
- "ustar" format: `75 73 74 61 72` ("ustar") at offset 257.
- GNU tar: `75 73 74 61 72 20 20 00` at offset 257.
- POSIX: `75 73 74 61 72 00 30 30` at offset 257.
- Old V7 tar: No magic, just header structure.

**Header Structure:**
- 512-byte records
- First 100 bytes: filename
- Bytes 124-135: file size in octal ASCII
- Bytes 148-155: checksum
- Byte 156: type flag
- Bytes 257-262: "ustar" magic (if present)

**Size Detection Strategy:**
1. TAR is a sequence of 512-byte headers + file data (padded to 512).
2. Walk headers:
   - Read filename (if all nulls, might be end).
   - Read size field (octal ASCII), convert to bytes.
   - Skip size bytes (rounded up to 512).
   - Continue until two consecutive null blocks (1024 bytes of zeros).
3. Validate checksums to reduce false positives.

**Complexity:** Medium - ASCII parsing, checksum validation.

**Config Entry:**
```yaml
- id: "tar"
  extensions: ["tar"]
  header_patterns:
    - id: "tar_ustar"
      hex: "7573746172"      # "ustar" at offset 257
      offset: 257
    - id: "tar_gnu"
      hex: "757374617220200" # "ustar  \0" at offset 257
      offset: 257
  max_size: 10737418240      # 10 GiB
  min_size: 512              # Single header block
  validator: "tar"
```

**Note:** Offset-based patterns require scanner support for non-zero offsets.

---

## Implementation Plan

### Phase 1: XZ Handler (Most Structured)

1. Create `src/carve/xz.rs`:
   - Validate XZ header magic.
   - Parse stream header (12 bytes).
   - Walk blocks: read block header size, compressed size, skip data.
   - Read Index section (variable length).
   - Validate footer magic.
2. Add config entry.
3. Add unit tests.

### Phase 2: TAR Handler

1. Create `src/carve/tar.rs`:
   - Check for "ustar" magic at offset 257.
   - Walk 512-byte records.
   - Parse size field (octal ASCII to u64).
   - Validate header checksum.
   - Stop at two null blocks or invalid header.
2. Handle edge cases: long filenames (extended headers), sparse files.
3. Add config entry.
4. Add unit tests.

### Phase 3: GZIP Handler

1. Create `src/carve/gzip.rs`:
   - Validate GZIP header.
   - Parse optional header fields (FEXTRA, FNAME, FCOMMENT, FHCRC).
   - Two approaches:
     - **A (Simple):** Use max_size, rely on validation.
     - **B (Better):** Implement deflate block boundary detection.
2. For approach B: detect BFINAL bit in deflate blocks.
3. Add config entry.
4. Add unit tests.

### Phase 4: BZIP2 Handler

1. Create `src/carve/bzip2.rs`:
   - Validate "BZh" header.
   - Options:
     - **Simple:** Max size limit.
     - **Complex:** Parse bzip2 block structure (bit-aligned).
2. Consider using max_size approach initially.
3. Add config entry.
4. Add unit tests.

---

## Implementation Considerations

### Compressed Archive Challenges

1. **No embedded size:** Unlike containers, compressed streams don't declare their size upfront.
2. **Bit-alignment:** BZIP2 uses bit-level markers, making byte-scanning unreliable.
3. **Validation tradeoffs:** Full decompression for validation is expensive.

### Recommended Approach

1. **Start simple:** Use max_size limits for compressed formats.
2. **Add heuristics:** Look for next valid file header as end marker.
3. **Optional validation:** Attempt header-only decompression to verify format.
4. **Flag truncated:** If hitting max_size, mark file as potentially truncated.

### Offset-Based Patterns (TAR)

TAR requires matching at offset 257, which may need scanner enhancement:

**Option A:** Add offset support to pattern matching.
**Option B:** Use a generic first-512-byte scan, let validator handle TAR detection.
**Option C:** Scan for "ustar" string, then verify full TAR structure at offset-257.

Recommendation: Option C - scan for "ustar", validate backwards.

---

## Expected Tests

- `xz_basic`: Carve XZ file, verify block walking.
- `tar_ustar`: Carve POSIX tar, verify header walking.
- `tar_multifile`: Carve tar with multiple entries.
- `gzip_basic`: Carve GZIP file with max_size approach.
- `bzip2_basic`: Carve BZIP2 file with max_size approach.

---

## Impact on Docs/README

- Update file type lists in README.md.
- Add entries to docs/config.md for new validators.
- Document limitations of compressed stream carving.
- Note that .tar.gz files will be carved as GZIP (not extracted).

---

## Open Questions

1. **Deflate parsing complexity:**
   - Is it worth implementing deflate block parsing for GZIP?
   - Recommendation: Start with max_size, add parsing later if needed.

2. **TAR offset patterns:**
   - Does the scanner support offset-based patterns?
   - If not, use string scan for "ustar" and validate from there.

3. **Compound archives (tar.gz, tar.xz):**
   - Should we detect and specially handle these?
   - Recommendation: No. Carve outer format, let user decompress.

4. **Compression validation:**
   - Should we decompress first N bytes to validate?
   - Recommendation: Optional, off by default (expensive).
