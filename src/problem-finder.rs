// quick script to find a 20-variable file that isn't working
// the last file printed before failure doesn't solve correctly
// rustc problem-finder.rs && ./problem-finder
// (run above command from src)
pub mod connections;
pub mod solver;
pub mod variable;
use solver::Solver;
use std::fs;
use std::fs::File;
fn main() {
    let mut solver = Solver::new();
    let paths = fs::read_dir("./../benchmark-cases/all-20-variable/").unwrap();
    for path in paths {
        let f = path.as_ref().unwrap().path();
        let benchmark_file = File::open(f).expect("failed to open benchmark file");
        solver
            .load_cnf(benchmark_file)
            .expect("failed to parse benchmark file");
        println!("Name: {}", path.unwrap().path().display());
        solver.solve();
    }
    println!("Success!");
}
