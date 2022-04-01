use lazy_static::lazy_static;
use std::collections::HashMap;
use wordle_generator::{Seed, Wordle};

lazy_static! {
    // D. E. Knuth, The Stanford GraphBase: A platform for combinatorial computing.
    // New York, NY: ACM Press, 2009. https://www-cs-faculty.stanford.edu/~knuth/sgb-words.txt
    static ref WORDS: Vec<&'static str> = include_str!("words.txt").lines().collect();
}

#[test]
fn first_window() {
    let seed: Seed = [0; 32];
    let mut wordle = Wordle::from_seed(&WORDS, 365, seed);

    assert_eq!(wordle.get(0), "maple");
    assert_eq!(wordle.get(1), "swung");
    assert_eq!(wordle.get(2), "koala");
    assert_eq!(wordle.get(246), "yogis");
    assert_eq!(wordle.get(364), "delve");
}

#[test]
fn same_seed() {
    let seed: Seed = [0xF0; 32];
    let other_seed = seed.clone();
    let mut first = Wordle::from_seed(&WORDS, 200, seed);
    let mut second = Wordle::from_seed(&WORDS, 200, other_seed);

    assert_eq!(first.get(0), second.get(0));
    assert_eq!(first.get(1), second.get(1));
    assert_eq!(first.get(2349234), second.get(2349234));
}

#[test]
fn different_seed() {
    let seed: Seed = [0; 32];
    let other_seed: Seed = [1; 32];
    let mut first = Wordle::from_seed(&WORDS, 365, seed);
    let mut second = Wordle::from_seed(&WORDS, 365, other_seed);

    assert_ne!(first.get(0), second.get(0));
    assert_ne!(first.get(434484), second.get(434484));
}

#[test]
fn change_windows() {
    let seed: Seed = [0xF0; 32];
    let mut wordle = Wordle::from_seed(&WORDS, 365, seed);

    assert_eq!(wordle.get(0), "riser");
    assert_eq!(wordle.get(365), "owner");
    assert_eq!(wordle.get(429), "fasts");
    assert_eq!(wordle.get(730), "clunk");
    assert_eq!(wordle.get(14556), "oared");
}

#[test]
fn all_words_before_new_start() {
    let seed: Seed = [0xAB; 32];
    let window_len = WORDS.len() as u64;
    let mut wordle = Wordle::from_seed(&WORDS, window_len, seed);

    let mut occurrences = HashMap::new();
    for i in 1..=5 {
        for day in 0..WORDS.len() as u64 {
            let word = wordle.get(day);
            let entry = occurrences.entry(word).or_insert(0);
            *entry += 1;
        }

        assert_eq!(occurrences.len(), WORDS.len());
        assert!(occurrences.values().all(|&x| x == i));
    }
}

// todo: test non-periodicity
