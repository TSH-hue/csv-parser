// Checkpoint 02: unquoted fields, commas, and LF record endings.
// Quoting and CRLF are deliberately outside this checkpoint's supported cases.
fn parse(
    input: &[u8],
) -> Result<Vec<Vec<String>>, std::string::FromUtf8Error> {
    let mut field = Vec::new();
    let mut row = Vec::new();
    let mut rows = Vec::new();
    let mut started = false;
    for &byte in input {
        started = true;
        match byte {
            b',' => {
                let bytes = std::mem::take(&mut field);
                let text = String::from_utf8(bytes)?;
                row.push(text);
            }
            b'\n' => {
                row.push(String::from_utf8(std::mem::take(
                    &mut field,
                ))?);
                rows.push(std::mem::take(&mut row));
                started = false;
            }
            _ => field.push(byte),
        }
    }
    if started {
        row.push(String::from_utf8(field)?);
        rows.push(row);
    }
    Ok(rows)
}

fn main() {
    println!("{:?}", parse(b"Mira,tea\nNoah,milk").unwrap());
}

#[test]
fn plain_records_and_empty_fields() {
    assert_eq!(
        parse(b"Mira,tea\nNoah,milk").unwrap(),
        vec![vec!["Mira", "tea"], vec!["Noah", "milk"]]
    );
    assert_eq!(
        parse(b"Mira,\n,tea").unwrap(),
        vec![vec!["Mira", ""], vec!["", "tea"]]
    );
    assert!(parse(b"").unwrap().is_empty());
    assert_eq!(parse(b"\n").unwrap(), vec![vec![""]]);
    assert_eq!(parse(b"Mira\n").unwrap(), vec![vec!["Mira"]]);
}
