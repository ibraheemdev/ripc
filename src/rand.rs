use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Instant;

pub fn rand() -> u64 {
    pub struct FastRng(Cell<u64>);

    thread_local! {
        static RNG: FastRng = FastRng(Cell::new({
            let mut hasher = DefaultHasher::new();
            Instant::now().hash(&mut hasher);
            thread::current().id().hash(&mut hasher);
            (hasher.finish() << 1) | 1
        }));
    }

    RNG.with(|rng| {
        let mut x = rng.0.get();
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        rng.0.set(x);
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}
