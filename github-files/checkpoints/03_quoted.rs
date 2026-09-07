// Checkpoint 03: valid quoted fields without escaped content quotes.
// This working hypothesis does NOT validate malformed quotes or doubled quotes.
// The next checkpoint exposes and repairs those missing decisions.
fn parse(
    input: &[u8],
) -> Result<Vec<Vec<String>>, std::string::FromUtf8Error> {
    let mut field = Vec::new();
    let mut row = Vec::new();
    let mut rows = Vec::new();
    let mut in_quotes = false;
    let mut started = false;
    for &byte in input {
        started = true;
        if byte == b'"' {
            in_quotes = !in_quotes;
        } else if in_quotes {
            field.push(byte);
        } else if byte == b',' {
            row.push(String::from_utf8(std::mem::take(&mut field))?);
        } else if byte == b'\n' {
            row.push(String::from_utf8(std::mem::take(&mut field))?);
            rows.push(std::mem::take(&mut row));
            started = false;
        } else {
            field.push(byte);
        }
    }
    if started {
        row.push(String::from_utf8(field)?);
        rows.push(row);
    }
    Ok(rows)
}

fn main() {
    println!("{:?}", parse(b"Mira,\"bread, milk\"").unwrap());
}

#[test]
fn quoted_comma_and_line_break() {
    assert_eq!(
        parse(b"Mira,\"bread, milk\"").unwrap(),
        vec![vec!["Mira", "bread, milk"]]
    );
    assert_eq!(
        parse(b"Mira,\"top\nbottom\"\n").unwrap(),
        vec![vec!["Mira", "top\nbottom"]]
    );
}

#[test]
fn evidence_that_the_next_decision_is_needed() {
    // This is intentionally a limitation test, not a claim that the result is right.
    assert_ne!(
        parse(b"\"say \"\"hi\"\"\"").unwrap(),
        vec![vec!["say \"hi\""]]
    );
}
