use std::time::Instant;

fn fibonacci(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    let n = 40;
    let start = Instant::now();
    let result = fibonacci(n);
    let duration = start.elapsed();
    println!("{}", result);
    println!("{:.2}", duration.as_secs_f64() * 1000.0);
}
