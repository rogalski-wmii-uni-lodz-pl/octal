use game::gen_rares;
use std::env;
use std::time::Instant;

pub mod game;

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

    for n in first_uninitialized..max {
        g[n] = game::rc(&rules, &g, n, &mut seen, &rares, &rare_idx_and_nimber);
        // g[n] = game::naive(&rules, &g, n, &mut seen);

        if g[n] >= freq.len() {
            freq.resize((g[n] + 2).next_power_of_two() - 1, 0);
        }
        freq[g[n]] += 1;

        if rares[g[n]] {
            rare_idx_and_nimber.push((n, g[n]));
        }
        if n % 10000 == 0 {
            println!("G({}) = {}, {:?}", n, g[n], now.elapsed());
        }

        if g[n] > largest {
            largest = g[n];
            seen = game::make_bitset(largest);
            rares = game::gen_rares(&freq, largest);
            rare_idx_and_nimber.clear();
            for i in 1..=n {
                if rares[g[i]] {
                    rare_idx_and_nimber.push((i, g[i]));
                }
            }
        }
        seen.set_elements(0);
    }
}
