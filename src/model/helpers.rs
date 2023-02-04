use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

// Adapted from https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html

pub async fn gen_salt() -> String
{
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
