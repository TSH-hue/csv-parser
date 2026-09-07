use csv_film::{CsvError, Decoder, ErrorKind, ReadError, Records};
use std::io::{self, Read};

fn chunks(
    input: &[u8],
    cuts: &[usize],
) -> (Vec<Vec<String>>, Option<CsvError>) {
    let mut decoder = Decoder::new();
    let mut rows = Vec::new();
    let mut start = 0;
    for end in
        cuts.iter().copied().chain(std::iter::once(input.len()))
    {
        for &byte in &input[start..end] {
            match decoder.push(byte) {
                Ok(Some(row)) => rows.push(row),
                Ok(None) => {}
                Err(e) => return (rows, Some(e)),
            }
        }
        // Deliberately do nothing here: a chunk end is not final EOF.
        start = end;
    }
    match decoder.finish() {
        Ok(Some(row)) => rows.push(row),
        Ok(None) => {}
        Err(e) => return (rows, Some(e)),
    }
    (rows, None)
}

fn assert_case(input: &str, expected: &[&[&str]]) {
    let expected: Vec<Vec<String>> = expected
        .iter()
        .map(|row| row.iter().map(|x| x.to_string()).collect())
        .collect();
    let bytes = input.as_bytes();
    for cut in 0..=bytes.len() {
        assert_eq!(
            chunks(bytes, &[cut]),
            (expected.clone(), None),
            "cut {cut}, {input:?}"
        );
    }
    let every_byte: Vec<_> = (0..=bytes.len()).collect();
    assert_eq!(chunks(bytes, &every_byte), (expected.clone(), None));
    let actual =
        Records::new(bytes).collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn empty_fields_and_record_endings_are_distinct() {
    assert_case("", &[]);
    assert_case("\n", &[&[""]]);
    assert_case("\r\n", &[&[""]]);
    assert_case(",", &[&["", ""]]);
    assert_case(",\n", &[&["", ""]]);
    assert_case("Mira,", &[&["Mira", ""]]);
    assert_case("Mira\n", &[&["Mira"]]);
    assert_case("Mira\n\n", &[&["Mira"], &[""]]);
    assert_case(
        "Mira,tea\r\nNoah,milk",
        &[&["Mira", "tea"], &["Noah", "milk"]],
    );
    assert_case("a,b\nc", &[&["a", "b"], &["c"]]);
}

#[test]
fn quoted_contents_and_unicode_survive_every_cut() {
    assert_case("Mira,\"bread, milk\"", &[&["Mira", "bread, milk"]]);
    assert_case("\"\"", &[&[""]]);
    assert_case("\"\"\"\"", &[&["\""]]);
    assert_case(
        "Noah,\"say \"\"hi\"\"\"\r\n",
        &[&["Noah", "say \"hi\""]],
    );
    assert_case("\"top\nbottom\"", &[&["top\nbottom"]]);
    assert_case("\"top\r\nbottom\"", &[&["top\r\nbottom"]]);
    assert_case("\"a\rb\"", &[&["a\rb"]]);
    assert_case(" Zoë ,café 🦀", &[&[" Zoë ", "café 🦀"]]);
    assert_case(
        "A,\"\"\"café\"\",\nnext\",\r\nB,,done",
        &[&["A", "\"café\",\nnext", ""], &["B", "", "done"]],
    );
}

#[test]
fn malformed_input_has_stable_error_kind_and_position() {
    for (input, kind, at) in [
        (&b"ab\"c"[..], ErrorKind::QuoteInBareField, 2),
        (&b"\"x\"q"[..], ErrorKind::CharacterAfterQuote, 3),
        (&b"\"x\" "[..], ErrorKind::CharacterAfterQuote, 3),
        (&b"\"x"[..], ErrorKind::UnclosedQuote, 2),
        (&b"\"\"\""[..], ErrorKind::UnclosedQuote, 3),
        (&b"a\rb"[..], ErrorKind::ExpectedLf, 2),
        (&b"a\r"[..], ErrorKind::ExpectedLf, 2),
        (&b"x,\xff"[..], ErrorKind::InvalidUtf8, 2),
        (&b"x,\"\"\"\xff\""[..], ErrorKind::InvalidUtf8, 2),
        (&b"\xc3"[..], ErrorKind::InvalidUtf8, 0),
    ] {
        let expected = chunks(input, &[]);
        assert_eq!(expected.1, Some(CsvError { kind, offset: at }));
        for cut in 0..=input.len() {
            assert_eq!(chunks(input, &[cut]), expected);
        }
        assert_eq!(
            chunks(input, &(0..=input.len()).collect::<Vec<_>>()),
            expected
        );
    }
}

#[test]
fn completed_rows_survive_a_later_error() {
    let mut records = Records::new(&b"good,row\n\"bad\"x\n"[..]);
    assert_eq!(records.next().unwrap().unwrap(), ["good", "row"]);
    assert!(matches!(
        records.next(),
        Some(Err(ReadError::Csv(CsvError {
            kind: ErrorKind::CharacterAfterQuote,
            offset: 14
        })))
    ));
    assert!(records.next().is_none());
    assert!(records.next().is_none());
}

#[test]
fn decoder_cannot_resume_after_final_eof_or_error() {
    let mut d = Decoder::new();
    assert_eq!(d.finish(), Ok(None));
    assert_eq!(d.push(b'x').unwrap_err().kind, ErrorKind::Closed);
    assert_eq!(d.finish().unwrap_err().kind, ErrorKind::Closed);
    let mut d = Decoder::new();
    d.push(b'x').unwrap();
    d.push(b'"').unwrap_err();
    assert_eq!(d.push(b'y').unwrap_err().kind, ErrorKind::Closed);
}

struct Tiny<R> {
    input: R,
    limit: usize,
    interrupt: bool,
}
impl<R: Read> Read for Tiny<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = self.limit.min(out.len());
        self.input.read(&mut out[..n])
    }
}

#[test]
fn reader_handles_short_reads_interruptions_and_split_unicode() {
    let text = "Mira,\"a\"\"b\"\r\nZoë,café\n";
    for limit in 1..=text.len() {
        let reader = Tiny {
            input: text.as_bytes(),
            limit,
            interrupt: true,
        };
        let result = Records::new(reader)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            result,
            vec![vec!["Mira", "a\"b"], vec!["Zoë", "café"]]
        );
    }
}

struct Broken {
    first: bool,
}
impl Read for Broken {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.first {
            self.first = false;
            out[..2].copy_from_slice(b"a\n");
            Ok(2)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "disk failed"))
        }
    }
}

#[test]
fn io_error_is_emitted_once_without_losing_prior_record() {
    let mut records = Records::new(Broken { first: true });
    assert_eq!(records.next().unwrap().unwrap(), ["a"]);
    assert!(matches!(records.next(), Some(Err(ReadError::Io(_)))));
    assert!(records.next().is_none());
}

#[test]
fn next_retains_unread_bytes_and_does_not_prefetch_another_read() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct Count {
        calls: Rc<Cell<usize>>,
        input: &'static [u8],
    }
    impl Read for Count {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            self.calls.set(self.calls.get() + 1);
            self.input.read(out)
        }
    }
    let calls = Rc::new(Cell::new(0));
    let mut records = Records::new(Count {
        calls: calls.clone(),
        input: b"a\nb\n",
    });
    assert_eq!(records.next().unwrap().unwrap(), ["a"]);
    assert_eq!(calls.get(), 1);
    assert_eq!(records.next().unwrap().unwrap(), ["b"]);
    assert_eq!(calls.get(), 1);
    assert!(records.next().is_none());
    assert_eq!(calls.get(), 2);
}

#[test]
fn large_field_exceeds_the_input_buffer() {
    let field = "é,".repeat(5000);
    let input = format!("\"{field}\",end\r\n");
    let actual = Records::new(input.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(actual, vec![vec![field, "end".to_string()]]);
}

#[test]
fn exhaustive_short_byte_strings_do_not_depend_on_chunk_boundaries() {
    let alphabet = [b'a', b',', b'"', b'\r', b'\n'];
    for len in 0u32..=6 {
        for mut index in 0..5usize.pow(len) {
            let mut input = Vec::new();
            for _ in 0..len {
                input.push(alphabet[index % 5]);
                index /= 5;
            }
            let expected = chunks(&input, &[]);
            for cut in 0..=input.len() {
                assert_eq!(chunks(&input, &[cut]), expected);
            }
            assert_eq!(
                chunks(
                    &input,
                    &(0..=input.len()).collect::<Vec<_>>()
                ),
                expected
            );
        }
    }
}
