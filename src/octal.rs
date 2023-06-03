// use super::game;
use bitvec::prelude::*;
use cfg_if;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::time::Instant;

use rayon::{current_num_threads, prelude::*};

/// Rule represents possible moves from a position n after removing some i tokens are removed from a heap
///
/// If all is true, then 0 may be the successor of n if n == i (all tokens may be taken from the
/// heap).
/// if some is true, then n - i may be the successor of n if n > i (some, but not all tokens may
/// be taken from the heap).
/// If divide is true, then a pair (i, n - i) may be the successor of n if n > i (some tokens may
/// be taken from the heap, but the heap must be divided into two nonempty heaps after the tokens
/// are taken).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rule {
    pub all: bool,
    pub some: bool,
    pub divide: bool,
}

impl From<char> for Rule {
    fn from(c: char) -> Self {
        let d = c.to_digit(10).unwrap();
        Rule {
            all: ((d & 1) != 0),
            some: ((d & 2) != 0),
            divide: ((d & 4) != 0),
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "bits_bitvec")] {
        pub type BitV = BitVec<u64, Msb0>;
    } else if #[cfg(feature = "bits_u32")] {
        pub type BitV = u32;
    } else if #[cfg(feature = "bits_u64")] {
        pub type BitV = u64;
    } else if #[cfg(feature = "bits_u128")] {
        pub type BitV = u128;
    } else {
        pub type BitV = BitVec<u64, Msb0>;
    }
}

#[derive(Clone)]
pub struct Bin {
    bits: BitV,
}

#[cfg(any(
    feature = "bits_bitvec",
    not(any(feature = "bits_u32", feature = "bits_u64", feature = "bits_u128"))
))]
impl Bin {
    fn set_bit(&mut self, x: usize) {
        self.bits.set(x, true);
    }

    fn zero_bits(&mut self) {
        self.bits.set_elements(0);
    }

    fn get(&self, x: usize) -> bool {
        self.bits[x]
    }

    fn lowest_unset(&self) -> usize {
        self.bits.first_zero().unwrap()
    }

    fn make(largest: Nimber) -> Self {
        let bs = 2 * (largest as usize).next_power_of_two() + 2;
        Self {
            bits: bitvec!(u64, Msb0; 0; bs),
        }
    }

    fn count_unset(&self) -> usize {
        self.bits.count_zeros()
    }

    fn find_first_unset_also_unset_in(&self, other: &Self) -> usize {
        for i in 0..other.bits.len() {
            if !self.get(i) && !other.get(i) {
                return i;
            }
        }

        self.bits.len() - 1
    }

    fn copy_up_to_inclusive(&self, x: usize) -> Self {
        Self {
            bits: self.bits[0..x].to_owned(),
        }
    }

    fn set_all_bits_from(&mut self, other: &Self) {
        self.bits |= &other.bits
    }
}

#[cfg(all(
    not(feature = "bits_bitvec"),
    any(feature = "bits_u32", feature = "bits_u64", feature = "bits_u128")
))]
impl Bin {
    fn set_bit(&mut self, x: usize) {
        self.bits |= 1 << x;
    }

    fn zero_bits(&mut self) {
        self.bits = 0;
    }

    fn get(&self, x: usize) -> bool {
        self.bits & 1 << x != 0
    }

    fn lowest_unset(&self) -> usize {
        self.bits.trailing_ones() as usize
    }

    fn make(largest: Nimber) -> Self {
        let bs = (largest as u32 + 2).next_power_of_two() + 2;
        println!("{} largest, {} bs {}", largest, bs, BitV::BITS);
        // assert!(bs < BitV::BITS);
        Self {
            bits: if bs > BitV::BITS {
                0
            } else {
                !(0 as BitV) << (bs)
            },
        }
    }

    fn count_unset(&self) -> usize {
        self.bits.count_zeros() as usize
    }

    fn find_first_unset_also_unset_in(&self, other: &Self) -> usize {
        Self {
            bits: (self.bits | other.bits),
        }
        .lowest_unset()
    }

    fn copy_up_to_inclusive(&self, _x: usize) -> Self {
        // self.clone()
        Self {
            bits: ((!0) << _x) | self.bits,
        } // maybe set upper bits to 1?
    }

    fn set_all_bits_from(&mut self, other: &Self) {
        self.bits |= &other.bits
    }
}

/// Transform a game string like "0.034" into a Vector of Rules
///
/// I-th element of the vector is a Rule which represents possible moves after removing i tokens
/// from a heap.
pub fn rules_from_str(game: &str) -> Vec<Rule> {
    game.chars()
        .filter(|&x| x != '.')
        .map(|c| Rule::from(c))
        .collect()
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nimber_u8")] {
        pub type Nimber = u8;
    } else if #[cfg(feature = "nimber_u16")] {
        pub type Nimber = u16;
    } else if #[cfg(feature = "nimber_u32")] {
        pub type Nimber = u32;
    } else {
        pub type Nimber = u16;
    }
}

pub struct Nimbers {
    pub g: Vec<Nimber>,
    pub g_back: Vec<Nimber>,
    pub rare: Vec<(usize, Nimber)>,
}

impl Nimbers {
    pub fn new(max_full_memory: usize, max_tail_memory: usize) -> Self {
        Self {
            g: vec![Nimber::max_value(); max_full_memory],
            g_back: vec![Nimber::max_value(); max_tail_memory],
            rare: vec![],
        }
    }

    pub fn last(&self, n: usize) -> Nimber {
        self.g_back[n % self.g_back.len()]
    }

    pub fn copy_to_g_back(&mut self) {
        let max_tail_memory = self.g_back.len();
        let max_full_memory = self.g.len();
        self.g_back = self.g[(max_full_memory - max_tail_memory)..max_full_memory].into();
    }
}

pub struct Stats {
    pub largest_nimber: Nimber,
    pub frequencies: Vec<usize>,
    pub largest_nimber_index: usize,
    pub prev_values: usize,
    pub latest_rare: Nimber,
    pub latest_rare_index: usize,
}

pub struct Bits {
    pub rare: Bin,
    pub seen: Bin,
    pub partial: Vec<Bin>,
}

// pub fn make_bitset(largest: Nimber) -> BitV {
//     let bits = 2 * (largest as usize).next_power_of_two() + 2;
//     bitvec!(u64, Msb0; 0; bits)
// }

impl Bits {
    pub fn new() -> Self {
        Self {
            rare: Bin::make(0),
            seen: Bin::make(0),
            partial: vec![Bin::make(0); rayon::current_num_threads()],
        }
    }

    pub fn resize(&mut self, largest_nimber: Nimber) {
        self.rare = Bin::make(largest_nimber);
        self.seen = Bin::make(largest_nimber);
        self.partial = vec![Bin::make(largest_nimber); rayon::current_num_threads()];
    }
}

impl Stats {
    pub fn new() -> Self {
        Self {
            largest_nimber: Nimber::min_value(),
            frequencies: vec![],
            largest_nimber_index: 0,
            prev_values: 0,
            latest_rare: Nimber::min_value(),
            latest_rare_index: 0,
        }
    }

    pub fn initialize(&mut self, front: &Vec<Nimber>, first_uninitialized: usize) {
        self.set_largest_nimber(front, first_uninitialized);
        self.resize_frequencies();
        self.initialize_frequencies(front, first_uninitialized);
    }

    pub fn resize_frequencies(&mut self) {
        self.frequencies
            .resize((self.largest_nimber as usize + 2).next_power_of_two(), 0);
    }
    pub fn set_largest_nimber(&mut self, g: &Vec<Nimber>, first_uninitialized: usize) {
        self.largest_nimber = *g[1..first_uninitialized].iter().max().unwrap_or(&2);
    }

    pub fn initialize_frequencies(&mut self, g: &Vec<Nimber>, first_uninitialized: usize) {
        for n in 0..first_uninitialized {
            self.frequencies[g[n] as usize] += 1;
        }
    }

    /// Generate a bit vector of rare values, maximizing the sum of unset frequencies from
    /// self.frequencies.
    ///
    /// The bitset has to fulfil the following criteria:
    /// * for all set bit x, y in rares  x ^ y is also set,
    /// * for all unset bits x, y in rares, x ^ y is set,
    /// * for all set bits x and unset bits y in C, x ^ y in unset.
    /// while at the same time maximizing the sum of freq[x] if rares[x] is unset.
    pub fn gen_rares(&self) -> Bin {
        let mut r = HashSet::new();
        let mut c = HashSet::new();
        let mut vals: Vec<(usize, usize)> =
            self.frequencies.iter().map(|&e| e).enumerate().collect();
        vals.sort_by_key(|(_, f)| Reverse(*f));

        r.insert(0);
        for (x, _) in vals {
            if r.contains(&x) || c.contains(&x) {
                continue;
            } else {
                c.insert(x);
                let mut inserted = true;
                while inserted {
                    inserted = false;
                    for &c1 in c.iter() {
                        for &c2 in c.iter() {
                            inserted |= r.insert(c1 ^ c2);
                        }
                    }

                    let mut new_r = r.to_owned();

                    for &r1 in r.iter() {
                        for &r2 in r.iter() {
                            if r1 != 0 && r2 != 0 && r1 != r2 {
                                inserted |= new_r.insert(r1 ^ r2);
                            }
                        }
                    }

                    r = new_r;

                    let mut new_c = c.to_owned();

                    for &r1 in r.iter() {
                        for &c1 in c.iter() {
                            inserted |= new_c.insert(r1 ^ c1);
                        }
                    }

                    c = new_c;
                }
            }
        }

        let mut rares = Bin::make(self.largest_nimber);
        for &x in r.iter() {
            rares.set_bit(x)
        }
        rares
    }
}

pub struct Game {
    pub rules: Vec<Rule>,
    pub nimbers: Nimbers,
    pub stats: Stats,
    pub bits: Bits,
}

#[derive(Serialize, Deserialize)]
struct Freq {
    nimber: usize,
    frequency: usize,
    rare: bool,
}

impl Game {
    pub fn new(rules_str: &str, max_full_memory: usize, max_tail_memory: usize) -> Self {
        Game {
            rules: rules_from_str(rules_str),
            nimbers: Nimbers::new(max_full_memory, max_tail_memory),
            stats: Stats::new(),
            bits: Bits::new(),
        }
    }

    /// Initialize first `rules.len()` elements of g with nim-values of positions.
    ///
    /// Calculate the first `rules.len()` elements naively, but while checking if the rule may be
    /// applied (for n in 0..rules.len(), check if i > n).
    /// This check is unnecessary for n's larger than `rules.len()`.
    pub fn initialize(&mut self) {
        self.nimbers.g[0] = 0;

        for n in 1..self.rules.len() {
            let mut seen = bitvec!(u64, Msb0; 0; 2 * self.rules.len() + 2);

            if n < self.rules.len() && self.rules[n].all {
                seen.set(0, true);
            }

            for i in 1..self.rules.len() {
                if self.rules[i].some && n > i {
                    seen.set(self.nimbers.g[n - i] as usize, true);
                }

                if self.rules[i].divide && n > i {
                    for j in 1..=(n - i) / 2 {
                        let x = self.nimbers.g[j];
                        let y = self.nimbers.g[n - i - j];
                        seen.set((x ^ y) as usize, true);
                    }
                }
            }

            self.nimbers.g[n as usize] = seen.first_zero().unwrap() as Nimber;
        }
    }

    pub fn init(&mut self) {
        self.initialize();
        let first_uninitialized = self.rules.len();

        self.stats.initialize(&self.nimbers.g, first_uninitialized);
        self.resize(first_uninitialized - 1);
    }

    pub fn set_seen_bits_from_some_moves(&mut self, n: usize) {
        // set the non-xor values
        for i in 1..self.rules.len() {
            if self.rules[i].some {
                self.bits.seen.set_bit(self.nimbers.g[n - i] as usize);
            }
        }
    }

    pub fn set_seen_bits_from_some_moves_back(&mut self, n: usize) {
        // set the non-xor values
        for i in 1..self.rules.len() {
            if self.rules[i].some {
                self.bits.seen.set_bit(self.nimbers.last(n - i) as usize);
            }
        }
    }

    /// Compute the nimber g[n] leveraging the sparce space phenonmenon, under the following
    /// assumptions:
    /// * self.nimbers.g[0..n] were computed correctly, accodring to the rules of some octal game,
    /// * n is at least rules.len(),
    /// * self.bits.rare is a binary vector which in which a set bit at position i signifies that i
    /// is a member of R
    /// * the self.nimbers.rare vector represents a correct decmposition into R and C sets (that is
    /// values from g are decomposed into two mutually exclusive sets R and C such that for all x,
    /// y in R. x ^ y in R, and for all x, y in C, x ^ y in R, and for all x in R, y in C, x ^ y in
    /// C).  * rare_idx_and_nimber is a vector in which contains all pairs (index, nimber) for
    /// nimbers from g such that rares[nimber] is true (in python terms: rare_idx_and_nimber =
    /// [(index,nimber) for nimber in g if rares[nimber]]).
    ///
    /// The sparse space phenomenon is an observeable phenomenon in at least some octal games, where
    /// the set of nimbers is divisible into two sets: the common (C) and the rare (R) sets, such that:
    /// * for all x, y in R. x ^ y in R,
    /// * for all x, y in C, x ^ y in R,
    /// * for all x in R, y in C, x ^ y in C.
    /// Since most successsors of a position are in the form x ^ y where both x and y are common, then
    /// a position in an octal game is more likely to *not* have a rare value.
    /// It is therefore worthwhile to first check the values of all successsors in the from x ^ y,
    /// where x is rare.
    /// Since according to the Sprague-Grundy theorem, the nim-value of a position `n` is the smallest
    /// natural number such, that is not the nim-value of `n`'s successors, this eliminates all possible
    /// common values from the set of possible values of the position `n`.
    /// The most likely candidate for the  nim-value of `n` is therefore the smallest common value, or a
    /// new rare value (but this is unlikely).
    /// To prove that it is the common value, we need to iterate through the unchecked successors, and
    /// eliminate rare values smaller than the candidate.
    /// If this is successful, then the nim-value of `n` is the candidate.
    /// If this is unsuccessful, then the nim-value of `n` is rare.
    ///
    /// The advantage of this algorithm is that if number of elements of R is small (if values in R
    /// appear in g at most some constant amount of times), then the algorithm approaches a linear time
    /// complexity.  This is especially important, since the time complexity of the naive approach has
    /// quadratic time complexity wrt n.
    ///
    /// Note an observation made by dr Piotr Beling, that this algorithm works also when the chosen
    /// R and C do not correctly contain infrequent and frequent values in g, but as long as the
    /// two sens fulfil the criteria outlined above, the algorithm works correctly (although it may
    /// work even slower than the naive).  For instance, if we assume that all values are in R, and
    /// C is an empty set, then this algorithm still correctly identifies nimbers.
    pub fn rc(&mut self, n: usize) -> Nimber {
        self.bits.seen.zero_bits();
        self.bits.partial.iter_mut().for_each(|x| x.zero_bits());

        self.set_seen_bits_from_some_moves(n);
        self.set_0th_bit_if_can_be_divided_in_half(n);
        self.iterate_over_r_xor_c_mt(n);

        self.prove_mt(n)
        // self.prove(n)
    }

    pub fn rc_back(&mut self, n: usize) -> Nimber {
        self.bits.seen.zero_bits();

        self.set_seen_bits_from_some_moves_back(n);
        self.set_0th_bit_if_can_be_divided_in_half(n);
        self.iterate_over_r_xor_c_back(n);

        self.prove_back(n)
    }

    /// Naively compute the nimber g[n] assuming g[0..n] were computed correctly, accodring to the
    /// rules of some octal game, assuming that at n is at least rules.len().
    ///
    /// Assumption that n is at least rules.len() makes it possible to omit some checks (for instance,
    /// there are no more whole moves possible, and some and divide rules are always applicable, since
    /// n is greater than rules.len(0).
    pub fn naive(&mut self, n: usize) -> Nimber {
        assert!(n >= self.rules.len());
        self.bits.seen.zero_bits();

        for i in 1..self.rules.len() {
            if self.rules[i].some {
                self.bits.seen.set_bit(self.nimbers.g[n - i] as usize);
            }

            if self.rules[i].divide {
                for j in 1..=(n - i) / 2 {
                    let x = self.nimbers.g[j];
                    let y = self.nimbers.g[n - i - j];
                    self.bits.seen.set_bit((x ^ y) as usize);
                }
            }
        }

        self.bits.seen.lowest_unset() as Nimber
    }

    pub fn set_next_g_n(&mut self, n: usize, nim: Nimber) {
        self.nimbers.g[n] = nim;

        if nim >= self.stats.largest_nimber {
            self.stats.largest_nimber_index = n;
        }
        if nim > self.stats.largest_nimber {
            self.stats.largest_nimber = nim;
            println!("resizing {}", nim);
            self.resize(n);
            println!("resizing finished");
        }

        self.stats.frequencies[nim as usize] += 1;

        if n < self.nimbers.g.len() && self.bits.rare.get(nim as usize) {
            self.nimbers.rare.push((n, nim));
        }

        if n.is_power_of_two() {
            self.resize(n);
        }
    }

    pub fn set_next_g_back(&mut self, n: usize, nim: Nimber) {
        let loc = n % self.nimbers.g_back.len();
        self.nimbers.g_back[loc] = nim;

        if nim >= self.stats.largest_nimber {
            self.stats.largest_nimber_index = n;
        }
        if nim > self.stats.largest_nimber {
            self.stats.largest_nimber = nim;
            println!("resizing {}", nim);
            self.resize(n);
            println!("resizing finished");
        }

        self.stats.frequencies[nim as usize] += 1;

        if n < self.nimbers.g.len() && self.bits.rare.get(nim as usize) {
            self.nimbers.rare.push((n, nim));
        }

        if n.is_power_of_two() {
            self.resize(n);
        }
    }

    pub fn dump_stats(&self, n: usize, start: &Instant) {
        println!(
            " {:10}s ({:.2} nimbers/s), prev={}, largest={} @ {}, rares={}, latest_rare={} @ {}, G({}) = {}",
            start.elapsed().as_secs(),
            n as u64 / std::cmp::max(1, start.elapsed().as_secs()),
            self.stats.prev_values,
            self.stats.largest_nimber,
            self.stats.largest_nimber_index,
            self.nimbers.rare.len() + 1, // +1 for (0, 0)
            self.stats.latest_rare,
            self.stats.latest_rare_index,
            n,
            self.nimbers.g[n],
        );
    }

    pub fn dump_stats_back(&self, skipped: usize, n: usize, start: &Instant) {
        println!(
            " {:10}s ({:.2} nimbers/s), prev={}, largest={} @ {}, rares={}, latest_rare={} @ {}, G({}) = {}",
            start.elapsed().as_secs(),
            (n - skipped) as u64 / std::cmp::max(1, start.elapsed().as_secs()),
            self.stats.prev_values,
            self.stats.largest_nimber,
            self.stats.largest_nimber_index,
            self.nimbers.rare.len() + 1, // +1 for (0, 0)
            self.stats.latest_rare,
            self.stats.latest_rare_index,
            n,
            self.nimbers.g_back[n % self.nimbers.g_back.len()],
        );
    }

    pub fn occasional_info(&mut self, n: usize, start: &Instant) {
        let max = self.nimbers.g.len();
        let inc = if max > (2 as usize).pow(30) {
            max / 1000
        } else {
            max / 100
        };

        if n % 100000 == 0 {
            // if n % 1 == 0 {
            self.dump_stats(n, &start);
        }

        if n.is_power_of_two() {
            self.dump_freqs(n, start);
        }

        if n % inc == 0 {
            let rate = n as u64 / std::cmp::max(1, start.elapsed().as_secs());
            let estimated_total = max as u64 / rate;
            let estimated_left = (max - n) as u64 / rate;
            println!(
                "{}%, will finish in approximately: {}s (total {}s)",
                (n * 100 / max),
                estimated_left,
                estimated_total,
            )
        }
    }

    pub fn occasional_info_back(&mut self, skipped: usize, n: usize, start: &Instant) {
        if n % 100000 == 0 {
            self.dump_stats_back(skipped, n, &start);
        }

        if n.is_power_of_two() {
            self.dump_freqs(n, start);
        }
    }

    pub fn calc_rc(&mut self, n: usize) {
        let nim = self.rc(n);
        self.set_next_g_n(n, nim);
    }

    pub fn calc_rc_back(&mut self, n: usize) {
        let nim = self.rc_back(n);
        self.set_next_g_back(n, nim);
    }

    pub fn calc_naive(&mut self, n: usize) {
        let nim = self.naive(n);
        self.set_next_g_n(n, nim);
    }

    pub fn dump_freqs(&self, n: usize, start: &Instant) {
        println!("{} freqs after {:?}", n, start.elapsed());

        let fs: Vec<Freq> = self
            .stats
            .frequencies
            .iter()
            .enumerate()
            .map(|(nimber, &frequency)| Freq {
                nimber,
                frequency,
                rare: self.bits.rare.get(nimber),
            })
            .collect();

        let formatted_json = serde_json::to_string_pretty(&fs).unwrap();
        println!("{}", formatted_json);
    }

    fn resize(&mut self, n: usize) {
        self.stats.resize_frequencies();
        self.bits.resize(self.stats.largest_nimber);
        self.bits.rare = self.stats.gen_rares();
        self.nimbers.rare.clear();
        for i in 1..std::cmp::min(n + 1, self.nimbers.g.len()) {
            if self.bits.rare.get(self.nimbers.g[i] as usize) {
                self.nimbers.rare.push((i, self.nimbers.g[i]));
            }
        }
    }

    fn prove(&mut self, n: usize) -> Nimber {
        let first_common = self
            .bits
            .seen
            .find_first_unset_also_unset_in(&self.bits.rare);

        let mut mex = self.bits.seen.copy_up_to_inclusive(first_common + 1);
        let mut remaining_unset = mex.count_unset() - 1; // -1 for mex[first_common]

        for i in 1..self.rules.len() {
            if remaining_unset == 0 {
                return first_common as Nimber;
            }

            if self.rules[i].divide {
                for j in 1..=(n - i) / 2 {
                    let a = self.nimbers.g[j];
                    let b = self.nimbers.g[n - i - j];
                    let loc = (a ^ b) as usize;

                    if loc < first_common && !mex.get(loc) {
                        // a rare value smaller than first_common and not previously observed found
                        mex.set_bit(loc);
                        remaining_unset -= 1;
                        if remaining_unset == 0 {
                            // all smaller values than first_common found, the value is the smallest
                            // not observed common
                            self.stats.prev_values = std::cmp::max(self.stats.prev_values, j);
                            return first_common as Nimber;
                            // break
                        }
                    }
                }
            }
        }

        let nim = mex.lowest_unset() as Nimber;
        self.stats.latest_rare = nim;
        self.stats.latest_rare_index = n;

        nim
    }

    fn prove_mt(&mut self, n: usize) -> Nimber {
        let first_common = self
            .bits
            .seen
            .find_first_unset_also_unset_in(&self.bits.rare);

        let mut mex = self.bits.seen.copy_up_to_inclusive(first_common + 1);
        let mut mex_partial = vec![mex.clone(); rayon::current_num_threads()];
        // let mut remaining_unset = mex.count_unset();

        let chunk_size = 4096;
        let step = rayon::current_num_threads() * chunk_size;

        for i in 1..self.rules.len() {
            // if remaining_unset == 1 {
            //     return first_common as Nimber;
            // }

            if self.rules[i].divide {
                // let last_chunk = 0;
                let end = (n - i) / 2;
                for start in (1..=end).step_by(step) {
                    for par in mex_partial.iter_mut() {
                        par.set_all_bits_from(&mex)
                    }

                    self.nimbers.g[start..std::cmp::min(end + 1, start + step)]
                        .par_chunks(chunk_size)
                        .zip(mex_partial.par_iter_mut())
                        .enumerate()
                        .for_each(|(idx, (xs, par))| //{println!("{}", idx);}
                        {
                            for (k, x) in xs.iter().enumerate() {
                                let j = k + start + idx * chunk_size;
                                let b = self.nimbers.g[n - i - j];
                                let loc = (x ^ b) as usize;
                                if loc < first_common && !par.get(loc) {
                                    // if loc < first_common {
                                    par.set_bit(loc);
                                }
                            }
                        });

                    // mex_partial
                    //     .par_iter_mut()
                    //     .enumerate()
                    //     .for_each(|(idx, par)| {
                    //         // let idx = rayon::current_thread_index().unwrap();
                    //         let shift = start + idx * chunk_size;
                    //         let last = std::cmp::min(end + 1, shift + chunk_size);
                    //         for j in shift..last {
                    //             let a = self.nimbers.g[j];
                    //             let b = self.nimbers.g[n - i - j];
                    //             let loc = (a ^ b) as usize;

                    //             if loc < first_common && !par.get(loc) {
                    //                 // if loc < first_common {
                    //                 par.set_bit(loc);
                    //             }
                    //         }
                    //     });

                    for par in mex_partial.iter() {
                        mex.set_all_bits_from(par);
                    }

                    if mex.count_unset() == 1 {
                        return first_common as Nimber;
                    }
                }

                // println!("{end} {last_chunk}");
            }
        }

        let nim = mex.lowest_unset() as Nimber;
        self.stats.latest_rare = nim;
        self.stats.latest_rare_index = n;

        nim
    }

    fn prove_back(&mut self, n: usize) -> Nimber {
        let first_common = self
            .bits
            .seen
            .find_first_unset_also_unset_in(&self.bits.rare);

        let mut mex = self.bits.seen.copy_up_to_inclusive(first_common + 1);
        let mut remaining_unset = mex.count_unset() - 1; // -1 for mex[first_common]

        for i in 1..self.rules.len() {
            if remaining_unset == 0 {
                return first_common as Nimber;
            }

            if self.rules[i].divide {
                let shift = (self.nimbers.g_back.len() + n - i) % self.nimbers.g_back.len();
                let mut f = 1;
                for j in (1..=shift).rev() {
                    let a = self.nimbers.g[f];
                    f += 1;
                    let b = self.nimbers.g_back[j];
                    let loc = (a ^ b) as usize;

                    if loc < first_common && !mex.get(loc) {
                        // a rare value smaller than first_common and not previously observed found
                        mex.set_bit(loc);
                        remaining_unset -= 1;
                        if remaining_unset == 0 {
                            // all smaller values than first_common found, the value is the smallest
                            // not observed common
                            self.stats.prev_values = std::cmp::max(self.stats.prev_values, f);
                            return first_common as Nimber;
                            // break
                        }
                    }
                }

                for j in (shift + 1..self.nimbers.g_back.len()).rev() {
                    let a = self.nimbers.g[f];
                    f += 1;
                    let b = self.nimbers.g_back[j];
                    let loc = (a ^ b) as usize;

                    if loc < first_common && !mex.get(loc) {
                        // a rare value smaller than first_common and not previously observed found
                        mex.set_bit(loc);
                        remaining_unset -= 1;
                        if remaining_unset == 0 {
                            // all smaller values than first_common found, the value is the smallest
                            // not observed common
                            self.stats.prev_values = std::cmp::max(self.stats.prev_values, f);
                            return first_common as Nimber;
                            // break
                        }
                    }
                }
            }
        }

        let nim = mex.lowest_unset() as Nimber;
        self.stats.latest_rare = nim;
        self.stats.latest_rare_index = n;
        panic!("unexpectedly, larger rare value found! G({}) = {}", n, nim)
    }

    fn iterate_over_r_xor_c_mt(&mut self, n: usize) {
        // iterate over x ^ y such that x is in R
        for i in 1..self.rules.len() {
            if self.rules[i].divide {
                let mut m = self.nimbers.rare.len();
                while m > 0 && n <= i + self.nimbers.rare[m - 1].0 {
                    m -= 1;
                }
                self.nimbers.rare[0..m]
                    .par_chunks(1 + (m / rayon::current_num_threads()))
                    .zip(self.bits.partial.par_iter_mut())
                    .for_each(|(rs, bs)| {
                        for (idx, x) in rs {
                            let s = (x ^ self.nimbers.g[n - i - idx]) as usize;
                            bs.set_bit(s);
                        }
                    });

                for c in self.bits.partial.iter() {
                    self.bits.seen.set_all_bits_from(&c);
                }
            }
        }
    }

    fn iterate_over_r_xor_c(&mut self, n: usize) {
        // iterate over x ^ y such that x is in R
        for i in 1..self.rules.len() {
            if self.rules[i].divide {
                let mut m = self.nimbers.rare.len();
                while m > 0 && n <= i + self.nimbers.rare[m - 1].0 {
                    m -= 1;
                }
                for (idx, x) in self.nimbers.rare[0..m].iter() {
                    let s = (x ^ self.nimbers.g[n - i - idx]) as usize;
                    self.bits.seen.set_bit(s);
                }
            }
        }
    }

    fn iterate_over_r_xor_c_back(&mut self, n: usize) {
        // iterate over x ^ y such that x is in R
        for i in 1..self.rules.len() {
            if self.rules[i].divide {
                // we assume that nimbers.rare's last value cannot exceed n - i
                for (idx, x) in self.nimbers.rare.iter() {
                    let s = (x ^ self.nimbers.last(n - i - idx)) as usize;
                    self.bits.seen.set_bit(s);
                }
            }
        }
    }

    fn set_0th_bit_if_can_be_divided_in_half(&mut self, n: usize) {
        // set an obvious 0, if the game has a dividing move to any pair (x, x)
        for i in 1..self.rules.len() {
            if self.rules[i].divide && (n - i) & 1 == 0 {
                self.bits.seen.set_bit(0);
                break;
            }
        }
    }

    pub fn check_period(&self, n: usize) -> bool {
        for period in 1..n {
            let mut start = n - period;
            while start > 0 && self.nimbers.g[start - 1] == self.nimbers.g[start - 1 + period] {
                start -= 1;
            }

            if n >= 2 * start + 2 * period + self.rules.len() - 1 {
                println!("period start: {}\n", start);
                println!("period: {}\n", period);
                return true;
            }
        }
        return false;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use phf::phf_map;

    #[test]
    fn test_game_to_rules() {
        let rules = rules_from_str("0.034");

        assert_eq!(
            rules,
            vec![
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: true,
                    some: true,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: true,
                },
            ]
        );

        let rules = rules_from_str("0.012345670");

        assert_eq!(
            rules,
            vec![
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: true,
                    some: false,
                    divide: false
                },
                Rule {
                    all: false,
                    some: true,
                    divide: false,
                },
                Rule {
                    all: true,
                    some: true,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: true,
                },
                Rule {
                    all: true,
                    some: false,
                    divide: true,
                },
                Rule {
                    all: false,
                    some: true,
                    divide: true,
                },
                Rule {
                    all: true,
                    some: true,
                    divide: true,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
            ]
        );
    }

    /// initial values taken from Achim Flammenkamp webpage:
    /// http://wwwhomes.uni-bielefeld.de/achim/octal.html
    static GAMES_NIMBERS: phf::Map<&'static str, [Nimber; 16]> = phf_map! {
        "0.004" =>  [0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 0, 3, 3, 3],
        "0.005" =>  [0, 0, 0, 1, 0, 1, 1, 2, 2, 2, 0, 3, 3, 4, 1, 1],
        "0.006" =>  [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 0, 3, 3, 1, 1, 1],
        "0.007" =>  [0, 0, 0, 1, 1, 1, 2, 2, 0, 3, 3, 1, 1, 1, 0, 4],
        "0.014" =>  [0, 0, 1, 0, 0, 1, 0, 1, 2, 2, 1, 2, 3, 4, 0, 1],
        "0.015" =>  [0, 0, 1, 1, 0, 1, 0, 2, 1, 2, 2, 3, 0, 1, 4, 2],
        "0.016" =>  [0, 0, 1, 0, 1, 2, 2, 2, 0, 1, 0, 1, 4, 4, 2, 2],
        "0.024" =>  [0, 0, 0, 1, 1, 2, 2, 3, 0, 4, 1, 1, 2, 5, 3, 2],
        "0.026" =>  [0, 0, 0, 1, 1, 2, 2, 3, 0, 4, 1, 1, 2, 5, 3, 3],
        "0.034" =>  [0, 0, 1, 1, 0, 2, 2, 3, 1, 4, 0, 1, 4, 3, 1, 2],
        "0.04" =>  [0, 0, 0, 0, 1, 1, 1, 2, 2, 0, 3, 3, 1, 1, 1, 0],
        "0.054" =>  [0, 0, 1, 0, 1, 2, 2, 2, 3, 4, 4, 1, 1, 1, 6, 3],
        "0.055" =>  [0, 0, 1, 1, 1, 2, 2, 2, 3, 1, 1, 1, 4, 4, 4, 3],
        "0.06" =>  [0, 0, 0, 1, 1, 2, 2, 0, 3, 1, 1, 2, 2, 3, 3, 4],
        "0.064" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 5, 3, 3],
        "0.104" =>  [0, 1, 0, 0, 0, 1, 0, 2, 2, 1, 2, 2, 4, 1, 0, 4],
        "0.106" =>  [0, 1, 0, 0, 0, 1, 2, 2, 2, 1, 4, 4, 0, 1, 0, 6],
        "0.114" =>  [0, 1, 1, 0, 0, 1, 1, 2, 0, 2, 1, 2, 0, 4, 1, 1],
        "0.125" =>  [0, 1, 0, 2, 1, 1, 0, 2, 1, 3, 0, 1, 1, 3, 0, 2],
        "0.126" =>  [0, 1, 0, 0, 2, 1, 3, 3, 2, 1, 0, 4, 2, 5, 0, 3],
        "0.127" =>  [0, 1, 0, 2, 2, 1, 0, 4, 4, 1, 2, 2, 0, 1, 4, 4],
        "0.135" =>  [0, 1, 1, 2, 0, 1, 1, 2, 0, 3, 1, 1, 0, 3, 1, 2],
        "0.136" =>  [0, 1, 1, 0, 0, 2, 1, 3, 0, 2, 1, 1, 0, 2, 2, 3],
        "0.14" =>  [0, 1, 0, 0, 1, 0, 2, 1, 2, 2, 1, 0, 4, 1, 4, 4],
        "0.142" =>  [0, 1, 0, 0, 2, 2, 2, 1, 1, 0, 3, 3, 2, 4, 1, 0],
        "0.143" =>  [0, 1, 0, 1, 2, 2, 2, 0, 1, 0, 4, 2, 2, 1, 5, 0],
        "0.146" =>  [0, 1, 0, 0, 2, 2, 2, 4, 1, 1, 1, 3, 3, 2, 4, 4],
        "0.156" =>  [0, 1, 1, 0, 2, 2, 2, 4, 4, 1, 1, 1, 3, 2, 2, 4],
        "0.16" =>  [0, 1, 0, 0, 1, 2, 2, 1, 4, 0, 1, 4, 2, 1, 4, 0],
        "0.161" =>  [0, 1, 0, 2, 1, 0, 2, 1, 3, 2, 1, 3, 2, 4, 3, 0],
        "0.162" =>  [0, 1, 0, 0, 2, 2, 3, 1, 1, 0, 4, 2, 2, 6, 1, 0],
        "0.163" =>  [0, 1, 0, 2, 2, 3, 1, 0, 4, 2, 2, 6, 1, 0, 4, 2],
        "0.164" =>  [0, 1, 0, 0, 1, 2, 2, 3, 4, 4, 5, 1, 1, 6, 3, 2],
        "0.165" =>  [0, 1, 0, 2, 1, 3, 2, 1, 3, 4, 4, 3, 6, 2, 3, 1],
        "0.166" =>  [0, 1, 0, 0, 2, 2, 3, 4, 1, 1, 6, 6, 2, 2, 4, 4],
        "0.167" =>  [0, 1, 0, 2, 2, 3, 4, 1, 1, 6, 2, 2, 4, 4, 1, 1],
        "0.172" =>  [0, 1, 1, 0, 2, 2, 3, 0, 1, 1, 3, 2, 2, 4, 4, 0],
        "0.174" =>  [0, 1, 1, 0, 2, 1, 3, 2, 2, 1, 4, 4, 5, 6, 4, 2],
        "0.204" =>  [0, 0, 1, 0, 1, 2, 0, 1, 0, 1, 2, 3, 1, 2, 1, 2],
        "0.205" =>  [0, 0, 1, 2, 0, 1, 0, 1, 2, 3, 1, 2, 3, 1, 3, 4],
        "0.206" =>  [0, 0, 1, 0, 1, 2, 3, 2, 0, 1, 0, 1, 2, 3, 2, 3],
        "0.207" =>  [0, 0, 1, 2, 1, 2, 0, 3, 0, 1, 2, 4, 5, 3, 1, 2],
        "0.224" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 1, 4, 3, 0, 4],
        "0.244" =>  [0, 0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 1, 5, 6, 7, 3],
        "0.245" =>  [0, 0, 1, 2, 1, 2, 3, 4, 5, 1, 5, 6, 7, 3, 2, 1],
        "0.264" =>  [0, 0, 1, 2, 3, 4, 5, 1, 6, 3, 2, 5, 1, 8, 6, 7],
        "0.314" =>  [0, 1, 2, 0, 1, 2, 0, 2, 1, 2, 3, 1, 2, 4, 5, 3],
        "0.324" =>  [0, 1, 0, 2, 1, 3, 0, 1, 3, 4, 0, 2, 3, 4, 2, 1],
        "0.334" =>  [0, 1, 2, 0, 1, 2, 0, 3, 1, 2, 3, 1, 2, 4, 3, 5],
        "0.336" =>  [0, 1, 2, 0, 3, 1, 2, 4, 0, 3, 1, 2, 0, 3, 4, 1],
        "0.342" =>  [0, 1, 0, 1, 2, 3, 2, 0, 1, 0, 3, 2, 3, 4, 5, 0],
        "0.344" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 4, 6, 2, 3, 2, 1],
        "0.346" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 6, 7, 2, 3, 2, 1],
        "0.354" =>  [0, 1, 2, 0, 1, 2, 4, 3, 1, 2, 3, 5, 2, 4, 3, 5],
        "0.356" =>  [0, 1, 2, 0, 2, 1, 2, 4, 5, 1, 6, 7, 5, 1, 2, 8],
        "0.36" =>  [0, 1, 0, 2, 1, 0, 2, 1, 3, 2, 1, 3, 2, 4, 3, 0],
        "0.362" =>  [0, 1, 0, 2, 3, 4, 1, 0, 2, 3, 4, 1, 5, 2, 3, 7],
        "0.364" =>  [0, 1, 0, 2, 1, 3, 2, 1, 3, 4, 5, 3, 4, 2, 3, 1],
        "0.366" =>  [0, 1, 0, 2, 3, 4, 5, 1, 6, 2, 3, 4, 5, 7, 6, 8],
        "0.37" =>  [0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 0, 3, 4, 2, 1],
        "0.371" =>  [0, 1, 2, 3, 1, 0, 3, 2, 4, 0, 2, 3, 4, 0, 1, 2],
        "0.374" =>  [0, 1, 2, 0, 1, 2, 4, 3, 1, 2, 3, 5, 2, 4, 3, 5],
        "0.376" =>  [0, 1, 2, 0, 3, 1, 2, 4, 3, 5, 2, 4, 3, 5, 1, 4],
        "0.404" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 6, 3, 3],
        "0.414" =>  [0, 0, 1, 1, 0, 2, 2, 3, 4, 4, 0, 1, 1, 3, 2, 2],
        "0.416" =>  [0, 0, 1, 1, 2, 2, 3, 4, 1, 1, 6, 6, 3, 2, 2, 1],
        "0.444" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 6, 3, 3],
        "0.45" =>  [0, 0, 1, 1, 2, 2, 3, 1, 1, 4, 4, 3, 2, 2, 1, 1],
        "0.454" =>  [0, 0, 1, 1, 2, 2, 3, 4, 1, 1, 6, 6, 3, 2, 2, 1],
        "0.56" =>  [0, 1, 0, 2, 2, 4, 1, 1, 3, 2, 4, 4, 6, 6, 2, 1],
        "0.564" =>  [0, 1, 0, 2, 2, 4, 4, 1, 1, 3, 2, 5, 4, 7, 6, 8],
        "0.6" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 0, 3, 4, 2],
        "0.604" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 5, 3, 4, 5],
        "0.606" =>  [0, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 5, 1, 2, 3, 4],
        "0.64" =>  [0, 0, 1, 2, 3, 4, 1, 5, 3, 2, 1, 5, 4, 2, 6, 8],
        "0.644" =>  [0, 0, 1, 2, 3, 4, 5, 1, 6, 3, 2, 5, 8, 9, 6, 10],
        "0.74" =>  [0, 1, 0, 1, 2, 3, 2, 4, 1, 4, 6, 2, 3, 2, 1, 5],
        "0.744" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 6, 7, 2, 3, 2, 1],
        "0.76" =>  [0, 1, 0, 2, 3, 4, 1, 6, 2, 3, 4, 1, 6, 7, 3, 2],
        "0.764" =>  [0, 1, 0, 2, 3, 4, 5, 1, 6, 2, 3, 4, 5, 7, 6, 8],
        "0.774" =>  [0, 1, 2, 3, 1, 4, 5, 6, 7, 1, 3, 2, 8, 9, 5, 4],
        "0.776" =>  [0, 1, 2, 3, 4, 1, 6, 3, 2, 1, 6, 7, 4, 5, 8, 1],
    };

    #[test]
    fn test_initialize() {
        for (rules_str, res) in GAMES_NIMBERS.into_iter() {
            let initial_len = rules_str.len() - 1; // -1 for '.'

            let mut g = Game::new(rules_str, initial_len, 0);
            g.init();

            assert_eq!(g.nimbers.g, res[0..initial_len]);
        }
    }

    #[test]
    fn test_naive() {
        for (rules_str, res) in GAMES_NIMBERS.into_iter() {
            let max = 16;
            let mut g = Game::new(rules_str, max, 0);
            g.init();

            for n in g.rules.len()..max {
                let nim = g.naive(n);
                g.set_next_g_n(n, nim);
            }

            assert_eq!(g.nimbers.g, res);
        }
    }

    #[test]
    #[ignore]
    fn test_rc_with_naive() {
        for (rules_str, _res) in GAMES_NIMBERS.into_iter() {
            println!("{}", rules_str);
            let max = 10000;
            let mut g = Game::new(rules_str, max, 0);
            g.init();

            for n in g.rules.len()..max {
                let nim_naive = g.naive(n);
                let nim_rc = g.rc(n);
                g.set_next_g_n(n, nim_rc);
                assert_eq!(nim_naive, nim_rc, " for game {} at {}", rules_str, n);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_rares() {
        for (rules_str, _res) in GAMES_NIMBERS.into_iter() {
            println!("{}", rules_str);
            let max = 10000;
            let mut g = Game::new(rules_str, max, 0);
            g.init();

            for n in g.rules.len()..max {
                let nim_rc = g.rc(n);
                g.set_next_g_n(n, nim_rc);
            }

            for x in 0..(g.stats.largest_nimber + 1).next_power_of_two() {
                for y in 0..(g.stats.largest_nimber + 1).next_power_of_two() {
                    let x_rare = g.bits.rare.get(x as usize);
                    let y_rare = g.bits.rare.get(y as usize);

                    let loc = (x ^ y) as usize;
                    if x_rare && y_rare {
                        assert!(g.bits.rare.get(loc));
                    }

                    if !x_rare && !y_rare {
                        assert!(g.bits.rare.get(loc));
                    }

                    if x_rare && !y_rare {
                        assert!(!g.bits.rare.get(loc));
                    }
                    if !x_rare && y_rare {
                        assert!(!g.bits.rare.get(loc));
                    }
                }
            }
        }
    }
}
