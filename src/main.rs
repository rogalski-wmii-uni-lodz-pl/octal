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

    for n in first_uninitialized..max {
        g[n] = game::naive(&rules, &g, n, &mut seen);
        if g[n] > largest {
            largest = g[n];
            seen = game::make_bitset(largest);
        }
        seen.set_elements(0);

        println!("G({}) = {}", n, g[n]);
    }
}
