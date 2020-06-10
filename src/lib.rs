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
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::{
    fs,
    io::Error as err,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
/// Parameters used to determine the source and target directories for backups
pub struct Flags {
    #[structopt(long)]
    source: String,
    #[structopt(long)]
    sink: String,
    #[structopt(long)]
    include_hidden: bool,
    #[structopt(long)]
    debug: bool,
}

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
        .into_par_iter()
        .map(|e| e.into_path())
        .collect()
}

fn collect_pairs(params: &Flags) -> Vec<(String, String)> {
    let pairs = enumerate_path(params)
        .into_par_iter()
        .skip(1) // first entry is always the dir itself
        .map(|p| {
            let path = p.canonicalize().unwrap();
            let path_str = path.to_str().unwrap();
            let target_path = Path::new(&params.sink).canonicalize().unwrap();
            let target_str = target_path.to_str().unwrap();
            let final_target_str = String::from(format!("{}/{}", target_str, p.to_str().unwrap()));
            (String::from(path_str), final_target_str)
        })
        .collect::<Vec<(String, String)>>();
    if params.debug {
        println!("pairs: {:#?}", pairs);
    }
    pairs
}

fn create_dirs(pairs: &Vec<(String, String)>) -> Result<Vec<u64>, err> {
    let mut vs: Vec<u64> = Vec::new();
    for p in pairs {
        let path = Path::new(&p.0);
        if path.is_dir() {
            match fs::create_dir_all(Path::new(&p.1)) {
                Ok(_) => {
                    vs.push(1u64);
                }
                Err(_) => eprintln!("Could create dir {}", &p.1),
            };
        }
    }
    Ok(vs)
}

/// Copies all data from `params.source` into `params.sink`. If source is not specified in the command-line arguments,
/// the current working directory is assumed to be the directory being backed up. If any errors are encountered during
/// the copy phase, the operation stops and the error is translated into the appropriate OS error value (an `i32`).
pub fn copy_all(params: &Flags) -> Result<Vec<u64>, err> {
    let pairs = collect_pairs(params);
    create_dirs(&pairs)?;
    pairs
        .into_par_iter()
        .progress()
        .filter(|f| Path::new(&f.0).canonicalize().unwrap().is_file())
        .map(|f| {
            println!("Copying {} to {}", &f.0, &f.1);
            match fs::copy(Path::new(&f.0), Path::new(&f.1)) {
                Ok(n) => Ok(n),
                Err(e) => {
                    println!("Could not copy {} to {}", &f.0, &f.1);
                    Err(err::from_raw_os_error(e.raw_os_error().unwrap()))
                }
            }
        })
        .collect::<Result<Vec<u64>, err>>()
}
