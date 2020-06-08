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

use bu::{copy_all, generate_copy_pairs, Flags};
use std::env::current_dir;
use structopt::StructOpt;
fn main() -> Result<(), Option<i32>> {
    let input = Flags::from_args();
    let wd = match current_dir() {
        Ok(p) => p,
        _ => unreachable!(),
    };
    let pairs = generate_copy_pairs(&input, wd);
    copy_all(pairs)?;
    Ok(())
}
