use csv_film::Records;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replacing this byte slice with File::open("shopping.csv")? changes only I/O.
    let source = b"name,note\r\nMira,\"bread, milk\"\r\nNoah,\"say \"\"hi\"\"\"\r\n";
    for result in Records::new(&source[..]) {
        let record = result?;
        println!("{record:?}");
    }
    Ok(())
}
