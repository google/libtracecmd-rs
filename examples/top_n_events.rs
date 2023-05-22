use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use argh::FromArgs;
use libtracecmd_rs::Handler;
use libtracecmd_rs::Input;
use libtracecmd_rs::Record;
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

#[derive(Default, Debug)]
struct StatsData {
    cnt: u32,
    stats: BTreeMap<String, u32>,
}

impl StatsData {
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

struct RwStats;

impl Handler for RwStats {
    /// Type of data passed to the callback to accumulate data.
    type DataType = StatsData;

    /// Callback that processes each trace event.
    /// This callback will be passed to `tracecmd_iterate_events` API.
    fn callback(input: &mut Input, rec: &mut Record, _cpu: i32, data: &mut Self::DataType) -> i32 {
        let event = input.find_event(rec).unwrap();
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

    let stats = RwStats::process(&mut input).unwrap();
    stats.print_top_n_events();
}
