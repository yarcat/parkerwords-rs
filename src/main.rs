use crossbeam_channel::bounded;
use std::{collections, thread};

struct Context<'a> {
    all_words: Vec<&'a str>,
    all_word_bits: Vec<u32>,
    bits_to_indexes: collections::HashMap<u32, usize>,
    letter_index: [Vec<u32>; 26],
    order: [u8; 26],
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
            all_words,
            all_word_bits,
            bits_to_indexes,
            letter_index,
            order,
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
            res: Vec::with_capacity(1_000),
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
        word_index: usize,     // And its active length.
        used_letters: u32,
        from_letter: usize,
        mut skipped: bool,
    ) {
        for (i, letter) in self.ctx.order.iter().enumerate().skip(from_letter) {
            if used_letters & (1 << letter) != 0 {
                // No new letters.
                continue;
            }

            for &w in &self.ctx.letter_index[i] {
                if w & used_letters != 0 {
                    // No new letters.
                    continue;
                }
                words[word_index] = w;
                if word_index == 4 {
                    // We've found all 5 words.
                    self.res.push(words.clone());
                } else {
                    self.find(words, word_index + 1, used_letters | w, i + 1, skipped);
                }
            }
            if skipped {
                break;
            }
            skipped = true;
        }
    }
}

fn find_all_words<'a>(ctx: &'a Context) -> Vec<WordArray> {
    // let mut f = Finder::new(ctx);
    // return  f.find_all().clone();
    thread::scope(|scope| {
        let (s, r) = bounded(1000);
        for _ in 0..num_cpus::get_physical() {
            scope.spawn({
                let r = r.clone();
                move || {
                    let mut f = Finder::new(&ctx);
                    let mut words = WordArray::default();
                    for (w, i) in r {
                        words[0] = w;
                        f.find(&mut words, 1, w, i + 1, false);
                    }
                    println!("!!! {}", f.res.len());
                }
            });
        }
        let mut cnt = 0;
        for (i, _) in ctx.order.iter().enumerate() {
            for &w in &ctx.letter_index[i] {
                cnt += 1;
                s.send((w, i)).expect("failed to create a job");
            }
        }
        println!("created {cnt} jobs");
    });
    println!("terminated");
    Vec::new()
}

fn main() {
    let input = include_str!("words_alpha.txt");
    let ctx = Context::from_words(input);

    // let mut solution = Vec::with_capacity(10_000);

    dbg!(/* unique words */ ctx.all_word_bits.len());

    let solutions = find_all_words(&ctx);
    println!("solutions: {num}", num = solutions.len());
    // println!("{solutions:#?}");
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
        assert_eq!(ctx.order[15..], [24, 25, 0, 1, 2, 3, 5, 6, 7, 8, 4,]); // e is repeated twice.
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
