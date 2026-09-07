// Checkpoint 07: the application owns headers, field counts, and domain rules.
#[path = "../rust/lib.rs"]
#[allow(dead_code)]
mod csv;

#[derive(Debug, PartialEq)]
struct Order {
    name: String,
    quantity: i32,
}

fn orders(
    input: &[u8],
) -> Result<Vec<Order>, Box<dyn std::error::Error>> {
    let mut records = csv::Records::new(input);
    let header = records.next().ok_or("missing header")??;
    if header != ["name", "quantity"] {
        return Err("unexpected header".into());
    }
    let mut orders = Vec::new();
    for result in records {
        let record = result?;
        if record.len() != 2 {
            return Err("expected two fields".into());
        }
        let quantity: i32 = record[1].parse()?;
        if quantity < 0 {
            return Err("quantity must not be negative".into());
        }
        orders.push(Order {
            name: record[0].clone(),
            quantity,
        });
    }
    Ok(orders)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", orders(b"name,quantity\nMira,12")?);
    Ok(())
}

#[test]
fn application_rules_are_separate_from_csv_syntax() {
    assert_eq!(
        orders(b"name,quantity\nMira,12").unwrap(),
        vec![Order {
            name: "Mira".into(),
            quantity: 12
        }]
    );
    assert!(orders(b"name,quantity\nMira,-3").is_err());
    assert!(orders(b"name,quantity\nMira,many").is_err());
    assert!(orders(b"name,quantity\nMira").is_err());
}
