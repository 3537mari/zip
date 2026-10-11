use serde_json::Value;
use std::time::Instant;

fn main() {
    let mut items = Vec::with_capacity(10000);
    for i in 0..10000 {
        items.push(serde_json::json!({"id": i}));
    }
    let data = serde_json::json!({"данные": items});
    let json_str = serde_json::to_string(&data).unwrap();

    let start = Instant::now();
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    let duration = start.elapsed();
    let _ = parsed["данные"].as_array().map(|a| a.len());
    println!("{:.2}", duration.as_secs_f64() * 1000.0);
}
