use fasthash::{murmur3::Hash128_x64, FastHash};
use bit_vec::BitVec;
use std::iter::successors;
use std::f64::consts::{E, LN_2};

struct BloomFilter {
    m_m: u64,
    m_k: u64,
    m_vec: BitVec,
}

impl BloomFilter {
    fn new(cap: u64, fp: f64) -> Self {
        let m = ((cap as f64) * fp.ln() / (1.0 / 2.0_f64.powf(LN_2)).ln()).ceil() as u64;
        let k = ((m as f64 / cap as f64) * 2.0_f64.ln()).round() as u64;
        let vec = BitVec::from_elem(m as usize, false);

        println!("k={}", k);
        println!("m={}", m);
        println!("n={}", cap);
        println!("Theoretical [false positives] {}%", fp * 100.0);

        Self {
            m_m: m,
            m_k: k,
            m_vec: vec,
        }
    }

    fn contains(&self, s: &str) -> bool {
        let h =  hash_murmur3(s);
        (0..self.m_k).all(|t| {
            let addr = self.hash(h[1], h[0], t + 1);
            self.m_vec.get(addr as usize).unwrap_or(false)
        })
    }

    fn insert(&mut self, s: &str) {
        let h =  hash_murmur3(s);
        for t in 0..self.m_k {
            let addr = self.hash(h[1], h[0], t + 1);
            self.m_vec.set(addr as usize, true);
        }
    }

    fn hash(&self, h1: u64, h2: u64, i: u64) -> u64 {
        ((h1.wrapping_add(i.wrapping_mul(h2.wrapping_add(i)))) % self.m_m) as u64
    }
}

fn main() {
    let fpt = 0.0228;
    let mut fpe = 0.0; // Experimental False Positive Estimation
    let cap = 1000000;
    let mut bf = BloomFilter::new(cap, fpt);

    let mut test = Vec::new();

    // Generate 2M samples from a single random seeder
    for _ in 0..cap * 2 {
        test.push(generate_random_alphanumeric_string(50));
    }

    // Insert first M samples in the filter
    for i in 0..test.len() / 2 {
        bf.insert(&test[i]);
    }

    // Use remaining M samples to test False positives in the filter
    for i in test.len() / 2..test.len() {
        fpe += (bf.contains(&test[i])).then_some(1.0_f64).unwrap_or(0.0_f64);
    }

    println!("Experimental [false positives] = {}%", (fpe / cap as f64) * 100.0_f64);
}

fn generate_random_alphanumeric_string(len: usize) -> String {
    static CHARS: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let result: String = successors(Some(()), |_| Some(()))
        .map(|()| {
            let idx = fastrand::usize(0..CHARS.len());
            CHARS.chars().nth(idx).unwrap()
        })
        .take(len)
        .collect();

    result
}


fn hash_murmur3(s: &str) -> [u64; 2] {
    let hash = Hash128_x64::hash_with_seed(s.as_bytes(),11);
    [(hash >> 64) as u64, hash as u64]
}
