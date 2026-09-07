use csv_film::Decoder;
use serde_json::{Value, json};

fn trace(id: &str, source: &str, cuts: &[usize]) -> Value {
    let mut parser = Decoder::new();
    let mut events = Vec::new();
    let mut rows = Vec::new();
    for (index, &byte) in source.as_bytes().iter().enumerate() {
        if cuts.contains(&index) {
            events.push(json!({"event":"chunk_end", "at":index,"state":format!("{:?}",parser.state()), "field":parser.field_bytes(),"row":parser.completed_fields()}));
        }
        let before = format!("{:?}", parser.state());
        let field_before = parser.field_bytes().to_vec();
        let row_before = parser.completed_fields().to_vec();
        let result = parser.push(byte);
        let mut event = json!({"event":"byte","at":index,"byte":byte,"before":before,"after":format!("{:?}",parser.state()),"field_before":field_before,"field":parser.field_bytes(),"row_before":row_before,"row":parser.completed_fields(),"awaiting_lf":parser.awaiting_lf()});
        match result {
            Ok(Some(row)) => {
                event["delivered"] = json!(row);
                rows.push(row);
            }
            Ok(None) => {}
            Err(error) => {
                event["error"] = json!({"kind":format!("{:?}",error.kind),"at":error.offset});
                events.push(event);
                return json!({"id":id,"source":source,"cuts":cuts,"events":events,"rows":rows});
            }
        }
        events.push(event);
    }
    let final_event = match parser.finish() {
        Ok(Some(row)) => {
            rows.push(row.clone());
            json!({"event":"eof","at":source.len(),"delivered":row})
        }
        Ok(None) => json!({"event":"eof","at":source.len()}),
        Err(error) => {
            json!({"event":"eof","at":source.len(),"error":{"kind":format!("{:?}",error.kind),"at":error.offset}})
        }
    };
    events.push(final_event);
    json!({"id":id,"source":source,"cuts":cuts,"events":events,"rows":rows})
}

fn main() {
    if let Some(file) = std::env::args().nth(1) {
        let scenes: Vec<Value> = serde_json::from_str(
            &std::fs::read_to_string(file).unwrap(),
        )
        .unwrap();
        let traces: Vec<_> = scenes
            .iter()
            .map(|s| {
                trace(
                    s["id"].as_str().unwrap(),
                    s["source"].as_str().unwrap(),
                    &[],
                )
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&traces).unwrap()
        );
        return;
    }
    let cases = vec![
        trace("opening", "Mira,\"bread, milk\"", &[]),
        trace("plain", "Mira,tea\nNoah,milk", &[]),
        trace("empty", "Mira,\n,tea", &[]),
        trace("newline", "Mira,\"top\nbottom\"\n", &[]),
        trace("doubled", "Noah,\"say \"\"hi\"\"\"\n", &[]),
        trace(
            "split_quote",
            "Noah,\"say \"\"hi\"\"\"\r\n",
            &[11, 18],
        ),
        trace("split_utf8", "Zoë,café\n", &[3, 9]),
        trace("bad_quote", "Mira,\"tea\"x", &[]),
        trace("unclosed", "Mira,\"tea", &[7]),
        trace(
            "combined",
            "A,\"\"\"café\"\",\nnext\",\r\nB,,done",
            &[4, 8, 20],
        ),
    ];
    println!("{}", serde_json::to_string_pretty(&cases).unwrap());
}
