// use game::gen_rares;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

pub mod octal;

fn save(n: usize, rules_str: &str, buf: &[u8]) {
    let path = format!("nimbers_{}_{}", rules_str, n);

    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap()
        .write(buf)
        .unwrap();
}

fn load(max: usize, path: &Path) -> Vec<octal::Nimber> {
    println!("Reading nimbers from {:?}", path);

    let nimber_bytes = (octal::Nimber::BITS / u8::BITS) as usize;
    let mut nimbers = vec![0 as octal::Nimber; max];
    let mut buf: Vec<u8> = Vec::with_capacity(max * nimber_bytes);

    fs::OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();

    for i in 0..max {
        let mut n: octal::Nimber = 0;
        for b in 0..nimber_bytes {
            let loc = (i * nimber_bytes) + (nimber_bytes - b) - 1;
            n += (buf[loc] as octal::Nimber) << (b * nimber_bytes);
        }
        nimbers[i] = n;
    }
    nimbers
}

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
        let mut last = 0;

        for i in (max_full_memory..).step_by(max_tail_memory) {
            let p = format!("nimbers_{}_{}", rules_str, i);
            let path = Path::new(&p);

            if path.exists() {
                last = i;
            } else {
                break;
            }
        }

        if last == 0 {
            last = max_full_memory;
            g.nimbers.copy_to_g_back();
        } else {
            let p = format!("nimbers_{}_{}", rules_str, last);
            let path = Path::new(&p);
            let loaded = load(max_tail_memory, path);
            g.nimbers.g_back = loaded;
        }

        let nimber_bytes = (octal::Nimber::BITS / u8::BITS) as usize;
        let mut buf: Vec<u8> = vec![0; max_tail_memory * nimber_bytes];

        for n in last.. {
            if n % max_tail_memory == 0 {
                for i in 0..max_tail_memory {
                    let nim = g.nimbers.g_back[i];
                    for b in 0..nimber_bytes {
                        let loc = (i * nimber_bytes) + (nimber_bytes - b) - 1;
                        buf[loc] = (nim >> b * nimber_bytes) as u8;
                    }
                }
                save(n, rules_str, &buf)
            }
            g.calc_rc_back(n);
            g.occasional_info_back(n, &start);
        }
    }
}
