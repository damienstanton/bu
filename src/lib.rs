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
    #[structopt(long, default_value = ".")]
    source: String,
    #[structopt(long)]
    sink: String,
    #[structopt(long)]
    include_hidden: bool,
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

/// Copies all data from `params.source` into `params.sink`. If source is not specified in the command-line arguments,
/// the current working directory is assumed to be the directory being backed up. If any errors are encountered during
/// the copy phase, the operation stops and the error is translated into the appropriate OS error value (an `i32`).
pub fn copy_all(params: &Flags, wd: PathBuf) -> Result<Vec<u64>, Option<i32>> {
    enumerate_path(params)
        .into_par_iter()
        .skip(1) // first entry is always the dir itself
        .map(|p| {
            let input_path = format!("{:?}/{:?}", wd.to_str().unwrap(), p.to_str().unwrap());
            let output_path = format!(
                "{:?}/{:?}/{:?}",
                wd.to_str().unwrap(),
                params.sink,
                p.to_str().unwrap()
            );
            (
                String::from(input_path).replace("\"", ""),
                output_path.replace("\"", "").replace("\\", ""),
            )
        })
        .collect::<Vec<(String, String)>>()
        .into_par_iter()
        .progress()
        .map(|f| {
            if Path::new(&f.0).is_dir() {
                match fs::create_dir_all(Path::new(&f.1)) {
                    Ok(_) => Ok(1u64),
                    Err(e) => Err(err::raw_os_error(&e)),
                }
            } else {
                match fs::copy(Path::new(&f.0), Path::new(&f.1)) {
                    Ok(n) => Ok(n),
                    Err(e) => Err(err::raw_os_error(&e)),
                }
            }
        })
        .collect::<Result<Vec<u64>, Option<i32>>>()
}
