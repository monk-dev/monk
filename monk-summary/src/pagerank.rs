#![allow(non_snake_case)]

use ndarray::{Array, Array1, Array2};
use ndarray_rand::{rand_distr::Uniform, RandomExt};
use tracing::info;

pub fn pagerank(probabilities: &Array2<f32>, d: f32, tolerance: f32) -> Array1<f32> {
    info!(
        "running pagerank on probabilities: {} x {}, d={}, tolerance={}",
        probabilities.shape()[0],
        probabilities.shape()[1],
        d,
        tolerance
    );

    // probabilities.for_each(|e| {
    //     if e.is_nan() {
    //         println!("probabilitiy is nan!");
    //     }
    // });

    // The number of total links:
    let N = probabilities.shape()[0];

    // The probability of one link going to another:
    let M = probabilities;
    let M_hat: Array2<f32> = d * M + (1.0 - d) / N as f32;

    // x is our final pagerank vector, initialized with a random distribution
    // and then normalized to sum to 1.
    let mut x: Array1<f32> = Array::random(N, Uniform::new(0.0, 1.0));
    x /= x.sum();

    info!("{x:?}");

    let mut last_x: Array1<f32> = Array1::zeros(N);

    let mut iteration = 0;
    loop {
        x = M_hat.dot(&x);

        let diff = &x - &last_x;
        let l1_diff = diff.dot(&diff).sqrt();

        if l1_diff < tolerance {
            break;
        }

        // info!("iteration: {}, l1_diff={}", iteration, l1_diff);

        // break;
        last_x = x.clone();

        if iteration % 10 == 0 {
            info!("iteration: {}, l1_diff={}", iteration, l1_diff);
        }

        iteration += 1;
    }

    x
}
