# Golden Image Sample Files

This document tracks sample files for the golden test image.

**Source:** Most files from [file-examples.com](https://file-examples.com) (free for testing)

**Scope:** fastcarve currently carves jpeg/png/gif/pdf/zip/webp/sqlite/bmp/tiff/mp4/rar/7z; docx/xlsx/pptx are classified from ZIP content. Other formats below are optional/future.

---

## Pictures / Images

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| jpeg/jpg | ✅ Have | file_example_JPG_100kB.jpg | 100 KB | |
| jpeg/jpg | ✅ Have | file_example_JPG_500kB.jpg | 500 KB | (pick smaller) |
| png | ✅ Have | file_example_PNG_500kB.png | 500 KB | |
| png | ✅ Have | file_example_PNG_1MB.png | 1 MB | (pick smaller) |
| gif | ✅ Have | file_example_GIF_500kB.gif | 500 KB | |
| gif | ✅ Have | 20251230_*.gif | ? | AI-generated |
| bmp | ✅ Have | test.bmp | small | Generated |
| webp | ✅ Have | file_example_WEBP_250kB.webp | 250 KB | |
| tiff/tif | ✅ Have | file_example_TIFF_1MB.tiff | 1 MB | Large, consider smaller |
| ico | ✅ Have | file_example_favicon.ico | small | |
| svg | ✅ Have | file_example_SVG_30kB.svg | 30 KB | Text-based, no carver |
| heic | ❌ Missing | - | - | Need to source |
| raw (cr2/nef/arw/dng) | ❌ Missing | - | - | Low priority |

---

## Audio / Music

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| mp3 | ✅ Have | file_example_MP3_1MG.mp3 | 1 MB | |
| wav | ✅ Have | file_example_WAV_1MG.wav | 1 MB | |
| ogg | ✅ Have | file_example_OOG_1MG.ogg | 1 MB | |
| aac | ❌ Missing | - | - | Generate with ffmpeg |
| flac | ❌ Missing | - | - | Generate with ffmpeg |
| m4a | ❌ Missing | - | - | Generate with ffmpeg |

---

## Video

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| mp4 | ✅ Have | file_example_MP4_640_3MG.mp4 | 3 MB | Large! |
| avi | ✅ Have | file_example_AVI_480_750kB.avi | 750 KB | |
| mov | ✅ Have | file_example_MOV_640_800kB.mov | 800 KB | |
| webm | ✅ Have | file_example_WEBM_640_1_4MB.webm | 1.4 MB | |
| ogg (video) | ✅ Have | file_example_OGG_640_2_7mg.ogg | 2.7 MB | Large! |
| wmv | ✅ Have | file_example_WMV_640_1_6MB.wmv | 1.6 MB | |
| mkv | ❌ Missing | - | - | Generate with ffmpeg |
| flv | ❌ Missing | - | - | Low priority (legacy) |

---

## Documents (Office & Text)

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| pdf | ✅ Have | file-sample_150kB.pdf | 150 KB | |
| doc | ✅ Have | file-sample_100kB.doc | 100 KB | OLE format |
| docx | ✅ Have | file-sample_100kB.docx | 100 KB | ZIP-based |
| xls | ✅ Have | file_example_XLS_50.xls | 50 KB | OLE format |
| xlsx | ✅ Have | file_example_XLSX_100.xlsx | 100 KB | ZIP-based |
| ppt | ✅ Have | file_example_PPT_250kB.ppt | 250 KB | OLE format |
| pptx | ✅ Have | test.pptx | small | Generated |
| odt | ✅ Have | file-sample_100kB.odt | 100 KB | ZIP-based |
| ods | ✅ Have | file_example_ODS_10.ods | 10 KB | ZIP-based |
| odp | ✅ Have | file_example_ODP_200kB.odp | 200 KB | ZIP-based |
| rtf | ✅ Have | file-sample_100kB.rtf | 100 KB | Text-based |
| txt | ✅ Have | documents/source.txt | small | Basic text sample |
| csv | ✅ Have | file_example_CSV_5000.csv | ~5 KB | |
| json | ✅ Have | file_example_JSON_1kb.json | 1 KB | |
| xml | ✅ Have | file_example_XML_24kb.xml.xml | 24 KB | |
| yaml | ❌ Missing | - | - | Create manually |
| html | ✅ Have | Title.html | small | |

---

## Archives / Containers

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| zip | ✅ Have | zip_2MB.zip | 2 MB | Large! |
| rar | ✅ Have | test.rar | small | RAR4? add RAR5 sample |
| rar (v5) | ❌ Missing | jpegs.rar5 | ? | From bulk_extractor tests/Data |
| 7z | ✅ Have | test.7z | small | |
| tar | ✅ Have | test.tar | small | |
| tar.gz | ✅ Have | test.tar.gz | small | |
| tar.bz2 | ✅ Have | test.tar.bz2 | small | |
| tar.xz | ✅ Have | test.tar.xz | small | |
| gz | ✅ Have | test.txt.gz | small | |
| bz2 | ✅ Have | test.txt.bz2 | small | |
| xz | ✅ Have | test.txt.xz | small | |
| iso | ❌ Missing | - | - | Low priority |

---

## Databases / Structured Data

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| sqlite | ✅ Have | test.sqlite | small | |
| mdb/accdb | ❌ Missing | - | - | Low priority |

---

## eBooks (Bonus - you have these!)

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| epub | ✅ Have | sample1.epub | ? | ZIP-based |
| azw3 | ✅ Have | sample1.azw3 | ? | Kindle format |
| fb2 | ✅ Have | sample1.fb2 | ? | XML-based |
| lrf | ✅ Have | sample1.lrf | ? | Sony Reader |

---

## Executables / Binaries

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| exe | ❌ Missing | - | - | Get minimal PE |
| dll | ❌ Missing | - | - | Get minimal DLL |
| elf | ❌ Missing | - | - | Compile hello world |
| so | ❌ Missing | - | - | Compile minimal lib |
| apk | ❌ Missing | - | - | Low priority |

---

## Email / Messaging

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| eml | ❌ Missing | - | - | Create manually |
| msg | ❌ Missing | - | - | Hard to create |
| pst | ❌ Missing | - | - | Hard to create |
| mbox | ❌ Missing | - | - | Create manually |

---

## Browser Artefacts

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| Chrome History | ✅ Have | History | small | SQLite |
| Chrome Cookies | ✅ Have | Cookies | small | SQLite |
| Firefox places.sqlite | ✅ Have | places.sqlite | small | |

---

## Other / Strings

| Format | Status | File | Size | Notes |
|--------|--------|------|------|-------|
| strings.txt | ✅ Have | other/strings.txt | small | URLs/emails/phones |
| utf16 text | ❌ Missing | utf16-examples.txt | ? | From bulk_extractor tests/Data |

---

## Deprioritized / Not Needed For fastcarve Core

- Audio formats (mp3/wav/ogg/aac/flac/m4a) are not carved by fastcarve.
- Video formats except mp4 are not carved by fastcarve.
- OLE/OpenDocument formats (doc/xls/ppt/odt/ods/odp/rtf) are not carved; docx/xlsx/pptx come from ZIP.
- tar/gz/bz2/xz/iso containers are not carved (keep only if you want extra variety).
- Executables, email containers, and PSD/VCF/PCAP-style artefacts are not parsed by fastcarve.

---

## Summary

### Currently Supported by fastcarve (12 types + ZIP-classified office types)

| Type | Have Sample? | Action Needed |
|------|--------------|---------------|
| jpeg | ✅ Yes | Use 100KB version |
| png | ✅ Yes | Use 500KB version |
| gif | ✅ Yes | OK |
| bmp | ✅ Yes | OK (test.bmp) |
| webp | ✅ Yes | OK |
| tiff | ✅ Yes | OK (but 1MB is large) |
| pdf | ✅ Yes | OK |
| docx | ✅ Yes | OK |
| xlsx | ✅ Yes | OK |
| pptx | ✅ Yes | OK (test.pptx) |
| zip | ✅ Yes | OK (but 2MB is large) |
| rar | ✅ Yes | OK (test.rar) |
| 7z | ✅ Yes | OK (test.7z) |
| mp4 | ✅ Yes | OK (but 3MB is large) |
| sqlite | ✅ Yes | OK (test.sqlite) |

### Files to Add (High Priority)

- `jpegs.rar5` (RAR5 coverage for the rar handler; bulk_extractor `tests/Data`)
- `utf16-examples.txt` (UTF-16 string scan coverage; bulk_extractor `tests/Data`)

## Licensing

All files from file-examples.com are free for testing purposes.
Files from filesamples.com are sample files for testing.
Generated files (ImageMagick, ffmpeg, sqlite3) are CC0/public domain.
