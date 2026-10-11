use std::time::Instant;

fn main() {
    const N: i32 = 1_000_000;
    let start = Instant::now();
    let mut h: u32 = 2166136261;
    let mut buf = String::new();
    for i in 0..N {
        let tmp = format!("{},", i % 1000);
        buf.push_str(&tmp);
        for b in tmp.bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        if buf.len() > 4096 {
            buf.clear();
        }
    }
    let duration = start.elapsed();
    println!("{}", h);
    println!("{:.2}", duration.as_secs_f64() * 1000.0);
}
