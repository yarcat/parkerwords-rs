use std::collections;

struct Context<'a> {
    frequences: [(u8, usize); 26],
    all_words: Vec<&'a str>,
    all_word_bits: Vec<u32>,
    bits_to_indexes: collections::HashMap<u32, usize>,
}

impl<'a> Context<'a> {
    fn from_words(buf: &'a str) -> Self {
        const ENTRIES: usize = 6_000; // It's gonna be ~6k entries.
        let mut all_words = Vec::with_capacity(ENTRIES);
        let mut all_word_bits = Vec::with_capacity(ENTRIES);
        let mut bits_to_indexes = collections::HashMap::<u32, usize>::with_capacity(ENTRIES);

        let mut frequences = [(0u8, 0usize); 26]; // 26 letters in the alphabet.
        for (i, f) in frequences.iter_mut().enumerate() {
            f.0 = i as u8;
        }

        for w in buf.split_ascii_whitespace() {
            // Skip words of the wrong length.
            if w.len() != 5 {
                continue;
            }
            let bits = word_bits(w.as_bytes().try_into().unwrap());
            // Skip words that don't have enough unique letters or if we have had these already.
            if bits.count_ones() == 5 && !bits_to_indexes.contains_key(&bits) {
                bits_to_indexes.insert(bits, all_word_bits.len());
                all_word_bits.push(bits);
                all_words.push(w);
                for b in w.as_bytes() {
                    frequences[(b - b'a') as usize].1 += 1;
                }
            }
        }

        // Least used letters get the lower index.
        frequences.sort_by_key(|x| (x.1, x.0));
        let mut reverse_order = [0usize; 26];
        for (i, f) in frequences.iter().enumerate() {
            reverse_order[f.0 as usize] = i;
        }

        // for w in &all_word_bits {
        //     let lowest_letter = w.trailing_zeros() as usize;
        //     let mut min = reverse_order[lowest_letter];
        //     let mut m = w & w-1; // Drop the lowest letter from the set.
        //     while m > 0 {
        //         let lowest_letter = m.trailing_zeros() as usize;
        //         min = std::cmp::min(v1, v2)
        //     }
        // }

        Self {
            frequences,
            all_words,
            all_word_bits,
            bits_to_indexes,
        }
    }
}

fn word_bits(w: &[u8; 5]) -> u32 {
    // TODO(yarcat): Make a loop out of this and check the code generated.
    1 << w[0] - b'a' | 1 << w[1] - b'a' | 1 << w[2] - b'a' | 1 << w[3] - b'a' | 1 << w[4] - b'a'
}

fn main() {
    let input = include_str!("words_alpha.txt");
    let ctx = Context::from_words(input);
    for w in ctx.all_words {
        println!("{w}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_word_bits() {
        assert_eq!(word_bits(&"aaaaa".as_bytes().try_into().unwrap()), 0b1);
        assert_eq!(word_bits(&"bbbbb".as_bytes().try_into().unwrap()), 0b10);
        assert_eq!(
            word_bits(&"zzzzz".as_bytes().try_into().unwrap()),
            0b10_0000_0000_0000_0000_0000_0000
        );
        assert_eq!(
            word_bits(&"azhkg".as_bytes().try_into().unwrap()),
            0b10_0000_0000_0000_0100_1100_0001
        );
    }

    #[test]
    fn test_context_creation() {
        let ctx = Context::from_words("to skip abcde cdabe aaaaa efghi");
        assert_eq!(ctx.all_words, ["abcde", "efghi"]);
        assert_eq!(ctx.all_word_bits, [0b00011111, 0b111110000]);
        // Let's sort to ensure those are deterministic.
        let mut bits_to_indexes = ctx.bits_to_indexes.iter().collect::<Vec<_>>();
        bits_to_indexes.sort();
        assert_eq!(bits_to_indexes, [(&0b11111, &0), (&0b111110000, &1)]);
        assert_eq!(
            ctx.frequences[15..],
            [
                (24, 0),
                (25, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (5, 1),
                (6, 1),
                (7, 1),
                (8, 1),
                (4, 2), // e is repeated twice.
            ]
        );
    }
}
