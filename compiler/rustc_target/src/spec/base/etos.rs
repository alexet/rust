use crate::spec::{Cc, Env, LinkerFlavor, Lld, Os, RelroLevel, TargetOptions, cvs};

/// Common options for etos, whose C library is mlibc.
///
/// etos programs are static-PIE executables with no interpreter: the kernel's
/// ELF loader applies only `R_X86_64_RELATIVE` relocations, so there is no
/// dynamic linking, and `crt-static` is the only mode. mlibc ships its own
/// `Scrt1.o`/`crti.o`/`crtn.o` and keeps libm and pthreads inside `libc.a`.
///
/// The link arguments name the CRT objects with `-l:` so they are found on the
/// ordinary `-L` search path (the sysroot's `usr/lib`) without a wrapper, and
/// without needing `crt-objects-fallback`.
pub(crate) fn opts() -> TargetOptions {
    let gnu_cc = LinkerFlavor::Gnu(Cc::Yes, Lld::Yes);
    TargetOptions {
        os: Os::Etos,
        env: Env::Mlibc,
        dynamic_linking: false,
        executables: true,
        families: cvs!["unix"],
        has_rpath: false,
        position_independent_executables: true,
        static_position_independent_executables: true,
        relro_level: RelroLevel::Full,
        has_thread_local: true,
        crt_static_default: true,
        crt_static_respected: true,
        crt_static_allows_dylibs: false,
        // No unwinder is available yet, so std is built as `panic=abort`.
        panic_strategy: crate::spec::PanicStrategy::Abort,
        linker: Some("clang".into()),
        linker_flavor: gnu_cc,
        pre_link_args: TargetOptions::link_args(
            gnu_cc,
            &[
                "-nostdlib",
                "-static-pie",
                "-Wl,--no-dynamic-linker",
                "-l:Scrt1.o",
                "-l:crti.o",
            ],
        ),
        late_link_args: TargetOptions::link_args(gnu_cc, &["-l:crtn.o"]),
        ..Default::default()
    }
}
