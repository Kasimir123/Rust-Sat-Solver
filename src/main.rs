// Make all crates public
pub mod connections;
pub mod solver;
pub mod variable;
pub mod conflict_set;
pub mod sat_linked_hash_set;

// import solver
use solver::Solver;

// import time for stats
use std::fs::File;
use std::time::Instant;

// main function
fn main() {



    // // make sure everything completes (prints the failed file name if user error)
    // let paths = std::fs::read_dir("./benchmark-cases/20-sat/").unwrap();
    // // let paths = fs::read_dir("./benchmark-cases/50-sat/").unwrap();
    // for path in paths {
    //     let mut solver = Solver::new();
    //     let file_path_buffer = path.as_ref().unwrap().path();
    //     let benchmark_file = File::open(file_path_buffer).expect("failed to open benchmark file");
    //     solver
    //         .load_cnf(benchmark_file)
    //         .expect("failed to parse benchmark file");
    //     solver.solve();
    //     if !solver.final_check() {
    //         println!("Failed: {}", path.unwrap().path().display());
    //     }
    // }
    // println!("Success!");


    
    // initialize the solver
    let mut solver = Solver::new();

    // let f = "./benchmark-cases/CBS_k3_n100_m403_b10_0.cnf";
    // let f = "./benchmark-cases/CBS_k3_n100_m449_b90_0.cnf";
    // let f = "./benchmark-cases/flat100-1.cnf";
    // let f = "./benchmark-cases/flat150-1.cnf";
    // let f = "./benchmark-cases/flat200-1.cnf";
    // let f = "./benchmark-cases/uf20.cnf";

    let f = "./benchmark-cases/20-sat/uf20-0519.cnf";
    
    // let f = "./benchmark-cases/uf50.cnf";
    // let f = "./benchmark-cases/uf75.cnf";
    // let f = "./benchmark-cases/uf100.cnf";
    // let f = "./benchmark-cases/uf125.cnf";
    // let f = "./benchmark-cases/uf150.cnf";
    // let f = "./benchmark-cases/uf175.cnf";
    // let f = "./benchmark-cases/uf200.cnf";
    // let f = "./benchmark-cases/uf250.cnf";
    
    // note that implementation is limited to 250 variables for now
    // see conflict_set.rs
    // let f = "./benchmark-cases/f600.cnf";

    // if you want to see what unsat looks like (it's not pretty, but at least it's fast lol)
    // let f = "./benchmark-cases/uuf50-01.cnf";

    let benchmark_file = File::open(f).expect("failed to open benchmark file");

    // start the timer
    let start = Instant::now();

    // load the sat problem
    solver
        .load_cnf(benchmark_file)
        .expect("failed to parse benchmark file");

    // run the solver and print the result
    let solve_result = solver.solve();

    // get the elapsed time
    let elapsed = start.elapsed().as_secs_f64();

    println!("primary check: {:?}", solve_result.sat);
    println!("user-error check: {:?}", solver.final_check());
    // solver.print_variables();

    // print out stats
    println!("Time:                     {}", elapsed);
    println!(
        "Connections Checked:      {}",
        solve_result.connections_checked
    );
    println!("Backtracks:               {}", solve_result.num_backtracks);
}
