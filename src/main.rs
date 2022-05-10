// Make all crates public
pub mod connections;
pub mod propositional;
pub mod solver;
pub mod variable;

// import solver
use solver::Solver;
use propositional::{PropositionalConnection, Operator};

// import time for stats
use std::fs::File;
use std::time::Instant;

// main function
fn main() {
    // initialize the solver
    let mut solver = Solver::new();

    // let f = "./benchmark-cases/CBS_k3_n100_m403_b10_0.cnf";
    // let f = "./benchmark-cases/CBS_k3_n100_m449_b90_0.cnf";
    // let f = "./benchmark-cases/flat100-1.cnf";
    // let f = "./benchmark-cases/flat150-1.cnf";
    // let f = "./benchmark-cases/flat200-1.cnf";
    let f = "./benchmark-cases/online-cnf.cnf";
    // let f = "./benchmark-cases/uf20.cnf";
    // let f = "./benchmark-cases/uf50.cnf";
    // let f = "./benchmark-cases/uf75.cnf";
    // let f = "./benchmark-cases/uf100.cnf";
    // let f = "./benchmark-cases/uf125.cnf";
    // let f = "./benchmark-cases/uf150.cnf";
    // let f = "./benchmark-cases/uf175.cnf";
    // let f = "./benchmark-cases/uf200.cnf";
    // let f = "./benchmark-cases/uf250.cnf";
    // let f = "./benchmark-cases/f600.cnf";

    let benchmark_file = File::open(f).expect("failed to open benchmark file");

    let mut con = PropositionalConnection::new(Operator::AND, false, None);
        let mut con1 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con11 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con12 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con2 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con21 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con22 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con3 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con31 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con32 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con33 = PropositionalConnection::new(Operator::AND, false, None);
        let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
        let not_a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let not_b = PropositionalConnection::new(Operator::NONE, true, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));
        let not_c = PropositionalConnection::new(Operator::NONE, true, Some("c".to_string()));
        let d = PropositionalConnection::new(Operator::NONE, false, Some("d".to_string()));
        let not_d = PropositionalConnection::new(Operator::NONE, true, Some("d".to_string()));
        let e = PropositionalConnection::new(Operator::NONE, false, Some("e".to_string()));
        let not_e = PropositionalConnection::new(Operator::NONE, true, Some("e".to_string()));
        let g = PropositionalConnection::new(Operator::NONE, false, Some("g".to_string()));
        let not_g = PropositionalConnection::new(Operator::NONE, true, Some("g".to_string()));

        con11.variables.push(a);
        con11.variables.push(not_d);

        con12.variables.push(not_a);
        con12.variables.push(d);

        con1.variables.push(con11);
        con1.variables.push(con12);

        con21.variables.push(b.clone());
        con21.variables.push(e.clone());

        con22.variables.push(not_b.clone());
        con22.variables.push(not_e.clone());

        con2.variables.push(con21);
        con2.variables.push(con22);

        con31.variables.push(c.clone());
        con31.variables.push(not_g);
        con31.variables.push(not_b.clone());
        con31.variables.push(not_e.clone());

        con32.variables.push(not_c);
        con32.variables.push(g.clone());
        con32.variables.push(not_b);
        con32.variables.push(not_e);

        con33.variables.push(c);
        con33.variables.push(g);
        con33.variables.push(b);
        con33.variables.push(e);

        con3.variables.push(con31);
        con3.variables.push(con32);
        con3.variables.push(con33);

        con.variables.push(con1);
        con.variables.push(con2);
        con.variables.push(con3);

        println!("{}", con);

    // start the timer
    let start = Instant::now();

    // load the sat problem
    solver
        .load_cnf(benchmark_file)
        .expect("failed to parse benchmark file");

    // solver.load_propositional(con);

    // run the solver and print the result
    let solve_result = solver.solve();

    // get the elapsed time
    let elapsed = start.elapsed().as_secs_f64();

    println!("primary check: {:?}", solve_result.sat);
    println!("user-error check: {:?}", solver.final_check());
    solver.print_variables();

    // print out stats
    println!("Time:                     {}", elapsed);
    println!(
        "Connections Checked:      {}",
        solve_result.connections_checked
    );
    println!("Backtracks:               {}", solve_result.num_backtracks);
}
