# When a Comma Is Not a Separator

The Rust code for the 37:29 video about building a streaming CSV reader. You can open every source file directly in this repository.

## Start here

- [Complete parser and record iterator — rust/lib.rs](rust/lib.rs)
- [Run the finished reader — rust/import.rs](rust/import.rs)
- [Boundary and streaming tests — tests/streaming.rs](tests/streaming.rs)
- [Exact CSV rules and error behavior — DIALECT.md](DIALECT.md)
- [Full transcript with timestamps and links to the code](TRANSCRIPT.md)

## Follow the seven stages

| Stage | File | What this version shows |
| --- | --- | --- |
| 1. One field | [01_field.rs](checkpoints/01_field.rs) | Accumulate bytes and form one UTF-8 string. |
| 2. Plain records | [02_records.rs](checkpoints/02_records.rs) | Commas, LF endings, empty fields and final EOF. |
| 3. Quoted contents | [03_quoted.rs](checkpoints/03_quoted.rs) | Keep commas and line breaks inside quotes. This intermediate version deliberately does not yet handle escaped or malformed quotes. |
| 4. Look ahead | [04_lookahead.rs](checkpoints/04_lookahead.rs) | Handle doubled quotes and malformed continuations when all input is available. LF endings only at this stage. |
| 5. Keep state across chunks | [05_decoder.rs](checkpoints/05_decoder.rs) | Use the persistent decoder even when input stops in the middle of a quote pair. |
| 6. Request a record | [06_reader.rs](checkpoints/06_reader.rs) | Read complete records through the blocking I/O adapter. |
| 7. Apply application rules | [07_application.rs](checkpoints/07_application.rs) | Check headers, field counts, numbers and domain rules after parsing. |

Stages 1–4 contain their own implementations. Stages 5–7 use the completed implementation in [rust/lib.rs](rust/lib.rs), including CRLF handling, UTF-8 validation and errors. Keep the `checkpoints/` and `rust/` directories together when running those examples. Every checkpoint includes a main program and tests.

## Run the finished example

Install a Rust toolchain that supports edition 2024. These files were checked with Rust 1.91.1. From the repository root:

```sh
cargo run --example import
cargo test
```

The parser itself uses only the Rust standard library. The development dependency `serde_json` is used by [rust/trace.rs](rust/trace.rs), which exports the execution events used for the animation. You can inspect those events with `cargo run --example trace`.

## Run one checkpoint

For example, compile stage 4:

```sh
rustc --edition=2024 checkpoints/04_lookahead.rs -o checkpoint.exe
./checkpoint.exe
```

To run its tests:

```sh
rustc --edition=2024 --test checkpoints/04_lookahead.rs -o checkpoint-test.exe
./checkpoint-test.exe
```

With Python installed, `python check_checkpoints.py` runs the main programs and tests for all seven stages.

## Subtitles and chapters

- [English subtitles with timing — captions.srt](captions.srt)
- [WebVTT subtitles — captions.vtt](captions.vtt)
- [Chapter timestamps — chapters.txt](chapters.txt)

The subtitle timestamps match the 37:29 film. The transcript links to the exact source files used for each displayed excerpt.
