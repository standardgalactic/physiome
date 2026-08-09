//! Run with: cargo run --example endocrine_stress_scenario

use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{all_continuations, all_repair_ops, step_until, PhysiologicalState};

fn main() {
    let state = PhysiologicalState::baseline();
    let inputs = Inputs {
        exercise_intensity: 0.7,
        pathogen_load: 1.2,
        ..Default::default()
    };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = clocks.as_vec();
    let perturbation = Perturbation { seed: 88 };

    let (final_state, log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        &perturbation,
        3.0 * 3600.0,
        30,
        all_violations,
    );

    println!("events={}", log.len());
    println!("cortisol={:.2}", final_state.endocrine.cortisol);
    println!(
        "sex_hormone_index={:.2}",
        final_state.reproductive.sex_hormone_index
    );
    println!(
        "remaining_violations={}",
        all_violations(&final_state).len()
    );
}
