# Parent-description representation

This research representation prefixes owned semantic passages with a literal root
excerpt. It uses the approved recipe in ../context-design/parent-description.md.

`export.rs` calls the existing public read-only search selection API, selects the
first nonempty root Description, Notes or Biography field and its first nonempty
section in public selection order, and takes the first text-splitter chunk with
capacity128 using the existing BGE tokenizer JSON. Public TextEmbeddingTokenizer
independently checks the resulting token count. No HTML parsing or prepared-header
stripping is used. Excerpt offsets are UTF-8 byte offsets within the complete
prepared source section, whose hash and length are recorded. They are not raw
HTML offsets. Omitted bytes describe that section; later sections/fields remain
available in unchanged independent root semantic units.

`keys.json` is the initial5253-root inventory. `complement-keys.json` covers every
remaining root with owned semantic units; both use the same role-aware exporter.
Freeze requires complete complement validation, rather than assuming the initial
field-path inventory was exhaustive.

`override-mapping.json.gz` maps each changed unit ID to its original and new text
hash and parent excerpt key. Units not in overrides reuse the original corpus.
`documents.jsonl.gz` contains unique new raw inputs. Each model input uses this
same document order; Gemma alone has its documented formatting prefix. Document
byteOffsets are planned float32 offsets by model dimension; actual inference
output receipts must verify them. No vectors or inference are produced here.

`validate.cjs` independently checks every override against original unit text and
all four model input streams against the unique document map. Token preflight
uses the existing model runtime in check-only mode with native limits. It rejects
rather than truncates overlong inputs.
