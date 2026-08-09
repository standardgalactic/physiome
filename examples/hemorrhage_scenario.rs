//! Run with: cargo run --example hemorrhage_scenario

use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{all_continuations, all_repair_ops, step_until, PhysiologicalState};

fn main() {
    let state = PhysiologicalState::baseline();
    let inputs = Inputs {
        hemorrhage_rate: 2.0,
        ..Default::default()
    };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = clocks.as_vec();
    let perturbation = Perturbation { seed: 7 };

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
    println!(
        "MAP={:.2}",
        final_state.cardiovascular.mean_arterial_pressure
    );
    println!("hemoglobin={:.2}", final_state.hematologic.hemoglobin);
    println!(
        "remaining_violations={}",
        all_violations(&final_state).len()
    );
}
