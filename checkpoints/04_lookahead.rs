// Checkpoint 04: the entire input is available. Peek resolves a doubled quote.
// LF record endings only at this stage. This parser is not incremental.
fn parse(bytes: &[u8]) -> Result<Vec<Vec<String>>, &'static str> {
    let mut input = bytes.iter().copied().peekable();
    let mut field = Vec::new();
    let mut row = Vec::new();
    let mut rows = Vec::new();
    let mut in_quotes = false;
    let mut closed_quote = false;
    let mut started = false;
    while let Some(byte) = input.next() {
        started = true;
        if in_quotes {
            if byte == b'"' {
                if input.peek() == Some(&b'"') {
                    input.next();
                    field.push(b'"');
                } else {
                    in_quotes = false;
                    closed_quote = true;
                }
            } else {
                field.push(byte);
            }
        } else if byte == b',' || byte == b'\n' {
            row.push(
                String::from_utf8(std::mem::take(&mut field))
                    .map_err(|_| "invalid UTF-8")?,
            );
            closed_quote = false;
            if byte == b'\n' {
                rows.push(std::mem::take(&mut row));
                started = false;
            }
        } else if closed_quote {
            return Err("character after closing quote");
        } else if byte == b'"' {
            if !field.is_empty() {
                return Err("quote in bare field");
            }
            in_quotes = true;
        } else {
            field.push(byte);
        }
    }
    if in_quotes {
        return Err("unclosed quote");
    }
    if started {
        row.push(
            String::from_utf8(field).map_err(|_| "invalid UTF-8")?,
        );
        rows.push(row);
    }
    Ok(rows)
}

fn main() {
    println!("{:?}", parse(b"Noah,\"say \"\"hi\"\"\"").unwrap());
}

#[test]
fn content_quotes_and_invalid_continuations() {
    assert_eq!(
        parse(b"Noah,\"say \"\"hi\"\"\"").unwrap(),
        vec![vec!["Noah", "say \"hi\""]]
    );
    assert_eq!(parse(b"\"\"\"\"").unwrap(), vec![vec!["\""]]);
    assert!(parse(b"\"tea\"x").is_err());
    assert!(parse(b"te\"a").is_err());
    assert!(parse(b"\"tea").is_err());
    assert!(parse(b" \"tea\"").is_err());
}
