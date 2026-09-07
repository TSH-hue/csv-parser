# The exact CSV rules implemented in this film

This is a small, specified CSV reader inspired by the quoting rules in RFC 4180, not a claim to recognize every CSV dialect. RFC 4180 is an informational memo. The library does not guess a dialect.

- Input is UTF-8, optionally quoted at the **start of a field**. A quote in a bare field is an error.
- Commas separate fields only outside quotes. Inside quotes, commas, CR, and LF are contents and are preserved exactly.
- Two consecutive quotes **inside a quoted field** represent one content quote. After a closing quote, only comma, LF, CRLF, or final EOF is legal. Spaces there are not silently removed.
- Outside quotes, LF and CRLF end records. A bare CR is rejected. CRLF is one boundary, even when split between reads.
- Spaces are data. No trimming, BOM removal, comment syntax, backslash escapes, or automatic number conversion.
- Empty input produces zero records. A blank line produces one record containing one empty field. A trailing comma creates an empty last field. A final record separator does not create a phantom record.
- The final record may lack a line ending. Final EOF inside an open quote or between CR and LF is an error. Running out of bytes in a **chunk** is not final EOF.
- Field counts may differ. Every record is delivered as `Vec<String>`. Whether the first record is a header, how many fields are expected, and which strings are valid numbers are application decisions.
- UTF-8 is checked when a field is completed, after its raw content bytes have accumulated. A chunk may split a character. ASCII syntax bytes cannot be UTF-8 continuation bytes. Invalid UTF-8 errors point to the **start of the source field**, including its opening quote; other syntax errors point to the offending source byte or final EOF. Offsets are zero-based bytes, not character indexes.
- On a parser or I/O error, the iterator emits that error once and stops. Previously yielded records stay valid. The current invalid record is not yielded. Interrupted reads are retried; other read errors are delivered as errors. This is a blocking adapter; it treats a zero-byte read as final EOF.
- Streaming storage consists of a fixed input buffer plus the current field and record. A very large single record still needs substantial memory. A caller can choose to collect all records, but the reader itself does not.


References: [RFC 4180](https://www.rfc-editor.org/rfc/rfc4180.html), [Read](https://doc.rust-lang.org/std/io/trait.Read.html), [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html).
