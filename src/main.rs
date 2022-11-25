use crossbeam_channel::bounded;
use std::{collections, io::Write, thread, time::Instant};

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

        for w in buf.split_ascii_whitespace().filter(|&w| w.len() == 5) {
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

        // Group the words by the least used letters.
        let mut letter_index: [Vec<u32>; 26] = Default::default();
        for w in &all_word_bits {
            // Iterate over the letters in the word and find the least used one.
            let mut m = *w;
            let mut min = reverse_order[m.trailing_zeros() as usize];
            m &= m - 1;
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

    fn words(&self, words: &WordArray) -> [&str; 5] {
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

fn find(
    ctx: &Context,
    res: &mut Vec<WordArray>,
    words: &mut WordArray, // Accumulator.
    word_index: usize,     // And its active length.
    used_letters: u32,
    from_letter: usize,
    mut skipped: bool,
) {
    for (i, letter) in ctx.order.iter().enumerate().skip(from_letter) {
        if used_letters & (1 << letter) != 0 {
            continue;
        }
        for &w in &ctx.letter_index[i] {
            if w & used_letters != 0 {
                continue;
            }
            words[word_index] = w;
            if word_index == 4 {
                res.push(words.clone());
            } else {
                find(
                    &ctx,
                    res,
                    words,
                    word_index + 1,
                    used_letters | w,
                    i + 1,
                    skipped,
                );
            }
        }
        if skipped {
            break;
        }
        skipped = true;
    }
}

fn find_all(ctx: &Context) -> Vec<WordArray> {
    let mut words = WordArray::default();
    let mut res = vec![];
    find(&ctx, &mut res, &mut words, 0, 0, 0, false);
    res
}

fn find_all_par(ctx: &Context) -> Vec<WordArray> {
    let mut res = Vec::new();

    let jobs = thread::available_parallelism().unwrap().into();
    let (res_send, res_recv) = bounded(jobs);
    thread::scope(|scope| {
        let (job_send, job_recv) = bounded(1000);

        for _ in 0..jobs {
            scope.spawn({
                let res_send = res_send.clone();
                let job_recv = job_recv.clone();
                move || {
                    let mut words = WordArray::default();
                    let mut res = vec![];
                    for (w, i, skipped) in job_recv {
                        words[0] = w;
                        find(&ctx, &mut res, &mut words, 1, w, i + 1, skipped);
                    }
                    res_send.send(res).expect("failed to send result");
                }
            });
        }

        let mut skipped = false;
        for (i, _) in ctx.order.iter().enumerate() {
            for &w in &ctx.letter_index[i] {
                job_send
                    .send((w, i, skipped))
                    .expect("failed to create a job");
            }
            if skipped {
                break;
            }
            skipped = true;
        }
    });
    for _ in 0..jobs {
        let mut f = res_recv.recv().expect("failed to receive result");
        res.append(&mut f);
    }

    res
}

fn main() {
    let start = Instant::now();

    let input = include_str!("words_alpha.txt");
    let ctx = Context::from_words(input);

    dbg!(/* unique words */ ctx.all_word_bits.len());

    let start_algo = Instant::now();
    let solutions = find_all_par(&ctx);

    let start_write = Instant::now();
    let mut f = std::io::BufWriter::new(std::fs::File::create("solutions.txt").unwrap());
    for w in solutions.iter().map(|w| ctx.words(w)) {
        w.iter().for_each(|&w| write!(f, "{w}\t").unwrap());
        f.write(b"\n").unwrap();
    }
    drop(f);
    let end = Instant::now();

    println!("{} solutions written to solutions.txt", solutions.len());
    println!("Total time: {:8}µs", (end - start).as_micros());
    println!("Read:       {:8}µs", (start_algo - start).as_micros());
    println!("Process:    {:8}µs", (start_write - start_algo).as_micros());
    println!("Write:      {:8}µs", (end - start_write).as_micros());
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
    fn test_find_all() {
        let words = "abcde fghij klmno pqrst uvwxy zabcd";
        let ctx = Context::from_words(words);
        let words = find_all(&ctx);
        assert_eq!(words.len(), 2);
        assert_eq!(
            ctx.words(&words[0]),
            ["abcde", "fghij", "klmno", "pqrst", "uvwxy"]
        );
        assert_eq!(
            ctx.words(&words[1]),
            ["fghij", "klmno", "pqrst", "uvwxy", "zabcd"]
        );
    }
}
