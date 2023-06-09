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

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use argh::FromArgs;
use libtracecmd::Event;
use libtracecmd::Handler;
use libtracecmd::Input;
use libtracecmd::Record;
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(FromArgs, Debug)]
/// Command line parameters.
struct Config {
    #[argh(option)]
    /// path to the input .dat file
    input: String,
    #[argh(option)]
    /// number of events that will be printed
    n: usize,
    #[argh(option)]
    /// prefix of trace event names to be counted
    prefix: Option<String>,
}

// Struct to accumulate statistics.
#[derive(Default, Debug)]
struct StatsData {
    cnt: u32,
    stats: BTreeMap<String, u32>,
}

impl StatsData {
    // Print top N events stored in `StatsData`.
    fn print_top_n_events(&self) {
        let cfg = CONFIG.get().unwrap();
        let n = cfg.n;

        let mut vec: Vec<(u32, String)> = vec![];
        for (name, count) in &self.stats {
            vec.push((*count, name.clone()));
        }
        vec.sort();
        vec.reverse();

        let n = std::cmp::min(vec.len(), n);
        println!("Top {n} events:");
        for i in 0..n {
            println!("#{}: {}: {} times", i + 1, vec[i].1, vec[i].0);
        }
    }
}

// Struct that we implement `libtracecmd::Handler` for.
struct TopNStats;

impl Handler for TopNStats {
    /// Type of data passed to the callback to accumulate data.
    type AccumulatedData = StatsData;

    /// Callback that processes each trace event `rec` and accumulate statistics to `data`.
    /// This callback is called for each trace event one by one.
    fn callback(
        input: &mut Input,
        rec: &mut Record,
        _cpu: i32,
        data: &mut Self::AccumulatedData,
    ) -> i32 {
        // Get event
        let event: Event = input.find_event(rec).unwrap();
        let name = event.name;

        let cfg = CONFIG.get().unwrap();

        let name = if let Some(pre) = &cfg.prefix {
            if !name.starts_with(pre) {
                return 0;
            }
            name.trim_start_matches(pre).to_string()
        } else {
            name
        };

        // Store `name` in `data`.
        match data.stats.entry(name) {
            Entry::Vacant(o) => {
                o.insert(1);
            }
            Entry::Occupied(mut o) => {
                *o.get_mut() += 1;
            }
        }

        data.cnt += 1;
        0
    }
}

fn main() {
    let cfg: Config = argh::from_env();
    let input = cfg.input.clone();
    CONFIG.set(cfg).unwrap();
    let mut input = Input::new(&input).unwrap();

    // Calls `Handler::process` implemented for `TopNStats` to get `stats`, which
    let stats = TopNStats::process(&mut input).unwrap();
    stats.print_top_n_events();
}
