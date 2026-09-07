// Checkpoint 06: request complete records through the blocking Read adapter.
#[path = "../rust/lib.rs"]
#[allow(dead_code)]
mod csv;

fn main() -> Result<(), csv::ReadError> {
    let input = b"Mira,\"bread, milk\"\r\nNoah,tea\r\n";
    for result in csv::Records::new(&input[..]) {
        let record = result?;
        println!("{record:?}");
    }
    Ok(())
}

#[test]
fn two_requests_then_end() {
    let mut reader = csv::Records::new(&b"Mira,tea\nNoah,milk"[..]);
    assert_eq!(reader.next().unwrap().unwrap(), vec!["Mira", "tea"]);
    assert_eq!(reader.next().unwrap().unwrap(), vec!["Noah", "milk"]);
    assert!(reader.next().is_none());
    assert!(reader.next().is_none());
}
