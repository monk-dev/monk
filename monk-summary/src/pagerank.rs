#![allow(non_snake_case)]

use ndarray::{Array, Array1, Array2};
use ndarray_rand::{rand_distr::Uniform, RandomExt};

pub fn pagerank(probabilities: &Array2<f32>, d: f32, tolerance: f32) -> Array1<f32> {
    // The number of total links:
    let N = probabilities.shape()[0];

    // The probability of one link going to another:
    let M = probabilities;
    let M_hat: Array2<f32> = d * M + (1.0 - d) / N as f32;

    // x is our final pagerank vector, initialized with a random distribution
    // and then normalized to sum to 1.
    let mut x: Array1<f32> = Array::random(N, Uniform::new(0.0, 1.0));
    x /= x.sum();

    let mut last_x: Array1<f32> = Array1::zeros(N);

    loop {
        x = M_hat.dot(&x);

        let diff = &x - &last_x;
        let l1_diff = diff.dot(&diff).sqrt();

        if l1_diff < tolerance {
            break;
        }

        last_x = x.clone();
    }

    x
}
