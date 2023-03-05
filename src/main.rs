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

    let mut seen = game::make_bitset(largest);
    let mut rares = game::make_bitset(largest);

    let mut rare_idx_and_nimber = vec![];

    for (i, &x) in g[1..first_uninitialized].iter().enumerate() {
        if x != 0 {
            rare_idx_and_nimber.push((i + 1, x));
            rares.set(x, true);
        }
    }

    for n in first_uninitialized..max {
        let na = game::naive(&rules, &g, n, &mut seen);
        seen.set_elements(0);
        let rc = game::rc(&rules, &g, n, &mut seen, &rares, &rare_idx_and_nimber);

        g[n] = na;

        if g[n] != 0 {
            rare_idx_and_nimber.push((n, g[n]));
        }
        if g[n] > largest {
            largest = g[n];
            seen = game::make_bitset(largest);
            rares = game::make_bitset(largest);
            for i in 1..=n {
                rares.set(g[i], true);
            }
            rares.set(0, false);
        }
        seen.set_elements(0);

        println!("G({}) = {}, {}", n, na, rc);
        assert_eq!(na, rc);
    }
}
