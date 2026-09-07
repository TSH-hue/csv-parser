# CSV Parser in Rust

A streaming CSV reader that turns UTF-8 input into records of strings. It handles quoted commas, doubled quotes, embedded line breaks, and input arriving in separate chunks.

## The problem

A comma can separate two fields or belong inside a quoted field:

```csv
Mira,"bread, milk"
Noah,"say ""hi"""
```

The parser produces:

```text
["Mira", "bread, milk"]
["Noah", "say \"hi\""]
```

The surrounding quotes disappear. The comma inside the first note stays, and each doubled quote becomes one quote in the second note.

## Run the example

Use a Rust toolchain that supports edition 2024. Open a terminal in the directory containing `Cargo.toml` and run:

```sh
cargo run --example import
```

Output:

```text
["name", "note"]
["Mira", "bread, milk"]
["Noah", "say \"hi\""]
```

The example includes a header row. The parser returns it as an ordinary record; the application decides whether to treat it as a header.

Run the parser tests with:

```sh
cargo test
```

## Find the code

| File | Purpose |
| --- | --- |
| [rust/lib.rs](rust/lib.rs) | Complete parser, parser errors, and record iterator. |
| [rust/import.rs](rust/import.rs) | Small executable that reads and prints records. |
| [tests/streaming.rs](tests/streaming.rs) | Tests for quoting, empty fields, UTF-8, chunk boundaries, and I/O errors. |
| [DIALECT.md](DIALECT.md) | Exact accepted format and error rules. |
| [rust/trace.rs](rust/trace.rs) | Optional executable that prints parser state changes as JSON. Run with `cargo run --example trace`. |

The parser uses only the Rust standard library. The trace executable uses the development dependency `serde_json`.

## How the pieces fit

- **`Decoder` understands CSV syntax.** Call `push(byte)` for each input byte. It returns a record when one is complete. Keep the same decoder between chunks, and call `finish()` once at the actual end of input.
- **`Records<R>` handles reading.** It takes a blocking input source implementing `Read`, feeds bytes to the decoder, and yields one record at a time.
- **The application interprets the values.** Header names, expected field counts, number conversion, and domain rules belong in the calling code. [Stage 7](checkpoints/07_application.rs) demonstrates this separation.

## Build up the parser

Each numbered file contains a runnable example and tests. The early stages support a smaller set of inputs; use [rust/lib.rs](rust/lib.rs) for the complete implementation.

| Stage | Source | Added behavior |
| --- | --- | --- |
| 1 | [01_field.rs](checkpoints/01_field.rs) | Collect bytes into one UTF-8 field. No separators yet. |
| 2 | [02_records.rs](checkpoints/02_records.rs) | Separate plain fields and records; handle empty fields and final EOF. LF endings only. |
| 3 | [03_quoted.rs](checkpoints/03_quoted.rs) | Preserve commas and line breaks inside quotes. Intentionally incomplete: doubled quotes and malformed quoting are not handled yet. |
| 4 | [04_lookahead.rs](checkpoints/04_lookahead.rs) | Resolve doubled quotes by looking ahead, and reject malformed quoting. Requires the entire input; LF endings only. |
| 5 | [05_decoder.rs](checkpoints/05_decoder.rs) | Keep the complete decoder alive as separate input chunks arrive. |
| 6 | [06_reader.rs](checkpoints/06_reader.rs) | Read complete records through the blocking input adapter. |
| 7 | [07_application.rs](checkpoints/07_application.rs) | Validate headers, field counts, numeric values, and application rules. |

Stages 1-4 contain independent parsers. Stages 5-7 import `rust/lib.rs`, so keep the `rust/` and `checkpoints/` folders beside each other.

To run stage 4:

```sh
rustc --edition=2024 checkpoints/04_lookahead.rs -o checkpoint.exe
./checkpoint.exe
```

To run its tests:

```sh
rustc --edition=2024 --test checkpoints/04_lookahead.rs -o checkpoint-test.exe
./checkpoint-test.exe
```

With Python installed, `python check_checkpoints.py` runs the examples and tests for all seven stages.

## Format boundaries

The complete parser accepts LF and CRLF record endings, preserves spaces and quoted line breaks, and reports malformed quotes or invalid UTF-8. Empty input produces no records. A trailing comma creates an empty final field.

It does not guess a CSV dialect, strip a BOM, trim values, or convert strings into numbers. See [DIALECT.md](DIALECT.md) for all rules.
