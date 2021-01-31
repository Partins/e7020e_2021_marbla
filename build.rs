//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use core::f64::consts::PI;
use std::env;
use std::fs::File;
use std::{
    io::{Result, Write},
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    // generate a sine table
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("sin_abs_const.rs");
    let mut f = File::create(&dest_path).unwrap();

    const SINE_BUF_SIZE: usize = 65536;
    write!(f, "const SINE_BUF_SIZE: usize = {};\n", SINE_BUF_SIZE)?;
    write!(f, "const SINE_BUF: [u8; SINE_BUF_SIZE] = [")?;

    for i in 0..SINE_BUF_SIZE {
        let s = ((i as f64) * 2.0 * PI / SINE_BUF_SIZE as f64).sin();
        let v = (128.0 + 128.0 * s) as u8;

        write!(f, " {},", v)?;
    }
    write!(f, "];\n")?;

    Ok(())
}
