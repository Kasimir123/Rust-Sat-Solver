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

    let benchmark_file =
        File::open("./benchmark-cases/uf50.cnf").expect("failed to open benchmark file");

    // load the sat problem
    // solver.load_cnf("./benchmark-cases/uf20.cnf");
    solver
        .load_cnf(benchmark_file)
        .expect("failed to parse benchmark file");

    // start the timer
    let start = Instant::now();

    let solve_result = solver.solve();
    // run the solver and print the result

    // get the elapsed time
    let elapsed = start.elapsed().as_secs_f64();

    println!("{:?}", solve_result.sat);

    // print out stats
    println!("Time:                     {}", elapsed);
    println!("Number of Variables:      {}", solver.num_variables());
    println!("Number of Connections:    {}", solver.num_connections());
    println!(
        "Connections Checked:      {}",
        solve_result.connections_checked
    );
    println!("Backtracks:               {}", solve_result.num_backtracks);
}
