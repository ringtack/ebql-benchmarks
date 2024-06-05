use std::{env, path::PathBuf};

use libbpf_cargo::SkeletonBuilder;

const VMLINUX: &str = "../bpf";
const SRC: &str = "src/bpf";
const UNOPT_DIR: &str = "unopt";
const OPT_DIR: &str = "opt";
const EBQL_DIR: &str = "ebql";
const BPF_SRC: &str = "pread_query.bpf.c";
const OUT_SRC: &str = "pread_query.skel.rs";

fn main() {
    let mut srcs = vec![];
    for dir in [EBQL_DIR, OPT_DIR, UNOPT_DIR] {
        let mut out =
            PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));
        out.push(format!("{}_{}", dir, OUT_SRC));

        // let src: PathBuf = [SRC, dir, BPF_SRC].iter().collect();
        let mut src: PathBuf = [SRC, dir, BPF_SRC].iter().collect();
        if dir == OPT_DIR {
            src = [SRC, dir, "pread_query_next.bpf.c"].iter().collect();
        }
        srcs.push(src.clone());

        SkeletonBuilder::new()
            .source(src)
            .clang_args([format!("-I{VMLINUX}")])
            .build_and_generate(out)
            .expect("bpf compilation failed");
    }

    for src in srcs {
        println!(
            "cargo:rerun-if-changed={}",
            src.as_os_str().to_str().unwrap()
        );
    }
}
