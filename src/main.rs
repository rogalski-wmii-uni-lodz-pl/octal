// use game::gen_rares;
use std::env;
use std::time::Instant;

pub mod octal;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rules_str = if args.len() > 1 { &args[1] } else { "0.034" };

    let max_full_memory = if args.len() > 2 {
        args[2].parse::<usize>().unwrap()
    } else {
        1_000_000
    };

    let max_tail_memory = if args.len() > 3 {
        args[3].parse::<usize>().unwrap()
    } else {
        0
    };

    let start = Instant::now();

    println!(
        "nimber bitsize {}, maxval {}",
        octal::Nimber::BITS,
        octal::Nimber::MAX
    );

    let mut g = octal::Game::new(rules_str, max_full_memory, max_tail_memory);
    g.init();
    for n in g.rules.len()..max_full_memory {
        g.calc_rc(n);
        g.occasional_info(n, &start);
    }
    g.dump_freqs(max_full_memory, &start);
    g.dump_stats(max_full_memory - 1, &start);
    let start_period = Instant::now();

    let period_found = g.check_period(max_full_memory);
    println!("total period: {:?}", start_period.elapsed());
    println!("total: {:?}", start.elapsed());

    if !period_found && max_tail_memory != 0 {
        g.nimbers.copy_to_g_back();

        for n in max_full_memory.. {
            g.calc_rc2(n);
            g.occasional_info2(n, &start);
        }
    }
}
