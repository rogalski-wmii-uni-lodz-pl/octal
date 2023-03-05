use bitvec::prelude::*;
use phf::phf_map;

pub const UNSET: usize = usize::MAX;

/// Rule represents possible moves from a position n after removing some i tokens are removed from a heap
///
/// If all is true, then 0 may be the successor of n if n == i (all tokens may be taken from the
/// heap).
/// if some is true, then n - i may be the successor of n if n > i (some, but not all tokens may
/// be taken from the heap).
/// If divide is true, then a pair (i, n - i) may be the successor of n if n > i (some tokens may
/// be taken from the heap, but the heap must be divided into two nonempty heaps after the tokens
/// are taken).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rule {
    all: bool,
    some: bool,
    divide: bool,
}

impl From<char> for Rule {
    fn from(c: char) -> Self {
        let d = c.to_digit(10).unwrap();
        Rule {
            all: ((d & 1) != 0),
            some: ((d & 2) != 0),
            divide: ((d & 4) != 0),
        }
    }
}

/// Transform a game string like "0.034" into a Vector of Rules
///
/// I-th element of the vector is a Rule which represents possible moves after removing i tokens
/// from a heap.
pub fn rules_from_str(game: &str) -> Vec<Rule> {
    game.chars()
        .filter(|&x| x != '.')
        .map(|c| Rule::from(c))
        .collect()
}

/// Initialize first `rules.len()` elements of g with nim-values of positions.
///
/// Calculate the first `rules.len()` elements naively, but while checking if the rule may be
/// applied (for n in 0..rules.len(), check if i > n).
/// This check is unnecessary for n's larger than `rules.len()`, hence the separate function.
///
/// # Arguments
///
/// * `rules` - a vector of Rule representing rules of the octal game being examinated, usually
/// created by rules_from_str
/// * `g` - a vector of nimbers.  The function will initialize the first rules.len() elements to
/// Sprague-Grundy of the octal game according to the rules of an octal game stored in `rules`.
pub fn initialize(rules: &Vec<Rule>, g: &mut Vec<usize>) {
    g[0] = 0;

    for n in 1..rules.len() {
        let mut seen = bitvec!(u64, Msb0; 0; 2 * rules.len() + 2);

        seen.set(0, n < rules.len() && rules[n].all);

        for i in 1..rules.len() {
            if rules[i].some && n > i {
                seen.set(g[n - i] as usize, true);
            }

            if rules[i].divide && n > i {
                for j in 1..=(n - i) / 2 {
                    let x = g[j];
                    let y = g[n - i - j];
                    seen.set((x ^ y) as usize, true);
                }
            }
        }

        g[n] = seen.first_zero().unwrap();
    }
}

/// Naively compute the nimber g[n] assuming g[0..n] were computed correctly, accodring to the
/// rules of some octal game, assuming that at n is at least rules.len().
///
/// Assumption that n is at least rules.len() makes it possible to omit some checks (for instance,
/// there are no more whole moves possible, and some and divide rules are always applicable, since
/// n is greater than rules.len(0).
///
/// # Arguments
/// * `rules` - a vector of Rule representing rules of the octal game being examinated, usually
/// created by rules_from_str
/// * `g` - a vector of nimbers.  Values g[0..n] should be correctly filled to make it possible to
/// calculate g[n].  Other values may be arbitrary, but usize::MAX is suggested to mark
/// uninitialized values.
/// * `n` - initial position of the game
/// * `seen` - BitVec with enough space to calculate the nimber of n, initialized to zero.  It is
/// assumed to have been initialized to zero, and it is assumed to have a length of at least `2 *
/// g[0..n].iter().max().unwrap().next_power_of_two() + 2` bits.
pub fn naive(rules: &Vec<Rule>, g: &Vec<usize>, n: usize, seen: &mut BitVec<u64, Msb0>) -> usize {
    assert!(n >= rules.len());
    assert!(seen.not_any());

    for i in 1..rules.len() {
        if rules[i].some {
            seen.set(g[n - i] as usize, true);
        }

        if rules[i].divide {
            for j in 1..=(n - i) / 2 {
                let x = g[j];
                let y = g[n - i - j];
                seen.set((x ^ y) as usize, true);
            }
        }
    }

    seen.first_zero().unwrap()
}

pub fn make_bitset(largest: usize) -> BitVec<u64, Msb0> {
    let bits = 2 * largest.next_power_of_two() + 2;
    bitvec!(u64, Msb0; 0; bits)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_to_rules() {
        let rules = rules_from_str("0.034");

        assert_eq!(rules.len(), 4);
        assert_eq!(
            rules[0],
            Rule {
                all: false,
                some: false,
                divide: false
            }
        );
        assert_eq!(
            rules[1],
            Rule {
                all: false,
                some: false,
                divide: false
            }
        );
        assert_eq!(
            rules[2],
            Rule {
                all: true,
                some: true,
                divide: false
            }
        );
        assert_eq!(
            rules[3],
            Rule {
                all: false,
                some: false,
                divide: true
            }
        );

        let rules = rules_from_str("0.012345670");

        assert_eq!(rules.len(), 10);
        assert_eq!(
            rules[0],
            Rule {
                all: false,
                some: false,
                divide: false
            }
        );
        assert_eq!(
            rules[1],
            Rule {
                all: false,
                some: false,
                divide: false
            }
        );
        assert_eq!(
            rules[2],
            Rule {
                all: true,
                some: false,
                divide: false
            }
        );
        assert_eq!(
            rules[3],
            Rule {
                all: false,
                some: true,
                divide: false
            }
        );
        assert_eq!(
            rules[4],
            Rule {
                all: true,
                some: true,
                divide: false
            }
        );
        assert_eq!(
            rules[5],
            Rule {
                all: false,
                some: false,
                divide: true
            }
        );
        assert_eq!(
            rules[6],
            Rule {
                all: true,
                some: false,
                divide: true
            }
        );
        assert_eq!(
            rules[7],
            Rule {
                all: false,
                some: true,
                divide: true
            }
        );
        assert_eq!(
            rules[8],
            Rule {
                all: true,
                some: true,
                divide: true
            }
        );
        assert_eq!(
            rules[9],
            Rule {
                all: false,
                some: false,
                divide: false
            }
        );
    }

    /// initial values taken from Achim Flammenkamp webpage:
    /// http://wwwhomes.uni-bielefeld.de/achim/octal.html
    static GAMES_NIMBERS: phf::Map<&'static str, [usize;16] > = phf_map! {
        "0.004" =>  [0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 0, 3, 3, 3],
        "0.005" =>  [0, 0, 0, 1, 0, 1, 1, 2, 2, 2, 0, 3, 3, 4, 1, 1],
        "0.006" =>  [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 0, 3, 3, 1, 1, 1],
        "0.007" =>  [0, 0, 0, 1, 1, 1, 2, 2, 0, 3, 3, 1, 1, 1, 0, 4],
        "0.014" =>  [0, 0, 1, 0, 0, 1, 0, 1, 2, 2, 1, 2, 3, 4, 0, 1],
        "0.015" =>  [0, 0, 1, 1, 0, 1, 0, 2, 1, 2, 2, 3, 0, 1, 4, 2],
        "0.016" =>  [0, 0, 1, 0, 1, 2, 2, 2, 0, 1, 0, 1, 4, 4, 2, 2],
        "0.024" =>  [0, 0, 0, 1, 1, 2, 2, 3, 0, 4, 1, 1, 2, 5, 3, 2],
        "0.026" =>  [0, 0, 0, 1, 1, 2, 2, 3, 0, 4, 1, 1, 2, 5, 3, 3],
        "0.034" =>  [0, 0, 1, 1, 0, 2, 2, 3, 1, 4, 0, 1, 4, 3, 1, 2],
        "0.04" =>  [0, 0, 0, 0, 1, 1, 1, 2, 2, 0, 3, 3, 1, 1, 1, 0],
        "0.054" =>  [0, 0, 1, 0, 1, 2, 2, 2, 3, 4, 4, 1, 1, 1, 6, 3],
        "0.055" =>  [0, 0, 1, 1, 1, 2, 2, 2, 3, 1, 1, 1, 4, 4, 4, 3],
        "0.06" =>  [0, 0, 0, 1, 1, 2, 2, 0, 3, 1, 1, 2, 2, 3, 3, 4],
        "0.064" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 5, 3, 3],
        "0.104" =>  [0, 1, 0, 0, 0, 1, 0, 2, 2, 1, 2, 2, 4, 1, 0, 4],
        "0.106" =>  [0, 1, 0, 0, 0, 1, 2, 2, 2, 1, 4, 4, 0, 1, 0, 6],
        "0.114" =>  [0, 1, 1, 0, 0, 1, 1, 2, 0, 2, 1, 2, 0, 4, 1, 1],
        "0.125" =>  [0, 1, 0, 2, 1, 1, 0, 2, 1, 3, 0, 1, 1, 3, 0, 2],
        "0.126" =>  [0, 1, 0, 0, 2, 1, 3, 3, 2, 1, 0, 4, 2, 5, 0, 3],
        "0.127" =>  [0, 1, 0, 2, 2, 1, 0, 4, 4, 1, 2, 2, 0, 1, 4, 4],
        "0.135" =>  [0, 1, 1, 2, 0, 1, 1, 2, 0, 3, 1, 1, 0, 3, 1, 2],
        "0.136" =>  [0, 1, 1, 0, 0, 2, 1, 3, 0, 2, 1, 1, 0, 2, 2, 3],
        "0.14" =>  [0, 1, 0, 0, 1, 0, 2, 1, 2, 2, 1, 0, 4, 1, 4, 4],
        "0.142" =>  [0, 1, 0, 0, 2, 2, 2, 1, 1, 0, 3, 3, 2, 4, 1, 0],
        "0.143" =>  [0, 1, 0, 1, 2, 2, 2, 0, 1, 0, 4, 2, 2, 1, 5, 0],
        "0.146" =>  [0, 1, 0, 0, 2, 2, 2, 4, 1, 1, 1, 3, 3, 2, 4, 4],
        "0.156" =>  [0, 1, 1, 0, 2, 2, 2, 4, 4, 1, 1, 1, 3, 2, 2, 4],
        "0.16" =>  [0, 1, 0, 0, 1, 2, 2, 1, 4, 0, 1, 4, 2, 1, 4, 0],
        "0.161" =>  [0, 1, 0, 2, 1, 0, 2, 1, 3, 2, 1, 3, 2, 4, 3, 0],
        "0.162" =>  [0, 1, 0, 0, 2, 2, 3, 1, 1, 0, 4, 2, 2, 6, 1, 0],
        "0.163" =>  [0, 1, 0, 2, 2, 3, 1, 0, 4, 2, 2, 6, 1, 0, 4, 2],
        "0.164" =>  [0, 1, 0, 0, 1, 2, 2, 3, 4, 4, 5, 1, 1, 6, 3, 2],
        "0.165" =>  [0, 1, 0, 2, 1, 3, 2, 1, 3, 4, 4, 3, 6, 2, 3, 1],
        "0.166" =>  [0, 1, 0, 0, 2, 2, 3, 4, 1, 1, 6, 6, 2, 2, 4, 4],
        "0.167" =>  [0, 1, 0, 2, 2, 3, 4, 1, 1, 6, 2, 2, 4, 4, 1, 1],
        "0.172" =>  [0, 1, 1, 0, 2, 2, 3, 0, 1, 1, 3, 2, 2, 4, 4, 0],
        "0.174" =>  [0, 1, 1, 0, 2, 1, 3, 2, 2, 1, 4, 4, 5, 6, 4, 2],
        "0.204" =>  [0, 0, 1, 0, 1, 2, 0, 1, 0, 1, 2, 3, 1, 2, 1, 2],
        "0.205" =>  [0, 0, 1, 2, 0, 1, 0, 1, 2, 3, 1, 2, 3, 1, 3, 4],
        "0.206" =>  [0, 0, 1, 0, 1, 2, 3, 2, 0, 1, 0, 1, 2, 3, 2, 3],
        "0.207" =>  [0, 0, 1, 2, 1, 2, 0, 3, 0, 1, 2, 4, 5, 3, 1, 2],
        "0.224" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 1, 4, 3, 0, 4],
        "0.244" =>  [0, 0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 1, 5, 6, 7, 3],
        "0.245" =>  [0, 0, 1, 2, 1, 2, 3, 4, 5, 1, 5, 6, 7, 3, 2, 1],
        "0.264" =>  [0, 0, 1, 2, 3, 4, 5, 1, 6, 3, 2, 5, 1, 8, 6, 7],
        "0.314" =>  [0, 1, 2, 0, 1, 2, 0, 2, 1, 2, 3, 1, 2, 4, 5, 3],
        "0.324" =>  [0, 1, 0, 2, 1, 3, 0, 1, 3, 4, 0, 2, 3, 4, 2, 1],
        "0.334" =>  [0, 1, 2, 0, 1, 2, 0, 3, 1, 2, 3, 1, 2, 4, 3, 5],
        "0.336" =>  [0, 1, 2, 0, 3, 1, 2, 4, 0, 3, 1, 2, 0, 3, 4, 1],
        "0.342" =>  [0, 1, 0, 1, 2, 3, 2, 0, 1, 0, 3, 2, 3, 4, 5, 0],
        "0.344" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 4, 6, 2, 3, 2, 1],
        "0.346" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 6, 7, 2, 3, 2, 1],
        "0.354" =>  [0, 1, 2, 0, 1, 2, 4, 3, 1, 2, 3, 5, 2, 4, 3, 5],
        "0.356" =>  [0, 1, 2, 0, 2, 1, 2, 4, 5, 1, 6, 7, 5, 1, 2, 8],
        "0.36" =>  [0, 1, 0, 2, 1, 0, 2, 1, 3, 2, 1, 3, 2, 4, 3, 0],
        "0.362" =>  [0, 1, 0, 2, 3, 4, 1, 0, 2, 3, 4, 1, 5, 2, 3, 7],
        "0.364" =>  [0, 1, 0, 2, 1, 3, 2, 1, 3, 4, 5, 3, 4, 2, 3, 1],
        "0.366" =>  [0, 1, 0, 2, 3, 4, 5, 1, 6, 2, 3, 4, 5, 7, 6, 8],
        "0.37" =>  [0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 0, 3, 4, 2, 1],
        "0.371" =>  [0, 1, 2, 3, 1, 0, 3, 2, 4, 0, 2, 3, 4, 0, 1, 2],
        "0.374" =>  [0, 1, 2, 0, 1, 2, 4, 3, 1, 2, 3, 5, 2, 4, 3, 5],
        "0.376" =>  [0, 1, 2, 0, 3, 1, 2, 4, 3, 5, 2, 4, 3, 5, 1, 4],
        "0.404" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 6, 3, 3],
        "0.414" =>  [0, 0, 1, 1, 0, 2, 2, 3, 4, 4, 0, 1, 1, 3, 2, 2],
        "0.416" =>  [0, 0, 1, 1, 2, 2, 3, 4, 1, 1, 6, 6, 3, 2, 2, 1],
        "0.444" =>  [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 1, 1, 5, 6, 3, 3],
        "0.45" =>  [0, 0, 1, 1, 2, 2, 3, 1, 1, 4, 4, 3, 2, 2, 1, 1],
        "0.454" =>  [0, 0, 1, 1, 2, 2, 3, 4, 1, 1, 6, 6, 3, 2, 2, 1],
        "0.56" =>  [0, 1, 0, 2, 2, 4, 1, 1, 3, 2, 4, 4, 6, 6, 2, 1],
        "0.564" =>  [0, 1, 0, 2, 2, 4, 4, 1, 1, 3, 2, 5, 4, 7, 6, 8],
        "0.6" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 0, 3, 4, 2],
        "0.604" =>  [0, 0, 1, 2, 0, 1, 2, 3, 1, 2, 3, 4, 5, 3, 4, 5],
        "0.606" =>  [0, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 5, 1, 2, 3, 4],
        "0.64" =>  [0, 0, 1, 2, 3, 4, 1, 5, 3, 2, 1, 5, 4, 2, 6, 8],
        "0.644" =>  [0, 0, 1, 2, 3, 4, 5, 1, 6, 3, 2, 5, 8, 9, 6, 10],
        "0.74" =>  [0, 1, 0, 1, 2, 3, 2, 4, 1, 4, 6, 2, 3, 2, 1, 5],
        "0.744" =>  [0, 1, 0, 1, 2, 3, 2, 4, 5, 1, 6, 7, 2, 3, 2, 1],
        "0.76" =>  [0, 1, 0, 2, 3, 4, 1, 6, 2, 3, 4, 1, 6, 7, 3, 2],
        "0.764" =>  [0, 1, 0, 2, 3, 4, 5, 1, 6, 2, 3, 4, 5, 7, 6, 8],
        "0.774" =>  [0, 1, 2, 3, 1, 4, 5, 6, 7, 1, 3, 2, 8, 9, 5, 4],
        "0.776" =>  [0, 1, 2, 3, 4, 1, 6, 3, 2, 1, 6, 7, 4, 5, 8, 1],
    };

    #[test]
    fn test_initialize() {
        for (rules_str, res) in GAMES_NIMBERS.into_iter() {
            let rules = rules_from_str(rules_str);
            let initial_len = rules_str.len() - 1; // -1 for '.'
            let mut g = vec![UNSET; initial_len];

            initialize(&rules, &mut g);

            assert_eq!(g, res[0..initial_len]);
        }
    }

    #[test]
    fn test_naive() {
        for (rules_str, res) in GAMES_NIMBERS.into_iter() {
            let rules = rules_from_str(rules_str);

            let max = 16;

            let mut g = vec![UNSET; max];

            initialize(&rules, &mut g);

            let first_uninitialized = rules.len();

            let mut largest = 2;

            for n in 1..first_uninitialized {
                largest = std::cmp::max(g[n], largest)
            }

            let mut seen = make_bitset(largest);

            for n in first_uninitialized..max {
                g[n] = naive(&rules, &g, n, &mut seen);
                if g[n] > largest {
                    largest = g[n];
                    seen = make_bitset(largest);
                }
                seen.set_elements(0);
            }


            assert_eq!(g, res);
        }
    }
}
