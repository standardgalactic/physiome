//! Run with: cargo run --example heat_stress_scenario

use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{all_continuations, all_repair_ops, step_until, PhysiologicalState};

fn main() {
    let state = PhysiologicalState::baseline();
    let inputs = Inputs {
        ambient_temp: 34.0,
        ..Default::default()
    };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = clocks.as_vec();
    let perturbation = Perturbation { seed: 13 };

    let (final_state, log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        &perturbation,
        3600.0,
        30,
        all_violations,
    );

    println!("events={}", log.len());
    println!("core_temp={:.2}", final_state.thermal.core_temp);
    println!("sweat_rate={:.2}", final_state.integumentary.sweat_rate);
    println!(
        "remaining_violations={}",
        all_violations(&final_state).len()
    );
}
