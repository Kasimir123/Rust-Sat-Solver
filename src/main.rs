// Make all crates public
pub mod variable;
pub mod connections;
pub mod solver;

// import solver
use solver::Solver;

// import time for stats
use std::time::Instant;

// main function
fn main() {

    // initialize the solver
    let mut solver = Solver::new();

    // load the sat problem
    solver.load_cnf("./benchmark-cases/uf20.cnf");
    // solver.load_cnf("./benchmark-cases/uf50.cnf");

    // start the timer
    let start = Instant::now();

    // run the solver and print the result
    println!("{:?}", solver.solve().unwrap());

    // get the elapsed time
    let elapsed = start.elapsed().as_secs_f64();

    // print out stats
    println!("Time:                     {}", elapsed);
    println!("Number of Variables:      {}", solver.variables.len());
    println!("Number of Connections:    {}", solver.connections.len());
    println!("Connections Checked:      {}", solver.connections_checked);
    println!("Backtracks:               {}", solver.backtracks);
}
