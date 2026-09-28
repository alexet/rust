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
///
/// Unwinding is LLVM's libunwind, installed into the sysroot as `libunwind.a`
/// (with a `dl_iterate_phdr` for static-PIE); it locates unwind tables through
/// `.eh_frame_hdr`, hence `--eh-frame-hdr` below.
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
                // libunwind finds unwind tables through PT_GNU_EH_FRAME; the
                // bare-metal clang driver does not ask for `.eh_frame_hdr`.
                "-Wl,--eh-frame-hdr",
                // etos's loader applies only R_X86_64_RELATIVE. In a static
                // executable an undefined weak symbol is simply null, so do not
                // emit a dynamic (GLOB_DAT) relocation for it: mlibc has weak
                // references (e.g. into its dynamic linker) that would
                // otherwise make the binary unloadable.
                "-Wl,-z,nodynamic-undefined-weak",
            ],
        ),
        late_link_args: TargetOptions::link_args(gnu_cc, &["-l:crtn.o"]),
        ..Default::default()
    }
}
