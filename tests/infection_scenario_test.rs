use physiome::repair::Inputs;
use physiome::state::all_violations;
use physiome::{all_continuations, all_repair_ops, step_until, Continuation, PhysiologicalState};

fn continuations_vec(clocks: &(
    physiome::subsystems::cardiovascular::CardiovascularClock,
    physiome::subsystems::renal::RenalClock,
    physiome::subsystems::hepatic::HepaticClock,
    physiome::subsystems::gi::GiClock,
    physiome::subsystems::nervous::NervousClock,
    physiome::subsystems::immune::ImmuneClock,
    physiome::subsystems::hematologic::HematologicClock,
    physiome::subsystems::endocrine::EndocrineClock,
    physiome::subsystems::metabolic::MetabolicClock,
    physiome::subsystems::respiratory::RespiratoryClock,
    physiome::subsystems::thermal::ThermalClock,
)) -> Vec<&dyn Continuation> {
    vec![
        &clocks.0, &clocks.1, &clocks.2, &clocks.3, &clocks.4, &clocks.5, &clocks.6, &clocks.7,
        &clocks.8, &clocks.9, &clocks.10,
    ]
}

#[test]
fn baseline_state_is_fully_admissible() {
    let state = PhysiologicalState::baseline();
    let violations = all_violations(&state);
    assert!(
        violations.is_empty(),
        "baseline() should start admissible across all subsystems, found: {:?}",
        violations
    );
}

#[test]
fn pathogen_challenge_raises_core_temp_via_cytokine_coupling() {
    let state = PhysiologicalState::baseline();
    let baseline_temp = state.thermal.core_temp;

    let inputs = Inputs {
        pathogen_load: 3.0,
        ambient_temp: 22.0,
        ..Default::default()
    };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = continuations_vec(&clocks);

    let (final_state, log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        2.0 * 3600.0, // 2 simulated hours
        20,
        all_violations,
    );

    // The coupling should actually have fired.
    let fever_fired = log.iter().any(|e| e.op_name == "fever_response");
    assert!(fever_fired, "fever_response should have fired at least once under sustained pathogen_load");

    // And it should have had a visible effect: core_temp above baseline.
    assert!(
        final_state.thermal.core_temp > baseline_temp,
        "expected fever: core_temp {:.2} should exceed baseline {:.2}",
        final_state.thermal.core_temp,
        baseline_temp
    );

    // But bounded — thermoregulation shouldn't let it run away
    // indefinitely within a 2-hour window at this pathogen_load.
    assert!(
        final_state.thermal.core_temp < 40.0,
        "fever ran away unbounded: core_temp reached {:.2}",
        final_state.thermal.core_temp
    );
}

#[test]
fn no_pathogen_load_keeps_state_near_baseline() {
    let state = PhysiologicalState::baseline();
    let inputs = Inputs {
        ambient_temp: 22.0,
        ..Default::default()
    };

    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = continuations_vec(&clocks);

    let (final_state, _log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        2.0 * 3600.0,
        20,
        all_violations,
    );

    assert!(
        (final_state.thermal.core_temp - 37.0).abs() < 0.5,
        "with no pathogen challenge, core_temp should stay near baseline, got {:.2}",
        final_state.thermal.core_temp
    );
}
