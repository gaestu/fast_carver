# Miscellaneous Format Carving (ICO, EML, ELF, Ebook)

Status: Implemented
Implemented in version: 0.2.1

## Problem Statement

The golden image contains several additional formats that warrant carving support:
- **ICO** - Windows icon files
- **EML** - Email messages (RFC 822)
- **ELF/SO** - Linux executables and shared libraries
- **Ebook formats** - EPUB, AZW3, FB2, LRF

These are forensically relevant but have varying complexity and detection reliability.

## Scope

- Evaluate each format for carving feasibility.
- Implement handlers for formats with reliable detection.
- Document formats that are low-priority or problematic.
- Add unit tests for implemented formats.
- Update documentation.

## Non-Goals

- Content extraction from email or ebooks.
- Symbol table parsing for ELF files.
- Metadata extraction.

---

## File Format Details

### ICO (Windows Icon)

**Signature:**
- `00 00 01 00` - ICO header (reserved=0, type=1)
- `00 00 02 00` - CUR (cursor) header (type=2)

**Structure:**
- Bytes 0-1: Reserved (0)
- Bytes 2-3: Type (1=ICO, 2=CUR)
- Bytes 4-5: Number of images (LE u16)
- Bytes 6+: Image directory entries (16 bytes each)

**Directory Entry (16 bytes):**
- Byte 0: Width (0 = 256)
- Byte 1: Height (0 = 256)
- Byte 2: Color count
- Byte 3: Reserved
- Bytes 4-5: Color planes (ICO) or hotspot X (CUR)
- Bytes 6-7: Bits per pixel (ICO) or hotspot Y (CUR)
- Bytes 8-11: Image data size (LE u32)
- Bytes 12-15: Offset to image data (LE u32)

**Size Detection Strategy:**
1. Read number of images (bytes 4-5).
2. Parse directory entries.
3. Find max(offset + size) across all entries.

**Complexity:** Low - structured header.

**Config Entry:**
```yaml
- id: "ico"
  extensions: ["ico", "cur"]
  header_patterns:
    - id: "ico_header"
      hex: "00000100"        # ICO type=1
    - id: "cur_header"
      hex: "00000200"        # CUR type=2
  max_size: 10485760         # 10 MiB
  min_size: 22               # Header + 1 entry
  validator: "ico"
```

**Concern:** `00 00 01 00` and `00 00 02 00` are short patterns with high false positive risk.

### EML (Email Message)

**Signature:**
- No magic number; text-based RFC 822 format.
- Common patterns:
  - `From: ` or `From ` (mbox format)
  - `Received: ` 
  - `MIME-Version: 1.0`
  - `Content-Type: `
  - `Return-Path: `
  - `Date: `

**Structure:**
- Headers: `Name: Value` lines, separated by CRLF or LF.
- Blank line separates headers from body.
- Body may be MIME multipart with boundaries.

**Size Detection Strategy:**
1. **No reliable end marker** - plain text to EOF.
2. Options:
   - **Heuristic:** Look for next email header pattern.
   - **MIME boundary:** If multipart, end at final boundary `--boundary--`.
   - **Max size:** Use configured limit.

**Complexity:** High - text format, no structural end.

**Recommendation:** Low priority for carving. Better suited for string/pattern extraction than file carving.

**Config Entry (if implemented):**
```yaml
- id: "eml"
  extensions: ["eml"]
  header_patterns:
    - id: "eml_from"
      hex: "46726F6D3A20"     # "From: "
    - id: "eml_received"
      hex: "52656365697665643A" # "Received:"
  max_size: 52428800          # 50 MiB
  min_size: 32
  validator: "eml"
```

### ELF (Executable and Linkable Format)

**Signature:**
- `7F 45 4C 46` (`\x7FELF`) - ELF magic

**Structure:**
- Header (52 or 64 bytes depending on 32/64-bit).
- Program headers (for execution).
- Section headers (for linking).
- Section data.

**Header Fields (relevant):**
- Bytes 0-3: Magic
- Byte 4: Class (1=32-bit, 2=64-bit)
- Byte 5: Endianness (1=LE, 2=BE)
- Bytes 16-17 (32-bit) or 16-17 (64-bit): e_type (executable, shared object, etc.)
- Bytes 32-35 (32-bit) or 40-47 (64-bit): e_shoff (section header offset)
- Bytes 46-47 (32-bit) or 58-59 (64-bit): e_shnum (number of section headers)
- Bytes 44-45 (32-bit) or 56-57 (64-bit): e_shentsize (section header size)

**Size Detection Strategy:**
1. Read ELF header: class (32/64), endianness.
2. Read section header table offset (e_shoff).
3. Read section header count and size.
4. Calculate: `size = e_shoff + (e_shnum * e_shentsize)`
5. Alternatively: find highest section end (section offset + size).

**Complexity:** Medium - well-structured binary format.

**Config Entry:**
```yaml
- id: "elf"
  extensions: ["", "so", "elf"]
  header_patterns:
    - id: "elf_magic"
      hex: "7F454C46"        # "\x7FELF"
  max_size: 1073741824       # 1 GiB
  min_size: 52               # Minimal 32-bit header
  validator: "elf"
```

### EPUB (Electronic Publication)

**Current Status:** Detected as ZIP.

**Structure:**
- ZIP archive with:
  - `mimetype` file (first, uncompressed): `application/epub+zip`
  - `META-INF/container.xml`
  - Content files (XHTML, CSS, images)

**Enhancement:**
- Classify during ZIP carving based on `mimetype` content.
- Already handled if we implement ZIP document classification.

**Recommendation:** Handle via ZIP classification enhancement (see document_carving.md).

### AZW3 (Kindle Format 8)

**Signature:**
- Multiple possible structures:
  - MOBI-based: `BOOKMOBI` at offset 60 (inside PDB header)
  - Palm Database header at start

**PDB Header:**
- Bytes 0-31: Name (null-padded)
- Bytes 32-33: Attributes
- Bytes 34-35: Version
- Bytes 60-67: Type + Creator (`BOOKMOBI` for MOBI/AZW)

**Size Detection:**
- PDB files have record table in header.
- Read record count and offsets.
- Size = last record offset + last record size.

**Complexity:** Medium - PDB container format.

**Config Entry:**
```yaml
- id: "mobi"
  extensions: ["mobi", "azw", "azw3", "prc"]
  header_patterns:
    - id: "mobi_pdb"
      hex: "424F4F4B4D4F4249"  # "BOOKMOBI" at offset 60
      offset: 60
  max_size: 536870912         # 512 MiB
  min_size: 68
  validator: "mobi"
```

**Note:** Requires offset-based pattern matching.

### FB2 (FictionBook)

**Signature:**
- XML-based: `<?xml` followed by `<FictionBook`
- Pattern: `3C 3F 78 6D 6C` (`<?xml`)

**Size Detection:**
- XML document, ends with `</FictionBook>`
- Scan for closing tag.

**Complexity:** Medium - XML parsing or tag search.

**Config Entry:**
```yaml
- id: "fb2"
  extensions: ["fb2"]
  header_patterns:
    - id: "fb2_xml"
      hex: "3C3F786D6C"       # "<?xml"
  max_size: 104857600         # 100 MiB
  min_size: 64
  validator: "fb2"
```

**Note:** Many XML files start with `<?xml`. Need content validation.

### LRF (Sony BroadBand eBook)

**Signature:**
- `4C 52 46 00` ("LRF\0")

**Structure:**
- Proprietary Sony format.
- Header contains version and object offsets.

**Size Detection:**
- Header contains pointer to object table.
- Complex proprietary structure.

**Complexity:** High - proprietary, limited documentation.

**Recommendation:** Low priority unless specifically requested.

---

## Implementation Priority

### High Priority (Implement)

1. **ICO** - Simple structure, common format, reasonable signature.
2. **ELF** - Well-documented, forensically relevant, clear signature.

### Medium Priority (Consider)

3. **MOBI/AZW3** - PDB format is documented, common on Kindle devices.
4. **FB2** - XML-based, but high false positive risk from `<?xml`.

### Low Priority (Defer)

5. **EML** - Text-based, no reliable end marker, better for string extraction.
6. **LRF** - Proprietary, limited use.
7. **EPUB** - Handle via ZIP classification (already planned).

---

## Implementation Plan

### Phase 1: ICO Handler

1. Create `src/carve/ico.rs`:
   - Validate ICO/CUR signature.
   - Parse image count.
   - Walk directory entries.
   - Calculate size from max(offset + size).
   - Validate entry sanity (no overlaps, reasonable sizes).
2. Add config entry.
3. Add unit tests.

### Phase 2: ELF Handler

1. Create `src/carve/elf.rs`:
   - Validate ELF magic.
   - Parse header: class, endianness.
   - Read section header table location.
   - Calculate size from section headers.
   - Handle both 32-bit and 64-bit.
2. Add config entry.
3. Add unit tests with ELF executable and shared object.

### Phase 3: MOBI/PDB Handler (Optional)

1. Create `src/carve/mobi.rs`:
   - Parse PDB header.
   - Validate "BOOKMOBI" type/creator.
   - Read record table.
   - Calculate size from records.
2. Add config entry.
3. Add unit tests.

---

## False Positive Considerations

### ICO
- `00 00 01 00` is only 4 bytes and appears in many contexts.
- Mitigation: Validate directory entry structure (reasonable dimensions, offsets within file).

### ELF
- `\x7FELF` is distinctive, low false positive rate.
- Mitigation: Validate ELF class and endianness bytes.

### EML
- Text patterns like "From: " are very common in many contexts.
- Mitigation: If implemented, require multiple header patterns to match.

### FB2
- `<?xml` appears in many XML files.
- Mitigation: Search for `<FictionBook` after XML declaration.

---

## Expected Tests

- `ico_single`: Carve ICO with single image.
- `ico_multi`: Carve ICO with multiple images.
- `elf_executable`: Carve 64-bit ELF executable.
- `elf_shared_object`: Carve .so file.
- `elf_32bit`: Carve 32-bit ELF.

---

## Impact on Docs/README

- Update file type lists in README.md.
- Add entries to docs/config.md for new validators.
- Document false positive risks for certain formats.
- Note that EPUB is handled via ZIP classification.

---

## Open Questions

1. **ICO false positives:**
   - Is `00 00 01 00` too short?
   - Recommendation: Implement with strict validation, monitor false positive rate.

2. **ELF stripped binaries:**
   - Stripped binaries may have no section headers.
   - Use program headers (segment table) as fallback for size calculation.

3. **Email handling:**
   - Should EML be carved or should we focus on email header extraction?
   - Recommendation: Focus on string extraction for email artifacts, defer EML carving.

4. **PDB/MOBI offset patterns:**
   - "BOOKMOBI" is at offset 60, not start.
   - Requires scanner enhancement or string-based detection with validation.

5. **SVG files:**
   - XML-based, starts with `<?xml` or `<svg`.
   - High false positive risk, complex XML parsing for end.
   - Recommendation: Defer, low forensic priority.
