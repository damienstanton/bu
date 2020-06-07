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

//! `bu` is a simple backup program
use rayon::prelude::*;
use std::{
    env::current_dir,
    fs,
    io::Error as err,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
/// configuration flags
struct Flags {
    #[structopt(long, default_value = ".")]
    source: String,
    #[structopt(long)]
    sink: String,
    #[structopt(long)]
    include_hidden: bool,
}

/// enumerate directories in a given path, skipping hidden directories and files
fn enumerate_path(input: &Flags) -> Vec<PathBuf> {
    WalkDir::new(&input.source)
        .into_iter()
        .filter_entry(|e: &DirEntry| {
            e.file_name()
                .to_str()
                .map(|s| match input.include_hidden {
                    true => !s.is_empty(),
                    false => !s.starts_with("."),
                })
                .unwrap()
        })
        .filter_map(|e| e.ok())
        .collect::<Vec<DirEntry>>()
        .into_iter()
        .map(|e| e.into_path())
        .collect()
}

fn main() -> Result<(), err> {
    let input = Flags::from_args();
    let wd = match current_dir() {
        Ok(p) => p,
        _ => unreachable!(),
    };
    let backup_iter = enumerate_path(&input).into_par_iter();
    let targets = backup_iter
        .skip(1) // first entry is always the dir itself
        .map(|p| {
            let input_path = format!("{:?}/{:?}", wd.to_str().unwrap(), p.to_str().unwrap());
            let output_path = format!(
                "{:?}/{:?}/{:?}",
                wd.to_str().unwrap(),
                input.sink,
                p.to_str().unwrap()
            );
            (
                String::from(input_path).replace("\"", ""),
                output_path.replace("\"", "").replace("\\", ""),
            )
        })
        .collect::<Vec<(String, String)>>()
        .into_iter();
    let _ = targets
        .map(|f| {
            if Path::new(&f.0).is_dir() {
                match fs::create_dir_all(Path::new(&f.1)) {
                    Ok(_) => 1u64,
                    _ => 0u64,
                }
            } else {
                match fs::copy(Path::new(&f.0), Path::new(&f.1)) {
                    Ok(n) => n,
                    _ => 0u64,
                }
            }
        })
        .collect::<Vec<u64>>();
    Ok(())
}
