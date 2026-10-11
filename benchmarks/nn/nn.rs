use std::time::Instant;

struct Layer {
    n_in: usize,
    n_out: usize,
    act: u8, // 1 relu 2 sigmoid
    w: Vec<f64>,
    b: Vec<f64>,
    last_in: Vec<f64>,
    last_out: Vec<f64>,
}

fn xavier(rng: &mut u32, n_in: usize, n_out: usize) -> f64 {
    *rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
    let r = ((*rng % 2000000) as i32 - 1000000) as f64 / 1000000.0;
    let limit = (6.0 / (n_in + n_out) as f64).sqrt();
    r * limit
}

impl Layer {
    fn new(n_in: usize, n_out: usize, act: u8, rng: &mut u32) -> Self {
        let mut w = vec![0.0; n_in * n_out];
        for v in &mut w {
            *v = xavier(rng, n_in, n_out);
        }
        Self {
            n_in,
            n_out,
            act,
            w,
            b: vec![0.0; n_out],
            last_in: vec![0.0; n_in],
            last_out: vec![0.0; n_out],
        }
    }

    fn forward(&mut self, x: &[f64], y: &mut [f64]) {
        self.last_in.copy_from_slice(x);
        for j in 0..self.n_out {
            let mut z = self.b[j];
            for i in 0..self.n_in {
                z += x[i] * self.w[i * self.n_out + j];
            }
            y[j] = if self.act == 1 {
                if z > 0.0 { z } else { 0.0 }
            } else {
                1.0 / (1.0 + (-z).exp())
            };
            self.last_out[j] = y[j];
        }
    }

    fn backward(&mut self, gout: &[f64], gin: Option<&mut [f64]>, lr: f64) {
        let mut gz = vec![0.0; self.n_out];
        for j in 0..self.n_out {
            let y = self.last_out[j];
            let da = if self.act == 1 {
                if y > 0.0 { 1.0 } else { 0.0 }
            } else {
                y * (1.0 - y)
            };
            gz[j] = gout[j] * da;
        }
        if let Some(gin) = gin {
            for i in 0..self.n_in {
                let mut s = 0.0;
                for j in 0..self.n_out {
                    s += gz[j] * self.w[i * self.n_out + j];
                }
                gin[i] = s;
            }
        }
        for i in 0..self.n_in {
            for j in 0..self.n_out {
                self.w[i * self.n_out + j] -= lr * self.last_in[i] * gz[j];
            }
        }
        for j in 0..self.n_out {
            self.b[j] -= lr * gz[j];
        }
    }
}

fn main() {
    let mut rng = 12345u32;
    let mut l1 = Layer::new(2, 4, 1, &mut rng);
    let mut l2 = Layer::new(4, 1, 2, &mut rng);
    let x = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];
    let y_t = [0.0, 1.0, 1.0, 0.0];
    let lr = 0.5;
    let epochs = 10000;
    let start = Instant::now();
    for _ in 0..epochs {
        for k in 0..4 {
            let mut h = [0.0; 4];
            let mut y = [0.0; 1];
            l1.forward(&x[k], &mut h);
            l2.forward(&h, &mut y);
            let err = y[0] - y_t[k];
            let mut gin = [0.0; 4];
            l2.backward(&[err], Some(&mut gin), lr);
            l1.backward(&gin, None, lr);
        }
    }
    println!("{:.2}", start.elapsed().as_secs_f64() * 1000.0);
}
