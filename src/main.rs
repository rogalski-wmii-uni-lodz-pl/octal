use game::gen_rares;
use std::env;
use serde::{Serialize, Deserialize};
use std::time::Instant;
use bitvec::prelude::*;

pub mod game;

#[derive(Serialize, Deserialize)]
struct Freq {
    nimber: usize,
    frequency: usize,
    rare: bool,
}

fn dump_freqs(freqs : &Vec<usize>, rares: &BitVec<u64, Msb0>) {
    let fs : Vec<Freq>= freqs.iter().enumerate().map(|(nimber, &frequency)| Freq { nimber, frequency, rare : rares[nimber] }).collect();
    let res = serde_json::to_string_pretty(&fs);
    println!("{}", res.unwrap());
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let rules_str = if args.len() > 1 { &args[1] } else { "0.034" };

    let rules = game::rules_from_str(rules_str);

    let max = if args.len() > 2 {
        args[2].parse::<usize>().unwrap()
    } else {
        1_000_000
    };
    let mut g = vec![game::UNSET; max];

    game::initialize(&rules, &mut g);

    let first_uninitialized = rules.len();

    let mut largest = *g[1..first_uninitialized].iter().max().unwrap_or(&2);

    let mut freq = vec![0 as usize; (largest + 2).next_power_of_two() - 1];
    for n in 1..first_uninitialized {
        freq[g[n]] += 1;
    }

    let mut seen = game::make_bitset(largest);

    let mut rares = gen_rares(&freq, largest);

    let mut rare_idx_and_nimber = vec![];

    for i in 1..first_uninitialized {
        let x = g[i];
        if rares[x] {
            rare_idx_and_nimber.push((i, x));
        }
    }

    let now = Instant::now();

    let mut maxs = 0;

    for n in first_uninitialized..max {
        let (s, x) = game::rc(&rules, &g, n, &mut seen, &rares, &rare_idx_and_nimber);
        g[n] = x;

        maxs = std::cmp::max(maxs, s);
        // g[n] = game::naive(&rules, &g, n, &mut seen);

        if g[n] >= freq.len() {
            freq.resize((g[n] + 2).next_power_of_two() - 1, 0);
        }
        freq[g[n]] += 1;

        if rares[g[n]] {
            rare_idx_and_nimber.push((n, g[n]));
        }
        if n % 10000 == 0 {
            println!("G({}) = {}, {:?}, {}", n, g[n], now.elapsed(), maxs);
        }


        if g[n] > largest {
            largest = g[n];
            seen = game::make_bitset(largest);
            rares = game::gen_rares(&freq, largest);
            // dump_freqs(&freq, &rares);
            rare_idx_and_nimber.clear();
            for i in 1..=n {
                if rares[g[i]] {
                    rare_idx_and_nimber.push((i, g[i]));
                }
            }
        } else if n.is_power_of_two() {
            rares = game::gen_rares(&freq, largest);
            rare_idx_and_nimber.clear();
            println!("{} freqs after {:?}", n, now.elapsed());
            dump_freqs(&freq, &rares);
            for i in 1..=n {
                if rares[g[i]] {
                    rare_idx_and_nimber.push((i, g[i]));
                }
            }
        }
        seen.set_elements(0);
    }
    println!("{} freqs after {:?}", max, now.elapsed());
    dump_freqs(&freq, &rares);
}
