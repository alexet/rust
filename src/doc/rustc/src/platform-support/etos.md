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
`crtn.o` (mlibc keeps libm and pthreads inside `libc.a`), plus a `libunwind.a`.
etos has no unwinder yet, so its sysroot provides a placeholder `libunwind.a`
that reports an empty stack, and the target uses `panic=abort`. The linker is
`clang` driving `rust-lld` (`gnu-lld-cc`); pass the sysroot with
`-C link-arg=-L<sysroot>/usr/lib`.

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
- Backtraces are empty and `panic=unwind` is unavailable until a real unwinder
  (LLVM's libunwind) is ported.

## Testing

This target does not run the Rust test suite.
