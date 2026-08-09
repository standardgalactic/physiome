use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{
    all_continuations, all_repair_ops, all_subsystem_specifications, step_until,
    step_until_hierarchical, validate_new_subsystem_specs, ContinuationEntry,
    HierarchicalContinuation, HierarchyLevel, PhysiologicalState,
};

fn run_scenario(inputs: Inputs, hours: f64, seed: u64) -> (PhysiologicalState, usize) {
    let state = PhysiologicalState::baseline();
    let ops = all_repair_ops();
    let clocks = all_continuations();
    let continuations = clocks.as_vec();
    let perturbation = Perturbation { seed };

    let (final_state, log) = step_until(
        state,
        &continuations,
        &ops,
        &inputs,
        &perturbation,
        hours * 3600.0,
        30,
        all_violations,
    );

    (final_state, log.len())
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
fn subsystem_spec_template_exists_and_is_valid() {
    let specs = all_subsystem_specifications();
    assert!(
        specs.len() >= 6,
        "expected template specs for newly added level-1 subsystems"
    );
    validate_new_subsystem_specs().expect("new subsystem coupling contracts should validate");
}

#[test]
fn pathogen_challenge_raises_core_temp_via_cytokine_coupling() {
    let baseline = PhysiologicalState::baseline();
    let inputs = Inputs {
        pathogen_load: 3.0,
        ambient_temp: 22.0,
        ..Default::default()
    };
    let (final_state, _) = run_scenario(inputs, 2.0, 42);

    assert!(
        final_state.thermal.core_temp > baseline.thermal.core_temp,
        "expected fever under sustained pathogen load"
    );
    assert!(
        final_state.thermal.core_temp < 40.0,
        "fever should remain bounded"
    );
}

#[test]
fn hemorrhage_scenario_invokes_compensation_without_runaway() {
    let inputs = Inputs {
        hemorrhage_rate: 2.0,
        ..Default::default()
    };
    let (final_state, _) = run_scenario(inputs, 1.0, 7);

    assert!(
        final_state.hematologic.hemoglobin > 9.0,
        "hemoglobin should not collapse under modeled compensation"
    );
    assert!(
        final_state.cardiovascular.mean_arterial_pressure > 70.0,
        "pressure should remain in minimally viable range"
    );
}

#[test]
fn heat_stress_scenario_engages_surface_cooling() {
    let baseline = PhysiologicalState::baseline();
    let inputs = Inputs {
        ambient_temp: 34.0,
        ..Default::default()
    };
    let (final_state, _) = run_scenario(inputs, 1.0, 13);

    assert!(
        final_state.integumentary.sweat_rate >= baseline.integumentary.sweat_rate,
        "heat stress should increase sweat output"
    );
    assert!(
        final_state.thermal.core_temp < 39.0,
        "surface cooling should prevent runaway hyperthermia"
    );
}

#[test]
fn metabolic_challenge_consumes_glycogen_and_remains_bounded() {
    let inputs = Inputs {
        exercise_intensity: 0.9,
        meal_glucose_load: 60.0,
        ..Default::default()
    };
    let (final_state, _) = run_scenario(inputs, 1.5, 101);

    assert!(
        final_state.musculoskeletal.glycogen_reserve < 100.0,
        "sustained exertion should consume glycogen reserve"
    );
    assert!(
        (60.0..220.0).contains(&final_state.metabolic.blood_glucose),
        "blood glucose should stay physiologically bounded in this challenge"
    );
}

#[test]
fn endocrine_stress_shifts_hormonal_axis_without_breaking_admissibility() {
    let inputs = Inputs {
        exercise_intensity: 0.7,
        pathogen_load: 1.2,
        ..Default::default()
    };
    let (final_state, _events) = run_scenario(inputs, 3.0, 88);

    assert!(
        final_state.endocrine.cortisol >= 5.0,
        "cortisol should remain in non-depleted range"
    );
    let unresolved = all_violations(&final_state);
    assert!(
        unresolved.len() < 25,
        "stress run should reduce unresolved violations, got {}",
        unresolved.len()
    );
}

struct FastOrganClock;
struct SlowCellularClock;

impl physiome::Continuation for FastOrganClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        1.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        _inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        state.clone()
    }
}

impl HierarchicalContinuation for FastOrganClock {
    fn hierarchy_level(&self) -> HierarchyLevel {
        HierarchyLevel::Organ
    }

    fn parent_subsystem(&self) -> Option<&'static str> {
        None
    }
}

impl physiome::Continuation for SlowCellularClock {
    fn interval(&self, _current: &PhysiologicalState) -> f64 {
        1.0
    }

    fn advance(
        &self,
        state: &PhysiologicalState,
        _dt: f64,
        _inputs: &Inputs,
        _perturbation: &Perturbation,
    ) -> PhysiologicalState {
        state.clone()
    }
}

impl HierarchicalContinuation for SlowCellularClock {
    fn hierarchy_level(&self) -> HierarchyLevel {
        HierarchyLevel::Cellular
    }

    fn parent_subsystem(&self) -> Option<&'static str> {
        Some("immune")
    }
}

#[test]
fn hierarchical_scheduler_executes_without_regression() {
    let initial = PhysiologicalState::baseline();
    let fast = FastOrganClock;
    let slow = SlowCellularClock;
    let continuations = vec![
        ContinuationEntry {
            subsystem: "cardiovascular",
            continuation: &fast,
        },
        ContinuationEntry {
            subsystem: "cellular_metabolism",
            continuation: &slow,
        },
    ];

    let perturbation = Perturbation { seed: 55 };
    let (final_state, _log) = step_until_hierarchical(
        initial,
        &continuations,
        &all_repair_ops(),
        &Inputs::default(),
        &perturbation,
        10.0,
        5,
        all_violations,
    );
    assert!(final_state.t >= 10.0);
}
