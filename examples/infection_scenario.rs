//! Run with: cargo run --example infection_scenario
//!
//! Scenario: a pathogen challenge starting at t=0. Tracks immune
//! mobilization (leukocytosis), the resulting fever response
//! (cytokine_level -> thermal::core_temp), and whether thermoregulation
//! keeps the fever bounded rather than runaway.

use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{all_continuations, all_repair_ops, step_until, PhysiologicalState};

fn main() {
    let state = PhysiologicalState::baseline();
    println!(
        "t=0.0  wbc={:.2}  cytokine={:.3}  core_temp={:.2}",
        state.immune.wbc_count, state.immune.cytokine_level, state.thermal.core_temp
    );

    let inputs = Inputs {
        pathogen_load: 3.0,
        ambient_temp: 22.0,
        ..Default::default()
    };
    let _perturbation = Perturbation { seed: 42 };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = clocks.as_vec();

    // 4 simulated hours, in seconds.
    let horizon = 4.0 * 3600.0;

    let (final_state, log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        &_perturbation,
        horizon,
        20, // max repair iterations per admissibility check
        all_violations,
    );

    println!("\n--- repair events ({}) ---", log.len());
    let mut fever_events = 0;
    let mut resolution_events = 0;
    for event in &log {
        match event.op_name {
            "fever_response" => fever_events += 1,
            "immune_resolution" => resolution_events += 1,
            _ => {}
        }
        if fever_events + resolution_events <= 6 {
            println!(
                "  t={:>7.1}  {:<24}  target={}::{}  severity={:.3}{}",
                event.t,
                event.op_name,
                event.target.subsystem,
                event.target.variable,
                event.target.severity,
                if event.skipped_due_to_conflict {
                    "  [skipped-conflict]"
                } else {
                    ""
                }
            );
        }
    }
    if log.len() > 6 {
        println!("  ... ({} more events)", log.len() - 6);
    }

    println!(
        "\n--- final state, t={:.0}s ({:.1}h) ---",
        final_state.t,
        final_state.t / 3600.0
    );
    println!(
        "wbc_count       = {:.2}  (baseline 7.00, admissible 4.5-11.0)",
        final_state.immune.wbc_count
    );
    println!(
        "cytokine_level  = {:.3}  (baseline 0.05, admissible 0.0-0.3)",
        final_state.immune.cytokine_level
    );
    println!(
        "core_temp       = {:.2}  (baseline 37.00, admissible 36.5-37.5)",
        final_state.thermal.core_temp
    );
    println!(
        "crp             = {:.2}  (baseline 1.00, admissible 0.0-10.0)",
        final_state.immune.crp
    );

    let remaining = all_violations(&final_state);
    println!("\nremaining unresolved violations: {}", remaining.len());
    for v in &remaining {
        println!(
            "  {}::{}  severity={:.3}",
            v.subsystem, v.variable, v.severity
        );
    }

    println!(
        "\nfever_response fired {} times, immune_resolution fired {} times",
        fever_events, resolution_events
    );
}
