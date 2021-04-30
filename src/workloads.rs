use crate::Allocator;
use rand::distributions::Bernoulli;
use rand::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Results {
    pub malloc_fails: usize,
    pub free_fails: usize,
    pub internal_frag: usize,
    pub external_frag: f32,
}

pub fn stack<T: Allocator>(mut allocator: T, ratio: f64) -> Results {
    let size = 32;
    let mut rng = rand::thread_rng();
    let mut results = Results::default();

    let samples: Vec<bool> = Bernoulli::new(ratio)
        .unwrap()
        .sample_iter(&mut rng)
        .take(990)
        .collect();
    let mut ptrs = Vec::new();

    for _ in 0..10 {
        match allocator.malloc(size) {
            Some(ptr) => ptrs.push(ptr),
            None => {
                results.malloc_fails += 1;
            }
        }
    }

    for sample in samples.into_iter() {
        if sample {
            match allocator.malloc(size) {
                Some(ptr) => ptrs.push(ptr),
                None => {
                    results.malloc_fails += 1;
                }
            }
        } else {
            if ptrs.is_empty() {
                continue;
            }
            if allocator.free(ptrs.pop().unwrap()).is_err() {
                results.free_fails += 1;
            }
        }
    }

    results.internal_frag = allocator.internal_frag();
    results.external_frag = allocator.external_frag();

    results
}

pub fn random_memory<T: Allocator>(mut allocator: T, ratio: f64) -> Results {
    let mut rng = rand::thread_rng();
    let mut results = Results::default();

    let samples: Vec<bool> = Bernoulli::new(ratio)
        .unwrap()
        .sample_iter(&mut rng)
        .take(990)
        .collect();
    let mut ptrs = Vec::new();

    for _ in 0..10 {
        match allocator.malloc(rng.gen_range(32..=128)) {
            Some(ptr) => ptrs.push(ptr),
            None => {
                results.malloc_fails += 1;
            }
        }
    }

    for sample in samples.into_iter() {
        if sample {
            let size = rng.gen_range(32..=128);
            match allocator.malloc(size) {
                Some(ptr) => ptrs.push(ptr),
                None => {
                    results.malloc_fails += 1;
                }
            }
        } else {
            if ptrs.is_empty() {
                continue;
            }
            let i = rng.gen_range(0..ptrs.len());
            if allocator.free(ptrs.remove(i)).is_err() {
                results.free_fails += 1;
            }
        }
    }

    results.internal_frag = allocator.internal_frag();
    results.external_frag = allocator.external_frag();

    results
}
