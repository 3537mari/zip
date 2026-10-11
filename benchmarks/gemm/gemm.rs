use std::time::Instant;

fn gemm(a: &[f64], b: &[f64], c: &mut [f64], m: usize, k: usize, n: usize) {
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for l in 0..k {
                sum += a[i * k + l] * b[l * n + j];
            }
            c[i * n + j] = sum;
        }
    }
}

fn main() {
    let size = 512;
    let a = vec![1.0; size * size];
    let b = vec![2.0; size * size];
    let mut c = vec![0.0; size * size];

    let start = Instant::now();
    gemm(&a, &b, &mut c, size, size, size);
    let duration = start.elapsed();

    println!("{}", c[0]);
    println!("{:.2}", duration.as_secs_f64() * 1000.0);
}
