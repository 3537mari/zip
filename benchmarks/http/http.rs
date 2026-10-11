use std::hint::black_box;
use std::time::Instant;

#[inline(never)]
fn parse_get(req: &str) -> i32 {
    if !req.starts_with("GET ") {
        return -1;
    }
    let bytes = req.as_bytes();
    let mut p = 4usize;
    if bytes.get(p) != Some(&b'/') {
        return -1;
    }
    while p < bytes.len() && bytes[p] != b' ' {
        p += 1;
    }
    (p - 4) as i32
}

fn main() {
    let req = "GET /api/v1/item/42 HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let n = 5_000_000;
    let start = Instant::now();
    let mut s = 0i64;
    for _ in 0..n {
        s += black_box(parse_get(black_box(req))) as i64;
    }
    black_box(s);
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("{}", s);
    println!("{:.2}", ms);
}
