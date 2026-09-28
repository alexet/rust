# `x86_64-unknown-etos`

**Tier: 3**

[etos](https://github.com/alexet/etos) is a small object-capability microkernel
for x86-64. Its C library is [mlibc](https://github.com/managarm/mlibc) (using
mlibc's `abis/linux` headers), and this target builds `std` on top of it the
same way other Unix-like targets do: through the `libc` crate and std's `unix`
backend.

## Target maintainers

[@alexet](https://github.com/alexet)

## Requirements

Cross-compiled only; `std` is supported, host tools are not. Binaries are
static-PIE ELF executables with no interpreter, and every relocation must be
`R_X86_64_RELATIVE` (that is all etos's loader applies), so `crt-static` is the
only mode and dynamic linking is unavailable.

Linking needs an mlibc sysroot containing `libc.a`, `Scrt1.o`, `crti.o` and
`crtn.o` (mlibc keeps libm and pthreads inside `libc.a`), plus `libunwind.a`.
Unwinding is LLVM's libunwind, built for etos and installed in the sysroot; it
finds unwind tables through `.eh_frame_hdr` (the target links with
`--eh-frame-hdr`) and needs a `dl_iterate_phdr` that works in a static-PIE, which
the sysroot's `libunwind.a` also provides because mlibc's only works through its
dynamic linker. The linker is `clang` driving `rust-lld` (`gnu-lld-cc`); pass the
sysroot with `-C link-arg=-L<sysroot>/usr/lib`. Undefined weak symbols are linked
as null rather than given dynamic relocations, since etos's loader applies only
`R_X86_64_RELATIVE`.

## Building

The `libc` crate does not yet have etos bindings upstream; `library/Cargo.toml`
patches it to the `etos` branch of `alexet/rust-libc`.

```toml
[build]
target = ["x86_64-unknown-etos"]

[target.x86_64-unknown-etos]
linker = "clang"
rustflags = ["-Clink-arg=-L/path/to/mlibc-sysroot/usr/lib"]
```

## What works

`std` is limited by what mlibc's etos port implements: files and directories
(`std::fs`), memory, threads, the clock, and standard I/O work. Pipes, sockets,
`poll`, `fork`/`exec` (`std::process::Command`), signals and file locking are not
implemented by mlibc on etos and fail at run time with `ENOSYS`, which `std`
reports as `ErrorKind::Unsupported`. Details that differ from other Unix targets:

- Start-up skips `/dev/null` fd sanitising and the SIGPIPE disposition, since
  etos has neither.
- `std::env::current_exe` returns `Unsupported`.
- Thread names are not passed to the OS.
- Random data comes from `getentropy`. mlibc's etos port does not implement
  `sys_getentropy` yet, so `std::random` panics, and `HashMap` seeding falls
  back to a weak address/clock-derived seed rather than aborting.
- Panics unwind (`panic=unwind` is the default) and `catch_unwind` works, and
  `std::backtrace` captures frames. Symbol names need debug info or a symbol
  table, which the etos build strips; mlibc also prints a missing-sysdep notice
  for `getcwd`, which the backtrace code calls.

## Testing

This target does not run the Rust test suite.
