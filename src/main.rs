use game::gen_rares;

pub mod game;

fn main() {
    let rules_str = "0.034";
    let rules = game::rules_from_str(rules_str);

    let max = 1000_000;
    let mut g = vec![game::UNSET; max];

    game::initialize(&rules, &mut g);

    let first_uninitialized = rules.len();

    let mut largest = 2;

    for n in 1..first_uninitialized {
        largest = std::cmp::max(g[n], largest)
    }
    let mut freq = vec![0 as usize ; (largest + 2).next_power_of_two() - 1];
    for n in 1..first_uninitialized {
        freq[g[n]] += 1;
    }

    let mut seen = game::make_bitset(largest);

    let mut rares = gen_rares(&freq);

    let mut rare_idx_and_nimber = vec![];

    for i in 1..first_uninitialized {
        let x = g[i];
        if rares[x] {
            rare_idx_and_nimber.push((i, x));
        }
    }

    for n in first_uninitialized..max {
            dbg!(&seen.len());
            dbg!(&rares.len());
        let na = game::naive(&rules, &g, n, &mut seen);
        seen.set_elements(0);
        let rc = game::rc(&rules, &g, n, &mut seen, &rares, &rare_idx_and_nimber);

        g[n] = na;
        if g[n] >= freq.len() {
            freq.resize((g[n] + 2).next_power_of_two() - 1, 0);
        }
        freq[g[n]] += 1;

        if rares[g[n]] {
            rare_idx_and_nimber.push((n, g[n]));
        }

        if g[n] > largest {
            largest = g[n];
            seen = game::make_bitset(largest);
            rares = game::gen_rares(&freq);
            rare_idx_and_nimber.clear();
            for i in 1..=n {
                if rares[g[i]] {
                    rare_idx_and_nimber.push((i, g[i]));
                }
            }
            // rares.set(0, false);
        }
        seen.set_elements(0);

        println!("G({}) = {}, {}", n, na, rc);
        assert_eq!(na, rc);
    }
}
