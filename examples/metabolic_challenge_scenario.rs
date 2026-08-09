//! Run with: cargo run --example metabolic_challenge_scenario

use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{all_continuations, all_repair_ops, step_until, PhysiologicalState};

fn main() {
    let state = PhysiologicalState::baseline();
    let inputs = Inputs {
        exercise_intensity: 0.9,
        meal_glucose_load: 60.0,
        ..Default::default()
    };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = clocks.as_vec();
    let perturbation = Perturbation { seed: 101 };

    let (final_state, log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        &perturbation,
        5400.0,
        30,
        all_violations,
    );

    println!("events={}", log.len());
    println!("blood_glucose={:.2}", final_state.metabolic.blood_glucose);
    println!(
        "glycogen={:.2}",
        final_state.musculoskeletal.glycogen_reserve
    );
    println!(
        "remaining_violations={}",
        all_violations(&final_state).len()
    );
}
