use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

pub mod octal;

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
            n += (buf[loc] as octal::Nimber) << (b * 8);
        }
        nimbers[i] = n;
    }
    nimbers
}

struct Mmap {
    buf: Vec<octal::Nimber>,
    end: usize,
    path: String,
}

impl Mmap {
    fn new(path: String, max_tail_memory: usize, end: usize) -> Self {
        let mut p = path.clone();
        p.push_str(&end.to_string());
        Self {
            path,
            end,
            buf: load(max_tail_memory, &Path::new(&p)),
        }
    }

    fn at(&mut self, i: usize) -> octal::Nimber {
        let len = self.buf.len();
        let begin = self.end - len;
        let outside_current_buf = i < begin || self.end <= i;
        if outside_current_buf {
            self.end = ((i / len) + 1) * len;
            let mut p = self.path.clone();
            p.push_str(&self.end.to_string());
            self.buf = load(len, &Path::new(&p))
        }

        return self.buf[i % len];
    }
}

struct Mem {
    left: Mmap,
    right: Mmap,
}

impl Mem {
    fn new(path: String, max_tail_memory: usize, end: usize) -> Self {
        Self {
            left: Mmap::new(path.to_owned(), max_tail_memory, end),
            right: Mmap::new(path, max_tail_memory, end),
        }
    }
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
        max_full_memory
    };

    let dir = if args.len() > 4 {
        args[4].clone()
    } else {
        String::from(".")
    };

    // let start = Instant::now();

    // println!("total period: {:?}", start_period.elapsed());
    // println!("total: {:?}", start.elapsed());

    let mut last = 0;
    let mut hm = vec![];

    // let mut total: usize = 0;

    let path = format!("{}/nimbers_{}_", dir, rules_str);


    let achim: Vec<usize> = if args.len() > 5 {
        args[5..]
            .iter()
            .map(|x| x.parse::<usize>().unwrap())
            .collect()
    } else {
        vec![]
    };

    for i in (max_full_memory..).step_by(max_tail_memory) {
        let p = path.to_owned() + &i.to_string();
        let pth = Path::new(&p);

        if pth.exists() {
            last = i;
        } else {
            break;
        }
    }

    let mut mem = Mem::new(path, max_tail_memory, max_tail_memory);

    for i in 1..last {
        let n = mem.right.at(i) as usize;

        if n >= hm.len() {
            hm.resize(n + 1, 0 as usize);
        }

        hm[n] += 1;
        if i.is_power_of_two() || achim.contains(&i) {
            println!("{} {}", i, (i as f64).log2());
            for (n, cnt) in hm.iter().enumerate() {
                println!("{} {}", n, cnt);
            }
            println!("{} {}", i, (i as f64).log2());
            println!("");
        }
    }


    let mut found = false;
    let mut longest = 0;

    for period in 1..=(last / 2) {
        if period % mem.right.buf.len() == 0 {
            println!("{}", period);
        }
        let mut start = last - period;

        while start > 0 && mem.left.at(start - 1) == mem.right.at(start - 1 + period) {
            start -= 1;
        }

        longest = std::cmp::max(longest, last-period-start);

        if last >= 2 * start + 2 * period + rules_str.len() - 1 {
            println!("period start: {}\n", start);
            println!("period: {}\n", period);
            found = true;
            break;
        }
    }

    if !found {
        println!("no period :(");
        println!("longest streak: {}", longest);
    }
}
