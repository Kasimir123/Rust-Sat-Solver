// quick script to find a file that isn't working
// the last file printed before failure doesn't solve correctly
// (if the file-name-print line isn't commented out)
// rustc problem-finder.rs && ./problem-finder
// (run above command from src)
pub mod connections;
pub mod solver;
pub mod variable;
<<<<<<< HEAD
=======
pub mod conflict_set;
pub mod var_unsat;
>>>>>>> backjumping2-avoiding-stack-overflow
use solver::Solver;
use std::fs;
use std::fs::File;
fn main() {
    let mut solver = Solver::new();
<<<<<<< HEAD
    // let paths = fs::read_dir("./../benchmark-cases/20-sat/").unwrap();
    let paths = fs::read_dir("./../benchmark-cases/50-sat/").unwrap();
=======
    let paths = fs::read_dir("./../benchmark-cases/20-sat/").unwrap();
    // let paths = fs::read_dir("./../benchmark-cases/50-sat/").unwrap();
>>>>>>> backjumping2-avoiding-stack-overflow
    for path in paths {
        let f = path.as_ref().unwrap().path();
        let benchmark_file = File::open(f).expect("failed to open benchmark file");
        solver
            .load_cnf(benchmark_file)
            .expect("failed to parse benchmark file");
        // println!("Name: {}", path.unwrap().path().display());
        solver.solve();
    }
    println!("Success!");
}
