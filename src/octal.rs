use super::game;
use bitvec::prelude::*;
use std::{fmt::DebugMap, time::Instant};
// use std::cmp::Reverse;
// use std::collections::{HashSet, VecDeque};

type Nimber = usize;

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

type BitV = BitVec<u64, Msb0>;

pub struct Bits {
    pub rare: BitV,
    pub seen: BitV,
}

impl Bits {
    pub fn new() -> Self {
        Self {
            rare: game::make_bitset(0),
            seen: game::make_bitset(0),
        }
    }

    pub fn resize(&mut self, largest_nimber: Nimber) {
        self.rare = game::make_bitset(largest_nimber);
        self.seen = game::make_bitset(largest_nimber);
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
            .resize((self.largest_nimber + 2).next_power_of_two() - 1, 0);
    }
    pub fn set_largest_nimber(&mut self, g: &Vec<Nimber>, first_uninitialized: usize) {
        self.largest_nimber = *g[1..first_uninitialized].iter().max().unwrap_or(&2);
    }

    pub fn initialize_frequencies(&mut self, g: &Vec<Nimber>, first_uninitialized: usize) {
        for n in 1..first_uninitialized {
            self.frequencies[g[n]] += 1;
        }
    }

    pub fn gen_rares(&self) -> BitV {
        game::gen_rares(&self.frequencies, self.largest_nimber)
    }
}

pub struct Game {
    pub rules: Vec<game::Rule>,
    pub nimbers: Nimbers,
    pub stats: Stats,
    pub bits: Bits,
}

impl Game {
    pub fn new(rules_str: &str, starting_size: usize) -> Self {
        Game {
            rules: game::rules_from_str(rules_str),
            nimbers: Nimbers::new(starting_size),
            stats: Stats::new(),
            bits: Bits::new(),
        }
    }

    pub fn init(&mut self) {
        game::initialize(&self.rules, &mut self.nimbers.g);
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

    pub fn rc(&mut self, n: usize) -> Nimber {
        self.bits.seen.set_elements(0);

        self.set_seen_bits_from_some_moves(n);
        self.set_0th_bit_if_can_be_divided_in_half(n);
        self.iterate_over_r_xor_c(n);

        self.prove(n)
    }

    pub fn calc(&mut self, n: usize, start: &Instant) {
        let nim = self.rc(n);
        self.nimbers.g[n] = nim;

        if nim > self.stats.largest_nimber {
            self.stats.largest_nimber = nim;

            self.resize(n);
        }

        self.stats.frequencies[nim] += 1;

        if n < self.nimbers.g.len() && self.bits.rare[nim] {
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
        game::dump_freqs(&self.stats.frequencies, &self.bits.rare);
    }

    fn resize(&mut self, n: usize) {
        self.stats.resize_frequencies();
        self.bits.resize(self.stats.largest_nimber);
        self.bits.rare = self.stats.gen_rares();
        self.nimbers.rare.clear();
        for i in 1..n {
            if self.bits.rare[self.nimbers.g[i]] {
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
                            return first_common;
                            // break
                        }
                    }
                }
            }
        }

        self.stats.largest_index = std::cmp::max(self.stats.largest_index, n);
        mex.first_zero().unwrap()
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
}
