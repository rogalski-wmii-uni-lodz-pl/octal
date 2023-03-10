// use std::{collections::HashSet, cmp::Reverse};

// use bitvec::prelude::*;
// use serde::{Serialize, Deserialize};
// pub const UNSET: usize = usize::MAX;

//#[derive(Copy, Clone, Debug, PartialEq)]


#[cfg(test)]
mod test {
    // use phf::phf_map;
    // use super::*;

    // #[test]
    // fn test_game_to_rules() {
    //     let rules = rules_from_str("0.034");

    //     assert_eq!(
    //         rules,
    //         vec![
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: true,
    //                 some: true,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: true,
    //             },
    //         ]
    //     );

    //     let rules = rules_from_str("0.012345670");

    //     assert_eq!(
    //         rules,
    //         vec![
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: true,
    //                 some: false,
    //                 divide: false
    //             },
    //             Rule {
    //                 all: false,
    //                 some: true,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: true,
    //                 some: true,
    //                 divide: false,
    //             },
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: true,
    //             },
    //             Rule {
    //                 all: true,
    //                 some: false,
    //                 divide: true,
    //             },
    //             Rule {
    //                 all: false,
    //                 some: true,
    //                 divide: true,
    //             },
    //             Rule {
    //                 all: true,
    //                 some: true,
    //                 divide: true,
    //             },
    //             Rule {
    //                 all: false,
    //                 some: false,
    //                 divide: false,
    //             },
    //         ]
    //     );
    // }

    // /// initial values taken from Achim Flammenkamp webpage:
    // /// http://wwwhomes.uni-bielefeld.de/achim/octal.html
    // static GAMES_NIMBERS: phf::Map<&'static str, [usize; 16]> = phf_map! {
    //     "0.004" =>  [0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 0, 3, 3, 3],
    //     "0.005" =>  [0, 0, 0, 1, 0, 1, 1, 2, 2, 2, 0, 3, 3, 4, 1, 1],
    //     "0.006" =>  [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 0, 3, 3, 1, 1, 1],
    //     "0.007" =>  [0, 0, 0, 1, 1, 1, 2, 2, 0, 3, 3, 1, 1, 1, 0, 4],
    //     "0.014" =>  [0, 0, 1, 0, 0, 1, 0, 1, 2, 2, 1, 2, 3, 4, 0, 1],
    //     "0.015" =>  [0, 0, 1, 1, 0, 1, 0, 2, 1, 2, 2, 3, 0, 1, 4, 2],
    //     "0.016" =>  [0, 0, 1, 0, 1, 2, 2, 2, 0, 1, 0, 1, 4, 4, 2, 2],
    //     "0.024" =>  [0, 0, 0, 1, 1, 2, 2, 3, 0, 4, 1, 1, 2, 5, 3, 2],
    //     "0.026" =>  [0, 0, 0, 1, 1, 2, 2, 3, 0, 4, 1, 1, 2, 5, 3, 3],
    //     "0.034" =>  [0, 0, 1, 1, 0, 2, 2, 3, 1, 4, 0, 1, 4, 3, 1, 2],
    //     "0.04" =>  [0, 0, 0, 0, 1, 1, 1, 2, 2, 0, 3, 3, 1, 1, 1, 0],
    //     "0.054" =>  [0, 0, 1, 0, 1, 2, 2, 2, 3, 4, 4, 1, 1, 1, 6, 3],
    //     "0.055" =>  [0, 0, 1, 1, 1, 2, 2, 2, 3, 1, 1, 1, 4, 4, 4, 3],
    //     "0.06" =>  [0, 0, 0, 1, 1, 2, 2, 0, 3, 1, 1, 2, 2, 3, 3, 4],
    //     "0.064" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 5, 3, 3],
    //     "0.104" =>  [0, 1, 0, 0, 0, 1, 0, 2, 2, 1, 2, 2, 4, 1, 0, 4],
    //     "0.106" =>  [0, 1, 0, 0, 0, 1, 2, 2, 2, 1, 4, 4, 0, 1, 0, 6],
    //     "0.114" =>  [0, 1, 1, 0, 0, 1, 1, 2, 0, 2, 1, 2, 0, 4, 1, 1],
    //     "0.125" =>  [0, 1, 0, 2, 1, 1, 0, 2, 1, 3, 0, 1, 1, 3, 0, 2],
    //     "0.126" =>  [0, 1, 0, 0, 2, 1, 3, 3, 2, 1, 0, 4, 2, 5, 0, 3],
    //     "0.127" =>  [0, 1, 0, 2, 2, 1, 0, 4, 4, 1, 2, 2, 0, 1, 4, 4],
    //     "0.135" =>  [0, 1, 1, 2, 0, 1, 1, 2, 0, 3, 1, 1, 0, 3, 1, 2],
    //     "0.136" =>  [0, 1, 1, 0, 0, 2, 1, 3, 0, 2, 1, 1, 0, 2, 2, 3],
    //     "0.14" =>  [0, 1, 0, 0, 1, 0, 2, 1, 2, 2, 1, 0, 4, 1, 4, 4],
    //     "0.142" =>  [0, 1, 0, 0, 2, 2, 2, 1, 1, 0, 3, 3, 2, 4, 1, 0],
    //     "0.143" =>  [0, 1, 0, 1, 2, 2, 2, 0, 1, 0, 4, 2, 2, 1, 5, 0],
    //     "0.146" =>  [0, 1, 0, 0, 2, 2, 2, 4, 1, 1, 1, 3, 3, 2, 4, 4],
    //     "0.156" =>  [0, 1, 1, 0, 2, 2, 2, 4, 4, 1, 1, 1, 3, 2, 2, 4],
    //     "0.16" =>  [0, 1, 0, 0, 1, 2, 2, 1, 4, 0, 1, 4, 2, 1, 4, 0],
    //     "0.161" =>  [0, 1, 0, 2, 1, 0, 2, 1, 3, 2, 1, 3, 2, 4, 3, 0],
    //     "0.162" =>  [0, 1, 0, 0, 2, 2, 3, 1, 1, 0, 4, 2, 2, 6, 1, 0],
    //     "0.163" =>  [0, 1, 0, 2, 2, 3, 1, 0, 4, 2, 2, 6, 1, 0, 4, 2],
    //     "0.164" =>  [0, 1, 0, 0, 1, 2, 2, 3, 4, 4, 5, 1, 1, 6, 3, 2],
    //     "0.165" =>  [0, 1, 0, 2, 1, 3, 2, 1, 3, 4, 4, 3, 6, 2, 3, 1],
    //     "0.166" =>  [0, 1, 0, 0, 2, 2, 3, 4, 1, 1, 6, 6, 2, 2, 4, 4],
    //     "0.167" =>  [0, 1, 0, 2, 2, 3, 4, 1, 1, 6, 2, 2, 4, 4, 1, 1],
    //     "0.172" =>  [0, 1, 1, 0, 2, 2, 3, 0, 1, 1, 3, 2, 2, 4, 4, 0],
    //     "0.174" =>  [0, 1, 1, 0, 2, 1, 3, 2, 2, 1, 4, 4, 5, 6, 4, 2],
    //     "0.204" =>  [0, 0, 1, 0, 1, 2, 0, 1, 0, 1, 2, 3, 1, 2, 1, 2],
    //     "0.205" =>  [0, 0, 1, 2, 0, 1, 0, 1, 2, 3, 1, 2, 3, 1, 3, 4],
    //     "0.206" =>  [0, 0, 1, 0, 1, 2, 3, 2, 0, 1, 0, 1, 2, 3, 2, 3],
    //     "0.207" =>  [0, 0, 1, 2, 1, 2, 0, 3, 0, 1, 2, 4, 5, 3, 1, 2],
    //     "0.224" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 1, 4, 3, 0, 4],
    //     "0.244" =>  [0, 0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 1, 5, 6, 7, 3],
    //     "0.245" =>  [0, 0, 1, 2, 1, 2, 3, 4, 5, 1, 5, 6, 7, 3, 2, 1],
    //     "0.264" =>  [0, 0, 1, 2, 3, 4, 5, 1, 6, 3, 2, 5, 1, 8, 6, 7],
    //     "0.314" =>  [0, 1, 2, 0, 1, 2, 0, 2, 1, 2, 3, 1, 2, 4, 5, 3],
    //     "0.324" =>  [0, 1, 0, 2, 1, 3, 0, 1, 3, 4, 0, 2, 3, 4, 2, 1],
    //     "0.334" =>  [0, 1, 2, 0, 1, 2, 0, 3, 1, 2, 3, 1, 2, 4, 3, 5],
    //     "0.336" =>  [0, 1, 2, 0, 3, 1, 2, 4, 0, 3, 1, 2, 0, 3, 4, 1],
    //     "0.342" =>  [0, 1, 0, 1, 2, 3, 2, 0, 1, 0, 3, 2, 3, 4, 5, 0],
    //     "0.344" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 4, 6, 2, 3, 2, 1],
    //     "0.346" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 6, 7, 2, 3, 2, 1],
    //     "0.354" =>  [0, 1, 2, 0, 1, 2, 4, 3, 1, 2, 3, 5, 2, 4, 3, 5],
    //     "0.356" =>  [0, 1, 2, 0, 2, 1, 2, 4, 5, 1, 6, 7, 5, 1, 2, 8],
    //     "0.36" =>  [0, 1, 0, 2, 1, 0, 2, 1, 3, 2, 1, 3, 2, 4, 3, 0],
    //     "0.362" =>  [0, 1, 0, 2, 3, 4, 1, 0, 2, 3, 4, 1, 5, 2, 3, 7],
    //     "0.364" =>  [0, 1, 0, 2, 1, 3, 2, 1, 3, 4, 5, 3, 4, 2, 3, 1],
    //     "0.366" =>  [0, 1, 0, 2, 3, 4, 5, 1, 6, 2, 3, 4, 5, 7, 6, 8],
    //     "0.37" =>  [0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 0, 3, 4, 2, 1],
    //     "0.371" =>  [0, 1, 2, 3, 1, 0, 3, 2, 4, 0, 2, 3, 4, 0, 1, 2],
    //     "0.374" =>  [0, 1, 2, 0, 1, 2, 4, 3, 1, 2, 3, 5, 2, 4, 3, 5],
    //     "0.376" =>  [0, 1, 2, 0, 3, 1, 2, 4, 3, 5, 2, 4, 3, 5, 1, 4],
    //     "0.404" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 6, 3, 3],
    //     "0.414" =>  [0, 0, 1, 1, 0, 2, 2, 3, 4, 4, 0, 1, 1, 3, 2, 2],
    //     "0.416" =>  [0, 0, 1, 1, 2, 2, 3, 4, 1, 1, 6, 6, 3, 2, 2, 1],
    //     "0.444" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 6, 3, 3],
    //     "0.45" =>  [0, 0, 1, 1, 2, 2, 3, 1, 1, 4, 4, 3, 2, 2, 1, 1],
    //     "0.454" =>  [0, 0, 1, 1, 2, 2, 3, 4, 1, 1, 6, 6, 3, 2, 2, 1],
    //     "0.56" =>  [0, 1, 0, 2, 2, 4, 1, 1, 3, 2, 4, 4, 6, 6, 2, 1],
    //     "0.564" =>  [0, 1, 0, 2, 2, 4, 4, 1, 1, 3, 2, 5, 4, 7, 6, 8],
    //     "0.6" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 0, 3, 4, 2],
    //     "0.604" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 5, 3, 4, 5],
    //     "0.606" =>  [0, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 5, 1, 2, 3, 4],
    //     "0.64" =>  [0, 0, 1, 2, 3, 4, 1, 5, 3, 2, 1, 5, 4, 2, 6, 8],
    //     "0.644" =>  [0, 0, 1, 2, 3, 4, 5, 1, 6, 3, 2, 5, 8, 9, 6, 10],
    //     "0.74" =>  [0, 1, 0, 1, 2, 3, 2, 4, 1, 4, 6, 2, 3, 2, 1, 5],
    //     "0.744" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 6, 7, 2, 3, 2, 1],
    //     "0.76" =>  [0, 1, 0, 2, 3, 4, 1, 6, 2, 3, 4, 1, 6, 7, 3, 2],
    //     "0.764" =>  [0, 1, 0, 2, 3, 4, 5, 1, 6, 2, 3, 4, 5, 7, 6, 8],
    //     "0.774" =>  [0, 1, 2, 3, 1, 4, 5, 6, 7, 1, 3, 2, 8, 9, 5, 4],
    //     "0.776" =>  [0, 1, 2, 3, 4, 1, 6, 3, 2, 1, 6, 7, 4, 5, 8, 1],
    // };

    // #[test]
    // fn test_initialize() {
    //     for (rules_str, res) in GAMES_NIMBERS.into_iter() {
    //         let rules = rules_from_str(rules_str);
    //         let initial_len = rules_str.len() - 1; // -1 for '.'
    //         let mut g = vec![UNSET; initial_len];

    //         initialize(&rules, &mut g);

    //         assert_eq!(g, res[0..initial_len]);
    //     }
    // }

    // #[test]
    // fn test_naive() {
    //     for (rules_str, res) in GAMES_NIMBERS.into_iter() {
    //         let rules = rules_from_str(rules_str);

    //         let max = 16;

    //         let mut g = vec![UNSET; max];

    //         initialize(&rules, &mut g);

    //         let first_uninitialized = rules.len();

    //         let mut largest = 2;

    //         for n in 1..first_uninitialized {
    //             largest = std::cmp::max(g[n], largest)
    //         }

    //         let mut seen = make_bitset(largest);

    //         for n in first_uninitialized..max {
    //             g[n] = naive(&rules, &g, n, &mut seen);
    //             if g[n] > largest {
    //                 largest = g[n];
    //                 seen = make_bitset(largest);
    //             }
    //             seen.set_elements(0);
    //         }

    //         assert_eq!(g, res);
    //     }
    // }

    // #[test]
    // fn test_rc_with_empty_common() {
    //     for (rules_str, res) in GAMES_NIMBERS.into_iter() {
    //         let rules = rules_from_str(rules_str);

    //         let max = 16;

    //         let mut g = vec![UNSET; max];

    //         initialize(&rules, &mut g);

    //         let first_uninitialized = rules.len();

    //         let mut largest = 2;

    //         for n in 1..first_uninitialized {
    //             largest = std::cmp::max(g[n], largest)
    //         }

    //         let mut seen = make_bitset(largest);
    //         let mut rares = make_bitset(largest);

    //         let mut rare_idx_and_nimber = vec![];
    //         for (i, &x) in g[1..first_uninitialized].iter().enumerate() {
    //             rare_idx_and_nimber.push((i + 1, x));
    //             rares.set(x, true);
    //         }

    //         for n in first_uninitialized..max {
    //             g[n] = rc(&rules, &g, n, &mut seen, &rares, &rare_idx_and_nimber);
    //             rare_idx_and_nimber.push((n, g[n]));
    //             if g[n] > largest {
    //                 largest = g[n];
    //                 seen = make_bitset(largest);
    //                 rares = make_bitset(largest);
    //                 for i in 1..=n {
    //                     rares.set(g[i], true);
    //                 }
    //             }
    //             seen.set_elements(0);
    //         }

    //         assert_eq!(g, res);
    //     }
    // }

    // #[test]
    // fn test_gen_rares() {
    //     let rules = rules_from_str("0.034");

    //     let max = 10000;

    //     let mut g = vec![UNSET; max];

    //     initialize(&rules, &mut g);

    //     let first_uninitialized = rules.len();

    //     let mut largest = 2;

    //     for n in 1..first_uninitialized {
    //         largest = std::cmp::max(g[n], largest)
    //     }

    //     let mut freq = vec![0 as usize ; (largest + 2).next_power_of_two() - 1];
    //     for n in 1..first_uninitialized {
    //         freq[g[n]] += 1;
    //     }
    //     let mut seen = make_bitset(largest);

    //     for n in first_uninitialized..max {
    //         g[n] = naive(&rules, &g, n, &mut seen);
    //         if g[n] > largest {
    //             largest = g[n];
    //             seen = make_bitset(largest);
    //         }
    //         seen.set_elements(0);

    //         if g[n] >= freq.len() {
    //             freq.resize((g[n] + 2).next_power_of_two() - 1, 0);
    //         }
    //         freq[g[n]] += 1;

    //         let rares = gen_rares(&freq, largest);

    //         for x in 0..(largest + 1).next_power_of_two() {
    //             for y in 0..(largest + 1).next_power_of_two() {
    //                 let x_rare = rares[x];
    //                 let y_rare = rares[y];
    //                 if x_rare && y_rare {
    //                     assert!(rares[x ^ y]);
    //                 }

    //                 if !x_rare && !y_rare {
    //                     assert!(rares[x ^ y]);
    //                 }

    //                 if x_rare && !y_rare {
    //                     assert!(!rares[x ^ y]);
    //                 }
    //                 if !x_rare && y_rare {
    //                     assert!(!rares[x ^ y]);
    //                 }
    //             }
    //         }
    //     }
    // }

}
