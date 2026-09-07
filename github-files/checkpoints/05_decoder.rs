// Checkpoint 05: retain the Decoder while arbitrary chunks arrive.
// The implementation is linked here from the same production source used in the film.
#[path = "../rust/lib.rs"]
#[allow(dead_code)]
mod csv;

fn main() -> Result<(), csv::CsvError> {
    let chunks: &[&[u8]] =
        &[b"Noah,\"say \"", b"\"hi\"\"\"\r", b"\n"];
    let mut decoder = csv::Decoder::new();
    for chunk in chunks {
        for &byte in *chunk {
            if let Some(record) = decoder.push(byte)? {
                println!("{record:?}");
            }
        }
        // No finish call here: another chunk may arrive.
    }
    if let Some(record) = decoder.finish()? {
        println!("{record:?}");
    }
    Ok(())
}

#[test]
fn split_quote_remains_unresolved_until_the_next_byte() {
    let mut decoder = csv::Decoder::new();
    for &byte in b"\"a\"" {
        decoder.push(byte).unwrap();
    }
    assert_eq!(decoder.state(), csv::State::QuoteSeen);
    decoder.push(b'"').unwrap();
    assert_eq!(decoder.state(), csv::State::Quoted);
    assert_eq!(decoder.field_bytes(), b"a\"");
    decoder.push(b'"').unwrap();
    assert_eq!(decoder.finish().unwrap().unwrap(), vec!["a\""]);
}
