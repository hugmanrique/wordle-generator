use lazy_static::lazy_static;
use std::collections::HashMap;
use wordle_sequence::feistel::KEY_LEN;
use wordle_sequence::Wordle;

lazy_static! {
    // D. E. Knuth, The Stanford GraphBase: A platform for combinatorial computing.
    // New York, NY: ACM Press, 2009. https://www-cs-faculty.stanford.edu/~knuth/sgb-words.txt
    static ref WORDS: Vec<&'static str> = include_str!("words.txt").lines().collect();
}

#[test]
fn first_days() {
    let key = [0xF0; KEY_LEN];
    let mut wordle = Wordle::new_with_key(&WORDS, key);

    assert_eq!(wordle.next(), Some("craws"));
    assert_eq!(wordle.next(), Some("gaols"));
    assert_eq!(wordle.next(), Some("agone"));
    assert_eq!(wordle.next(), Some("spays"));
    assert_eq!(wordle.next(), Some("early"));
    assert_eq!(wordle.next(), Some("gable"));
}

#[test]
fn nth() {
    let key = [0xF0; KEY_LEN];
    let mut wordle = Wordle::new_with_key(&WORDS, key);

    assert_eq!(wordle.nth(0), Some("craws"));
    assert_eq!(wordle.nth(1), Some("agone"));
    assert_eq!(wordle.nth(2), Some("gable"));
    assert_eq!(wordle.nth(14556), Some("shore"));
    assert_eq!(wordle.nth(2 * WORDS.len()), Some("sedan"));
}

#[test]
fn all_words_before_new_start() {
    let key = [123; KEY_LEN];
    let mut wordle = Wordle::new_with_key(&WORDS, key);

    let mut occurrences = HashMap::new();
    for i in 1..=5 {
        for _ in 0..WORDS.len() {
            let word = wordle.next().unwrap();
            let entry = occurrences.entry(word).or_insert(0);
            *entry += 1;
        }

        assert_eq!(occurrences.len(), WORDS.len());
        assert!(occurrences.values().all(|&x| x == i));
    }
}
