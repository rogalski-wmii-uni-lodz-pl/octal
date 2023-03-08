use std::{collections::HashSet, cmp::Reverse};

use bitvec::prelude::*;
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

/// Compute the nimber g[n] leveraging the sparce space phenonmenon, under the following
/// assumptions:
/// * g[0..n] were computed correctly, accodring to the rules of some octal game,
/// * n is at least rules.len(),
/// * rares is a binary vector which in which a set bit at position i signifies that i is a member
/// of R
/// * the rares vector represents a correct decmposition into R and C sets (that is values
/// from g are decomposed into two mutually exclusive sets R and C such that for all x, y in R. x ^
/// y in R, and for all x, y in C, x ^ y in R, and for all x in R, y in C, x ^ y in C).
/// * rare_idx_and_nimber is a vector in which contains all pairs (index, nimber) for nimbers from g
/// such that rares[nimber] is true (in python terms: rare_idx_and_nimber = [(index,nimber) for
/// nimber in g if rares[nimber]]).
///
/// The sparse space phenomenon is an observeable phenomenon in at least some octal games, where
/// the set of nimbers is divisible into two sets: the common (C) and the rare (R) sets, such that:
/// * for all x, y in R. x ^ y in R,
/// * for all x, y in C, x ^ y in R,
/// * for all x in R, y in C, x ^ y in C.
/// Since most successsors of a position are in the form x ^ y where both x and y are common, then
/// a position in an octal game is more likely to *not* have a rare value.
/// It is therefore worthwhile to first check the values of all successsors in the from x ^ y,
/// where x is rare.
/// Since according to the Sprague-Grundy theorem, the nim-value of a position `n` is the smallest
/// natural number such, that is not the nim-value of `n`'s successors, this eliminates all possible
/// common values from the set of possible values of the position `n`.
/// The most likely candidate for the  nim-value of `n` is therefore the smallest common value, or a
/// new rare value (but this is unlikely).
/// To prove that it is the common value, we need to iterate through the unchecked successors, and
/// eliminate rare values smaller than the candidate.
/// If this is successful, then the nim-value of `n` is the candidate.
/// If this is unsuccessful, then the nim-value of `n` is rare.
///
/// The advantage of this algorithm is that if number of elements of R is small (if values in R
/// appear in g at most some constant amount of times), then the algorithm approaches a linear time
/// complexity.  This is especially important, since the time complexity of the naive approach is
/// quadratic.
///
/// Note the following observation made by dr Piotr Beling, that this algorithm works also when the
/// chosen R and C do not correctly contain infrequent and frequent values in g, but as long as the
/// two sents fulfil the criteria outlined above, the algorithm works correctly (although it may
/// work even slower than the naive).  For instance, if we assume that all values are in R, and C
/// is an empty set, then this algorithm still correctly identifies nimbers.
///
pub fn rc(
    rules: &Vec<Rule>,
    g: &Vec<usize>,
    n: usize,
    seen: &mut BitVec<u64, Msb0>,
    rares: &BitVec<u64, Msb0>,
    rare_idx_and_nimber: &Vec<(usize, usize)>,
) -> usize {
    // set the non-xor values
    for i in 1..rules.len() {
        if rules[i].some {
            seen.set(g[n - i] as usize, true);
        }
    }

    // set an obvious 0, if the game has a dividing move to any pair (x, x)
    for i in 1..rules.len() {
        if rules[i].divide && (n - i) & 1 == 0 {
            seen.set(0, true);
            break;
        }
    }

    // iterate over x ^ y such that x is in R
    for (idx, x) in rare_idx_and_nimber.iter() {
        for i in 1..rules.len() {
            if rules[i].divide {
                if n > idx + i {
                    let s = (x ^ g[n - i - idx]) as usize;
                    seen.set(s, true);
                }
            }
        }
    }

    // find the smallest common value
    let mut first_common = 0;
    for i in 0..seen.len() {
        if !seen[i] && !rares[i] {
            first_common = i;
            break;
        }
    }

    let mut mex = bitvec!(u64, Msb0; 0; first_common + 1);
    for i in 0..first_common {
        // seen[first_common] is always false,
        // so we do not need to se it in seen2
        mex.set(i, seen[i]);
    }

    // calculate how many unset rare values in mex remain
    let mut remaining_unset = mex.count_zeros() - 1; // -1 for seen2[first_common]

    // iterate over all x ^ y, including the previously checked
    // break early when all rare values smaller than first_common are set
    for i in 1..rules.len() {
        if remaining_unset == 0 {
            break;
        }

        if rules[i].divide {
            for j in 1..=(n - i) / 2 {
                let a = g[j];
                let b = g[n - i - j];
                let loc = (a ^ b) as usize;

                if loc < first_common && !mex[loc] {
                    // a rare value smaller than first_common and not previously observed found
                    mex.set(loc, true);
                    remaining_unset -= 1;
                    if remaining_unset == 0 {
                        // all smaller values than first_common found, the value is the smallest
                        // not observed common
                        return first_common;
                        // break
                    }
                }
            }
        }
    }

    mex.first_zero().unwrap()
}

/// Generate a bit vector of rare values, maximizing the sum of unset frequencies.
///
/// # Arguments
/// * `freq` - a vector of frequencies of nimbers (freq[x] is the frequency of nimber x in some
/// sequence of nimbers for which we want to generate a bit vector of rare values.
///
/// The bitset has to fulfil the following criteria:
/// * for all set bit x, y in rares  x ^ y is also set,
/// * for all unset bits x, y in rares, x ^ y is set,
/// * for all set bits x and unset bits y in C, x ^ y in unset.
/// while at the same time maximizing the sum of freq[x] if rares[x] is unset.
///
pub fn gen_rares(freq: &Vec<usize>, largest: usize) -> BitVec<u64, Msb0> {
    let mut r = HashSet::new();
    let mut c = HashSet::new();
    let mut vals : Vec<(usize, usize)> = freq.iter().map(|&e| e).enumerate().collect();
    vals.sort_by_key(|(_, f)| Reverse(*f));
    r.insert(0);

    for (x, _) in vals {
        if r.contains(&x) || c.contains(&x) {
            continue
        } else {
            c.insert(x);
            let mut inserted = true;
            while inserted {
                inserted = false;
                for &c1 in c.iter() {
                    for &c2 in c.iter() {
                        inserted |= r.insert(c1 ^ c2);
                    }
                }

                let mut new_r = r.to_owned();

                for &r1 in r.iter() {
                    for &r2 in r.iter() {
                        if r1 != 0 && r2 != 0 && r1 != r2 {
                            inserted |= new_r.insert(r1 ^ r2);
                        }
                    }
                }

                r = new_r;

                let mut new_c = c.to_owned();

                for &r1 in r.iter() {
                    for &c1 in c.iter() {
                        inserted |= new_c.insert(r1 ^ c1);
                    }
                }

                c = new_c;
            }
        }
    }

    let mut rares = make_bitset(largest);
    for &x in r.iter() {
        rares.set(x, true);
    }
    rares
}

#[cfg(test)]
mod test {
    use phf::phf_map;
    use super::*;

    #[test]
    fn test_game_to_rules() {
        let rules = rules_from_str("0.034");

        assert_eq!(
            rules,
            vec![
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: true,
                    some: true,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: true,
                },
            ]
        );

        let rules = rules_from_str("0.012345670");

        assert_eq!(
            rules,
            vec![
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
                Rule {
                    all: true,
                    some: false,
                    divide: false
                },
                Rule {
                    all: false,
                    some: true,
                    divide: false,
                },
                Rule {
                    all: true,
                    some: true,
                    divide: false,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: true,
                },
                Rule {
                    all: true,
                    some: false,
                    divide: true,
                },
                Rule {
                    all: false,
                    some: true,
                    divide: true,
                },
                Rule {
                    all: true,
                    some: true,
                    divide: true,
                },
                Rule {
                    all: false,
                    some: false,
                    divide: false,
                },
            ]
        );
    }

    /// initial values taken from Achim Flammenkamp webpage:
    /// http://wwwhomes.uni-bielefeld.de/achim/octal.html
    static GAMES_NIMBERS: phf::Map<&'static str, [usize; 16]> = phf_map! {
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

    #[test]
    fn test_rc_with_empty_common() {
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
            let mut rares = make_bitset(largest);

            let mut rare_idx_and_nimber = vec![];
            for (i, &x) in g[1..first_uninitialized].iter().enumerate() {
                rare_idx_and_nimber.push((i + 1, x));
                rares.set(x, true);
            }

            for n in first_uninitialized..max {
                g[n] = rc(&rules, &g, n, &mut seen, &rares, &rare_idx_and_nimber);
                rare_idx_and_nimber.push((n, g[n]));
                if g[n] > largest {
                    largest = g[n];
                    seen = make_bitset(largest);
                    rares = make_bitset(largest);
                    for i in 1..=n {
                        rares.set(g[i], true);
                    }
                }
                seen.set_elements(0);
            }

            assert_eq!(g, res);
        }
    }

    #[test]
    fn test_gen_rares() {
        let rules = rules_from_str("0.034");

        let max = 10000;

        let mut g = vec![UNSET; max];

        initialize(&rules, &mut g);

        let first_uninitialized = rules.len();

        let mut largest = 2;

        for n in 1..first_uninitialized {
            largest = std::cmp::max(g[n], largest)
        }

        let mut freq = vec![0 as usize ; (largest + 2).next_power_of_two() - 1];
        for n in 1..first_uninitialized {
            freq[g[n]] += 1;
        }
        let mut seen = make_bitset(largest);

        for n in first_uninitialized..max {
            g[n] = naive(&rules, &g, n, &mut seen);
            if g[n] > largest {
                largest = g[n];
                seen = make_bitset(largest);
            }
            seen.set_elements(0);

            if g[n] >= freq.len() {
                freq.resize((g[n] + 2).next_power_of_two() - 1, 0);
            }
            freq[g[n]] += 1;

            let rares = gen_rares(&freq, largest);

            for x in 0..(largest + 1).next_power_of_two() {
                for y in 0..(largest + 1).next_power_of_two() {
                    let x_rare = rares[x];
                    let y_rare = rares[y];
                    if x_rare && y_rare {
                        assert!(rares[x ^ y]);
                    }

                    if !x_rare && !y_rare {
                        assert!(rares[x ^ y]);
                    }

                    if x_rare && !y_rare {
                        assert!(!rares[x ^ y]);
                    }
                    if !x_rare && y_rare {
                        assert!(!rares[x ^ y]);
                    }
                }
            }
        }
    }

}
