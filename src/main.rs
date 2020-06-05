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
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
/// configuration flags
pub struct Flags {
    #[structopt(long, default_value = ".")]
    source: String,
    #[structopt(long)]
    sink: String,
}

/// enumerate directories in a given path, skipping hidden directories and files
fn enumerate_path(path: &str) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_entry(|e: &DirEntry| e.file_name().to_str().map(|s| !s.starts_with(".")).unwrap())
        .filter_map(|e| e.ok())
        .collect::<Vec<DirEntry>>()
        .into_iter()
        .map(|e| e.into_path())
        .collect()
}

fn main() {
    let input = Flags::from_args();
    println!("Backing up {:#?} to {:#?}", input.source, input.sink);
    let sink_path = format!("{}{}", input.sink, "/");
    let backup_iter = enumerate_path(&input.source).into_par_iter();
    let targets = backup_iter
        .skip(1) // first entry is always the dir itself
        .map(|p| {
            let input_path = p.to_str().unwrap();
            let output_path = format!("{:?}{:?}", sink_path, input_path);
            (String::from(input_path), output_path.replace("\"", ""))
        })
        .collect::<Vec<(String, String)>>();
    println!("{:#?}", targets);
}
