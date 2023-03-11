// use super::game;
use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::time::Instant;
use cfg_if;

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

type BitV = BitVec<u64, Msb0>;

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

/// front - nimbers from 0 to some unspecified value (enough to calculate the next nimber)
/// back - nimbers from n to at lest n - front.len(), (again, enough to calculate the next nimber)
pub struct Nimbers {
    g: Vec<Nimber>,
    rare: Vec<(usize, Nimber)>,
}

impl Nimbers {
    pub fn new(max: usize) -> Self {
        Self {
            g: vec![Nimber::max_value(); max],
            rare: vec![],
        }
    }
}

pub struct Stats {
    pub largest_nimber: Nimber,
    pub frequencies: Vec<usize>,
    pub largest_index: usize,
}

pub struct Bits {
    pub rare: BitV,
    pub seen: BitV,
}


pub fn make_bitset(largest: Nimber) -> BitV {
    let bits = 2 * (largest as usize).next_power_of_two() + 2;
    bitvec!(u64, Msb0; 0; bits)
}


impl Bits {
    pub fn new() -> Self {
        Self {
            rare: make_bitset(0),
            seen: make_bitset(0),
        }
    }

    pub fn resize(&mut self, largest_nimber: Nimber) {
        self.rare = make_bitset(largest_nimber);
        self.seen = make_bitset(largest_nimber);
    }
}

impl Stats {
    pub fn new() -> Self {
        Self {
            largest_nimber: Nimber::min_value(),
            frequencies: vec![],
            largest_index: 0,
        }
    }

    pub fn initialize(&mut self, front: &Vec<Nimber>, first_uninitialized: usize) {
        self.set_largest_nimber(front, first_uninitialized);
        self.resize_frequencies();
        self.initialize_frequencies(front, first_uninitialized);
    }

    pub fn resize_frequencies(&mut self) {
        self.frequencies
            .resize((self.largest_nimber as usize + 2).next_power_of_two() - 1, 0);
    }
    pub fn set_largest_nimber(&mut self, g: &Vec<Nimber>, first_uninitialized: usize) {
        self.largest_nimber = *g[1..first_uninitialized].iter().max().unwrap_or(&2);
    }

    pub fn initialize_frequencies(&mut self, g: &Vec<Nimber>, first_uninitialized: usize) {
        for n in 1..first_uninitialized {
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
    pub fn gen_rares(&self) -> BitV {
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

        let mut rares = make_bitset(self.largest_nimber);
        for &x in r.iter() {
            rares.set(x, true);
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
    pub fn new(rules_str: &str, starting_size: usize) -> Self {
        Game {
            rules: rules_from_str(rules_str),
            nimbers: Nimbers::new(starting_size),
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

            seen.set(0, n < self.rules.len() && self.rules[n].all);

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
        self.resize(first_uninitialized);
    }

    pub fn set_seen_bits_from_some_moves(&mut self, n: usize) {
        // set the non-xor values
        for i in 1..self.rules.len() {
            if self.rules[i].some {
                self.bits.seen.set(self.nimbers.g[n - i] as usize, true);
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
        self.bits.seen.set_elements(0);

        self.set_seen_bits_from_some_moves(n);
        self.set_0th_bit_if_can_be_divided_in_half(n);
        self.iterate_over_r_xor_c(n);

        self.prove(n)
    }

    /// Naively compute the nimber g[n] assuming g[0..n] were computed correctly, accodring to the
    /// rules of some octal game, assuming that at n is at least rules.len().
    ///
    /// Assumption that n is at least rules.len() makes it possible to omit some checks (for instance,
    /// there are no more whole moves possible, and some and divide rules are always applicable, since
    /// n is greater than rules.len(0).
    pub fn naive(&mut self, n: usize) -> usize {
        assert!(n >= self.rules.len());
        self.bits.seen.set_elements(0);

        for i in 1..self.rules.len() {
            if self.rules[i].some {
                self.bits.seen.set(self.nimbers.g[n - i] as usize, true);
            }

            if self.rules[i].divide {
                for j in 1..=(n - i) / 2 {
                    let x = self.nimbers.g[j];
                    let y = self.nimbers.g[n - i - j];
                    self.bits.seen.set((x ^ y) as usize, true);
                }
            }
        }

        self.bits.seen.first_zero().unwrap()
    }

    pub fn calc(&mut self, n: usize, start: &Instant) {
        let nim = self.rc(n);
        self.nimbers.g[n] = nim;

        if nim > self.stats.largest_nimber {
            self.stats.largest_nimber = nim;

            self.resize(n);
        }

        self.stats.frequencies[nim as usize] += 1;

        if n < self.nimbers.g.len() && self.bits.rare[nim as usize] {
            self.nimbers.rare.push((n, nim));
        }

        if n % 10000 == 0 {
            println!(
                "G({}) = {}, {:?}, {}",
                n,
                nim,
                start.elapsed(),
                self.stats.largest_index
            );
        }

        if n.is_power_of_two() {
            self.resize(n);
            self.dump_freqs(n, start);
        }
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
                rare: self.bits.rare[nimber],
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
        for i in 1..n {
            if self.bits.rare[self.nimbers.g[i] as usize] {
                self.nimbers.rare.push((i, self.nimbers.g[i]));
            }
        }
    }

    fn prove(&mut self, n: usize) -> Nimber {
        let first_common = self.find_first_common_unset();

        let mut mex = self.bits.seen[0..first_common + 1].to_owned();
        let mut remaining_unset = mex.count_zeros() - 1;
        // -1 for seen2[first_common]

        for i in 1..self.rules.len() {
            if remaining_unset == 0 {
                break;
            }

            if self.rules[i].divide {
                for j in 1..=(n - i) / 2 {
                    let a = self.nimbers.g[j];
                    let b = self.nimbers.g[n - i - j];
                    let loc = (a ^ b) as usize;

                    if loc < first_common && !mex[loc] {
                        // a rare value smaller than first_common and not previously observed found
                        mex.set(loc, true);
                        remaining_unset -= 1;
                        if remaining_unset == 0 {
                            // all smaller values than first_common found, the value is the smallest
                            // not observed common
                            self.stats.largest_index = std::cmp::max(self.stats.largest_index, j);
                            return first_common as Nimber;
                            // break
                        }
                    }
                }
            }
        }

        self.stats.largest_index = std::cmp::max(self.stats.largest_index, n);
        mex.first_zero().unwrap() as Nimber
    }

    // find the smallest common value
    fn find_first_common_unset(&mut self) -> usize {
        for i in 0..self.bits.seen.len() {
            if !self.bits.seen[i] && !self.bits.rare[i] {
                return i;
            }
        }

        self.bits.seen.len() - 1
    }

    fn iterate_over_r_xor_c(&mut self, n: usize) {
        // iterate over x ^ y such that x is in R
        for (idx, x) in self.nimbers.rare.iter() {
            for i in 1..self.rules.len() {
                if self.rules[i].divide {
                    if n > idx + i {
                        let s = (x ^ self.nimbers.g[n - i - idx]) as usize;
                        self.bits.seen.set(s, true);
                    }
                }
            }
        }
    }

    fn set_0th_bit_if_can_be_divided_in_half(&mut self, n: usize) {
        // set an obvious 0, if the game has a dividing move to any pair (x, x)
        for i in 1..self.rules.len() {
            if self.rules[i].divide && (n - i) & 1 == 0 {
                self.bits.seen.set(0, true);
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
