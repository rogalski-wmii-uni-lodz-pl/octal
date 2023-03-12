// use game::gen_rares;
use std::env;
use std::time::Instant;

pub mod octal;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rules_str = if args.len() > 1 { &args[1] } else { "0.034" };

    let max = if args.len() > 2 {
        args[2].parse::<usize>().unwrap()
    } else {
        1_000_000
    };
    let start = Instant::now();

    println!(
        "nimber bitsize {}, maxval {}",
        octal::Nimber::BITS,
        octal::Nimber::MAX
    );

    let mut g = octal::Game::<octal::BitV>::new(rules_str, max);
    g.init();
    for n in g.rules.len()..max {
        g.calc(n, &start);
    }
    g.dump_freqs(max, &start);
    g.check_period(max);
}
