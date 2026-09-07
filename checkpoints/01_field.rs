// Checkpoint 01: one field. There are no separators in this checkpoint's input.
fn parse_field(
    input: &[u8],
) -> Result<String, std::string::FromUtf8Error> {
    let mut field = Vec::new();
    for &byte in input {
        field.push(byte);
    }
    String::from_utf8(field)
}

fn main() {
    assert_eq!(parse_field(b"Mira").unwrap(), "Mira");
    println!("{:?}", parse_field(b"Mira").unwrap());
}

#[test]
fn one_field() {
    assert_eq!(parse_field(b"Mira").unwrap(), "Mira");
    assert_eq!(parse_field(b"Noah").unwrap(), "Noah");
    assert_eq!(parse_field("café".as_bytes()).unwrap(), "café");
}
