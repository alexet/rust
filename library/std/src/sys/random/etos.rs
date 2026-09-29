//! Random data generation through `getentropy`, with a degraded fallback for
//! `HashMap` seeding.
//!
//! mlibc's etos port implements `getentropy` with the CPU's RDSEED/RDRAND, so
//! this normally just works. A CPU (or hypervisor) that offers neither makes it
//! fail with `ENOSYS`, and then the two callers differ. `fill_bytes` (the
//! unstable `std::random` API) must not hand out predictable data, so it keeps
//! `getentropy`'s contract and panics on failure. `hashmap_random_keys` is
//! different: every `HashMap::new()` calls it, and a program that never asked
//! for randomness must not die for want of an entropy source, so it falls back
//! to a weak seed. That only weakens HashDoS resistance, not correctness.

use crate::ptr;

pub fn fill_bytes(bytes: &mut [u8]) {
    // GETENTROPY_MAX is mandated to be at least 256.
    for chunk in bytes.chunks_mut(256) {
        let r = unsafe { libc::getentropy(chunk.as_mut_ptr().cast(), chunk.len()) };
        assert_ne!(r, -1, "failed to generate random data");
    }
}

pub fn hashmap_random_keys() -> (u64, u64) {
    let mut buf = [0u8; 16];
    let ok = unsafe { libc::getentropy(buf.as_mut_ptr().cast(), buf.len()) } == 0;
    if ok {
        let k1 = u64::from_ne_bytes(buf[..8].try_into().unwrap());
        let k2 = u64::from_ne_bytes(buf[8..].try_into().unwrap());
        (k1, k2)
    } else {
        // Same source as the `unsupported` targets: allocation addresses (which
        // ASLR-style loading of a static-PIE makes vary per run) plus the clock.
        let stack = 0u8;
        let heap = Box::new(0u8);
        let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
        let k1 = ptr::from_ref(&stack).addr() as u64 ^ (ts.tv_nsec as u64).rotate_left(32);
        let k2 = ptr::from_ref(&*heap).addr() as u64 ^ ts.tv_sec as u64;
        (k1, k2)
    }
}
