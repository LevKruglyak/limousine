use kdam::{tqdm, BarExt};
use limousine_core::learned::pgm::gapped::gapped_pgm::GappedPGM;
use std::io::Write;
use std::{collections::HashMap, fs, path::Path};
use workload::{Executor, Workload};

pub mod workload;

pub type BenchVal = i32;

struct OneWorkloadManyModelsExperiment<const VERBOSE: bool, const VERIFY: bool> {
    name: String,
    models: Vec<Box<dyn Executor<VERBOSE, VERIFY>>>,
    workload: Workload<VERBOSE, VERIFY>,
    headers: Vec<String>,
    columns: HashMap<String, Vec<u128>>,
}
impl<const VERBOSE: bool, const VERIFY: bool> OneWorkloadManyModelsExperiment<VERBOSE, VERIFY> {
    pub fn new(
        name: &str,
        models: Vec<Box<dyn Executor<VERBOSE, VERIFY>>>,
        workload: Workload<VERBOSE, VERIFY>,
    ) -> Self {
        let headers = vec![
            "INT_EPS".to_string(),
            "LEAF_EPS".to_string(),
            "LEAF_BUFSIZE".to_string(),
            "LEAF_FILL_DEC".to_string(),
            "LEAF_SPLIT_DEC".to_string(),
            "initial_size".to_string(),
            "build_time".to_string(),
            "upsert_time".to_string(),
            "read_time".to_string(),
            "final_size".to_string(),
        ];
        let mut columns = HashMap::new();
        for head in headers.iter() {
            columns.insert(head.clone(), vec![]);
        }
        Self {
            name: name.to_string(),
            models,
            workload,
            headers,
            columns,
        }
    }

    pub fn run(&mut self, ntrials: u32) {
        let mut pb = None;
        if VERBOSE {
            println!("Running experiment...");
            pb = Some(tqdm!(total = self.models.len() * ntrials as usize));
        }
        for trial in 0..ntrials {
            println!("Trial {}", trial);
            for model in self.models.iter_mut() {
                let result = model.measure(&self.workload);
                let mut row = HashMap::new();
                model.help_fill_row(&mut row);
                result.help_fill_row(&mut row);
                for (key, val) in row.iter() {
                    let col = self.columns.get_mut(key).unwrap();
                    col.push(*val);
                }
                if pb.is_some() {
                    let mut new_pb = pb.unwrap();
                    new_pb.update(1).ok();
                    pb = Some(new_pb);
                }
            }
            // Save progress
            if trial % 1 == 0 {
                self.to_csv();
                for (_, val) in self.columns.iter_mut() {
                    val.clear();
                }
            }
        }
        self.to_csv();
        if VERBOSE {
            println!("Experiment finished!\n");
        }
    }

    pub fn to_csv(&self) {
        if VERBOSE {
            println!("Saving experiment results...");
        }
        let path = format!("src/learned/pgm/gapped/bench/data/{}.csv", self.name);
        let mut fout = if Path::new(&path).exists() {
            fs::OpenOptions::new().write(true).append(true).open(&path).unwrap()
        } else {
            fs::File::create(&path).unwrap();
            let mut fout = fs::OpenOptions::new().write(true).append(true).open(&path).unwrap();
            for (ix, header) in self.headers.iter().enumerate() {
                write!(fout, "{}", header).unwrap();
                if ix + 1 < self.headers.len() {
                    write!(fout, ",").unwrap();
                } else {
                    write!(fout, "\n").unwrap();
                }
            }
            fout
        };
        // This access pattern is pretty bad but it's fine
        let mut ordered_columns = vec![];
        for header in self.headers.iter() {
            ordered_columns.push(self.columns.get(header).unwrap());
        }
        for rx in 0..ordered_columns[0].len() {
            for cx in 0..ordered_columns.len() {
                write!(fout, "{}", ordered_columns[cx][rx]).unwrap();
                if cx + 1 < self.headers.len() {
                    write!(fout, ",").unwrap();
                } else {
                    write!(fout, "\n").unwrap();
                }
            }
        }
        if VERBOSE {
            println!("Experiment results saved!\n");
        }
    }
}

fn initial_experiment() {
    const VERBOSE: bool = true;
    const VERIFY: bool = false;
    for mul in 2..10 {
        let seed = 0;
        let num_initial = 1_000_000 * mul;
        let num_upserts = 1_000_000 * mul;
        let num_bad_reads = 10_000 * mul;
        let models: Vec<Box<dyn Executor<VERBOSE, VERIFY>>> = vec![
            Box::new(GappedPGM::<BenchVal, 8, 64, 128, 5, 8>::blank()),
            Box::new(GappedPGM::<BenchVal, 8, 64, 128, 9, 10>::blank()),
            Box::new(GappedPGM::<BenchVal, 8, 64, 0, 5, 8>::blank()),
            Box::new(GappedPGM::<BenchVal, 8, 64, 0, 9, 10>::blank()),
        ];
        let workload = Workload::<VERBOSE, VERIFY>::get_uniform_workload(seed, num_initial, num_upserts, num_bad_reads);
        let mut experiment =
            OneWorkloadManyModelsExperiment::new(&format!("InitialExperiment{}e6", mul), models, workload);
        experiment.run(4);
    }
}

fn main() {
    initial_experiment();
}
