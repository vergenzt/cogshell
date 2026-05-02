use std::sync::atomic::{AtomicU32, Ordering};

fn roll_dice(seed: &AtomicU32) -> u8 {
    let mut random = seed.load(Ordering::SeqCst);
    random ^= random << 13;
    random ^= random >> 17;
    random ^= random << 5;
    seed.store(random, Ordering::SeqCst);

    (random % 6 + 1) as u8
}


