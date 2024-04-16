use std::{env, path::PathBuf};

use libbpf_cargo::SkeletonBuilder;

const SRC: &str = "src/bpf/pread_query.bpf.c";
const VMLINUX: &str = "../bpf";

fn main() {
    let mut out =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));
    out.push("pread_query.skel.rs");

    SkeletonBuilder::new()
        .source(SRC)
        .clang_args([format!("-I{VMLINUX}")])
        .build_and_generate(out)
        .expect("bpf compilation failed");

    println!("cargo:rerun-if-changed={}", SRC);
}
