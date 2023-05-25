// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use pkg_config::Library;

const HEADER_NAME: &str = "trace-cmd.h";
const OUT_FILENAME: &str = "bindings.rs";

fn package_info() -> Result<(PathBuf, Library)> {
    let library = pkg_config::probe_library("libtracecmd").context("failed to probe libracecmd")?;
    for path in &library.include_paths {
        let header_path = path.join(HEADER_NAME);
        if header_path.exists() {
            return Ok((header_path, library));
        }
    }

    Err(anyhow!("{HEADER_NAME} not found"))
}

fn main() -> Result<()> {
    let (header_path, library) = package_info()?;
    println!("cargo:rerun-if-changed={:?}", header_path.as_path());
    println!("cargo:rerun-if-changed=build.rs");

    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .derive_default(true)
        .clang_args(
            library
                .include_paths
                .iter()
                .map(|path| format!("-I{}", path.to_string_lossy())),
        )
        .generate()
        .expect("failed to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings.write_to_file(out_path.join(OUT_FILENAME))?;

    for lib in library.link_files {
        println!("cargo:rustc-link-lib=dylib={:?}", lib.as_os_str());
    }

    Ok(())
}
