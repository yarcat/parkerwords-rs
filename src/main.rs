use std::collections;

struct Context<'a> {
    frequences: [(u8, usize); 26],
    all_words: Vec<&'a str>,
    all_word_bits: Vec<u32>,
    bits_to_indexes: collections::HashMap<u32, usize>,
    letter_index: [Vec<u32>; 26],
    order: [u8; 26],
    reverse_order: [usize; 26],
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
        let mut order = [0u8; 26];
        let mut reverse_order = [0usize; 26];
        for (i, f) in frequences.iter().enumerate() {
            order[i] = f.0;
            reverse_order[f.0 as usize] = i;
        }

        // For each word we build an index where the first
        let mut letter_index: [Vec<u32>; 26] = Default::default();
        for w in &all_word_bits {
            let mut m = *w;
            let mut min = reverse_order[m.trailing_zeros() as usize]; // Lowest letter.
            m &= m - 1; // Drop the lowest letter;
            while m != 0 {
                min = std::cmp::min(min, reverse_order[m.trailing_zeros() as usize]);
                m &= m - 1;
            }
            letter_index[min].push(*w);
        }

        Self {
            frequences,
            all_words,
            all_word_bits,
            bits_to_indexes,
            letter_index,
            order,
            reverse_order,
        }
    }

    fn words(&self, words: WordArray) -> [&str; 5] {
        words
            .iter()
            .map(|w| self.all_words[self.bits_to_indexes[w]])
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

fn word_bits(w: &[u8; 5]) -> u32 {
    // TODO(yarcat): Make a loop out of this and check the code generated.
    1 << w[0] - b'a' | 1 << w[1] - b'a' | 1 << w[2] - b'a' | 1 << w[3] - b'a' | 1 << w[4] - b'a'
}

type WordArray = [u32; 5]; // 5 bit sets representing 5 words.

struct Finder<'a> {
    ctx: &'a Context<'a>,
    res: Vec<WordArray>,
}

impl<'a> Finder<'a> {
    fn new(ctx: &'a Context) -> Self {
        Self {
            ctx,
            res: Vec::with_capacity(100),
        }
    }

    fn find_all(&mut self) -> &Vec<WordArray> {
        let mut words = WordArray::default();
        self.find(&mut words, 0, 0, 0, false);
        &self.res
    }

    fn find(
        &mut self,
        words: &mut WordArray, // Accumulator.
        words_found: usize,    // And its length.
        selected_letters: u32,
        from_letter: usize,
        mut skipped: bool,
    ) {
        for i in from_letter..26 {
            let letter = self.ctx.frequences[i].0;
            let m = 1 << letter;
            if selected_letters & m != 0 {
                // No new letters.
                continue;
            }

            for &w in &self.ctx.letter_index[i] {
                if w & selected_letters != 0 {
                    // No new letters.
                    continue;
                }
                words[words_found] = w;
                if words_found == 4 {
                    // We've found all 5 words.
                    self.res.push(words.clone());
                } else {
                    self.find(words, words_found + 1, selected_letters | w, i + 1, skipped);
                }
            }
            if skipped {
                break;
            }
            skipped = true;
        }
    }
}

fn find_all_words(ctx: &Context) -> Vec<WordArray> {
    let mut f = Finder::new(ctx);
    f.find_all();

    // use crossbeam_channel::bounded;
    // let (s, r) = bounded(100);

    // let mut threads = vec![];
    // for _ in 0..dbg!(num_cpus::get()) {
    //     let r = r.clone();
    //     threads.push(std::thread::spawn(move || {
    //         for m in r {
    //             println!("received {m}");
    //         }
    //     }));
    // }

    // for i in 0..10 {
    //     s.send(format!("hello {i}")).expect("hello was not sent");
    //     s.send(format!("world {i}")).expect("world was not sent");
    // }
    // drop(s);

    // for t in threads {
    //     let _ = t.join();
    // }
    f.res
}

fn main() {
    let input = include_str!("words_alpha.txt");
    let ctx = Context::from_words(input);

    // let mut solution = Vec::with_capacity(10_000);

    dbg!(/* unique words */ ctx.all_word_bits.len());

    let solutions = find_all_words(&ctx);
    println!("solutions: {num}", num = solutions.len());
    println!("{solutions:#?}");
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

    #[test]
    fn test_finder() {
        let words = "abcde fghij klmno pqrst uvwxy zabcd";
        let ctx = Context::from_words(words);
        let mut f = Finder::new(&ctx);
        let words = f.find_all();
        assert_eq!(words.len(), 2);
        assert_eq!(
            ctx.words(words[0]),
            ["abcde", "fghij", "klmno", "pqrst", "uvwxy"]
        );
        assert_eq!(
            ctx.words(words[1]),
            ["fghij", "klmno", "pqrst", "uvwxy", "zabcd"]
        );
    }
}
