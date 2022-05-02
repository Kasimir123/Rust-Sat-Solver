// Make all crates public
pub mod connections;
pub mod solver;
pub mod variable;

// import solver
use solver::Solver;

// import time for stats
use std::fs::File;
use std::time::Instant;

// main function
fn main() {
    // initialize the solver
    let mut solver = Solver::new();

    // let f = "./benchmark-cases/uf20.cnf";
    // let f = "./benchmark-cases/uf50.cnf";
    // let f = "./benchmark-cases/uf75.cnf";
    // let f = "./benchmark-cases/uf100.cnf";
    // let f = "./benchmark-cases/uf125.cnf";
    // let f = "./benchmark-cases/uf150.cnf";
    let f = "./benchmark-cases/uf175.cnf";
    // let f = "./benchmark-cases/uf200.cnf";
    // let f = "./benchmark-cases/uf250.cnf";

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
    // println!("Number of Variables:      {}", solver.variables.len());
    // println!("Number of Connections:    {}", solver.connections.len());
    println!(
        "Connections Checked:      {}",
        solve_result.connections_checked
    );
    println!("Backtracks:               {}", solve_result.num_backtracks);
}
