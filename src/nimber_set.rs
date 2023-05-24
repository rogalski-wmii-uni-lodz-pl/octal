use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

pub trait Mex {
    /// Returns the smallest nimber not included in the `self` set.
    fn mex(&self) -> u16;
}

/// Set of nimbers.
/// 
/// Implemented by `u64` slices.
pub trait NimberSet: Mex {
    unsafe fn insert_nimber_unchecked(&mut self, nimber: u16);
    fn insert_nimber(&mut self, nimber: u16);
}

/// Set of nimbers that can be concurrently modified by multiple threads.
/// 
/// Implemented by `AtomicU64` slices.
pub trait AtomicNimberSet: Mex {
    unsafe fn insert_nimber_unchecked(&self, nimber: u16);
    fn insert_nimber(&self, nimber: u16);
}

/// Implemented by `Vec<u64 / AtomicU64>`.
pub trait NimberSetConstructor {
    /// Returns set to which nimbers from `0` to `max_nimber` can be inserted.
    fn with_max_nimber(max_nimber: u16) -> Self;
}

#[inline] fn mex<U64Iter: Iterator<Item = u64>>(iter: &mut U64Iter) -> u16 {
    let mut result = 0;
    for v in iter {
        if v == u64::MAX {
            result += 64;
        } else {
            return result + v.trailing_ones() as u16;
        }
    }
    result
}

impl Mex for [u64] {
    fn mex(&self) -> u16 { 
        mex(&mut self.iter().copied())
     }
}

impl Mex for [AtomicU64] {
    fn mex(&self) -> u16 {
        mex(&mut self.iter().map(|v| v.load(Relaxed))) 
    }
}

impl NimberSet for [u64] {
    unsafe fn insert_nimber_unchecked(&mut self, nimber: u16) {
        *self.get_unchecked_mut((nimber/64) as usize) = 1u64 << (nimber % 64) as u64;
    }
    
    fn insert_nimber(&mut self, nimber: u16) {
        self[(nimber/64) as usize] |= 1u64 << (nimber % 64) as u64;
    }
}

impl AtomicNimberSet for [AtomicU64] {
    unsafe fn insert_nimber_unchecked(&self, nimber: u16) {
        self.get_unchecked((nimber/64) as usize).fetch_or(1u64 << (nimber % 64) as u64, Relaxed);
    }

    fn insert_nimber(&self, nimber: u16) {
        self[(nimber/64) as usize].fetch_or(1u64 << (nimber % 64) as u64, Relaxed);
    }
}

impl NimberSetConstructor for Vec<u64> {
    fn with_max_nimber(max_nimber: u16) -> Self {
        vec![0; max_nimber as usize / 64 + 1]
    }
}

impl NimberSetConstructor for Vec<AtomicU64> {
    fn with_max_nimber(max_nimber: u16) -> Self {
        let len = max_nimber as usize / 64 + 1;
        let mut result = Vec::with_capacity(len);
        for _ in 0..len { result.push(Default::default()); }
        result
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_mex() {
        assert_eq!(vec![0u64].mex(), 0);
        assert_eq!(vec![0b1011u64].mex(), 2);
        assert_eq!(vec![u64::MAX].mex(), 64);
        assert_eq!(vec![u64::MAX, 0b1011u64].mex(), 66);
    }

    #[test]
    fn test_nimber_set() {
        let mut s: Vec<u64> = Vec::with_max_nimber(64);     // insert_nimber needs mut
        for i in 0..=64 { s.insert_nimber(i); }
        assert_eq!(s.mex(), 65);
    }

    #[test]
    fn test_atomic_nimber_set() {
        let s: Vec<AtomicU64> = Vec::with_max_nimber(64);   // mut is not needed
        for i in 0..=64 { s.insert_nimber(i); }
        assert_eq!(s.mex(), 65);
    }
}