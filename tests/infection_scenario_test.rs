use physiome::repair::{Inputs, Perturbation};
use physiome::state::all_violations;
use physiome::{
    all_continuations, all_repair_ops, all_subsystem_specifications, settle, step_until,
    step_until_hierarchical, validate_new_subsystem_specs, ContinuationEntry,
    HierarchicalContinuation, HierarchyLevel, PhysiologicalState,
};

fn run_scenario(inputs: Inputs, hours: f64, seed: u64) -> (PhysiologicalState, usize) {
    run_scenario_from(PhysiologicalState::baseline(), inputs, hours, seed)
}

fn run_scenario_from(
    state: PhysiologicalState,
    inputs: Inputs,
    hours: f64,
    seed: u64,
) -> (PhysiologicalState, usize) {
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

fn map_after_hours(inputs: Inputs, hours: f64, seed: u64) -> f64 {
    run_scenario(inputs, hours, seed)
        .0
        .cardiovascular
        .mean_arterial_pressure
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
        (5.0..=23.0).contains(&final_state.endocrine.cortisol),
        "cortisol should remain within endocrine admissibility bounds"
    );
    let unresolved = all_violations(&final_state);
    assert!(
        unresolved.iter().all(|v| v.subsystem != "endocrine"),
        "endocrine stress run should not leave unresolved endocrine violations: {:?}",
        unresolved
    );
    assert!(
        unresolved.len() <= 20,
        "stress run should reduce unresolved violations, got {}",
        unresolved.len()
    );
}

#[test]
fn nephrectomy_half_mass_uses_scaled_renal_admissibility() {
    let mut state = PhysiologicalState::baseline();
    state.renal.functioning_mass = 0.5;
    state.renal.gfr = 52.0;

    let (final_state, _) = run_scenario_from(state, Inputs::default(), 3.0, 2026);
    let unresolved = all_violations(&final_state);

    assert!(
        final_state.renal.gfr < 90.0,
        "half-mass renal state should not be forced to healthy two-kidney GFR"
    );
    assert!(
        unresolved.iter().all(|v| {
            !(v.subsystem == "renal"
                && v.variable == "gfr"
                && (45.0..=60.0).contains(&final_state.renal.gfr))
        }),
        "scaled renal boundary should treat half-mass GFR as admissible, got {:?}",
        unresolved
    );
}

#[test]
fn renal_artery_stenosis_decouples_local_perfusion_from_systemic_pressure() {
    let baseline = PhysiologicalState::baseline();
    let inputs = Inputs {
        renal_artery_stenosis: 0.45,
        ..Default::default()
    };
    let (final_state, _) = run_scenario(inputs, 2.0, 99);

    assert!(
        final_state.renal.perceived_perfusion_pressure
            < final_state.cardiovascular.mean_arterial_pressure,
        "stenosis should make local renal perfusion lower than systemic MAP"
    );
    assert!(
        final_state.cardiovascular.mean_arterial_pressure >= baseline.cardiovascular.mean_arterial_pressure,
        "RAAS compensation under stenosis should avoid systemic hypotension in this sketch"
    );
}

#[test]
fn renal_artery_stenosis_map_reaches_bounded_elevated_plateau() {
    let baseline = PhysiologicalState::baseline();
    let inputs = Inputs {
        renal_artery_stenosis: 0.45,
        ..Default::default()
    };

    let map_2h = map_after_hours(inputs.clone(), 2.0, 99);
    let map_4h = map_after_hours(inputs.clone(), 4.0, 99);
    let map_5h = map_after_hours(inputs, 5.0, 99);

    assert!(
        map_2h >= baseline.cardiovascular.mean_arterial_pressure,
        "stenosis run should trend toward elevated MAP by 2h"
    );
    assert!(map_5h <= 120.0, "stenosis MAP should remain bounded");
    assert!(
        (map_5h - map_4h).abs() <= 2.0,
        "late-horizon MAP should approach a plateau (4h={:.2}, 5h={:.2})",
        map_4h,
        map_5h
    );
}

#[test]
fn high_angii_input_raises_aldosterone_and_retains_volume() {
    let baseline = PhysiologicalState::baseline();
    let inputs = Inputs {
        exogenous_angiotensin_ii: 2.0,
        ..Default::default()
    };
    let (final_state, _) = run_scenario(inputs, 2.0, 314);

    assert!(
        final_state.endocrine.aldosterone > baseline.endocrine.aldosterone,
        "exogenous Ang II should increase aldosterone"
    );
    assert!(
        final_state.renal.plasma_volume >= baseline.renal.plasma_volume,
        "Ang II/aldosterone drive should not deplete renal plasma volume"
    );
}

#[test]
fn settle_helper_reduces_violations_from_scrambled_state() {
    let mut scrambled = PhysiologicalState::baseline();
    scrambled.cardiovascular.mean_arterial_pressure = 55.0;
    scrambled.renal.gfr = 35.0;
    scrambled.immune.cytokine_level = 0.8;
    scrambled.thermal.core_temp = 35.8;

    let before_violations = all_violations(&scrambled);
    let before_total_severity: f64 = before_violations.iter().map(|v| v.severity).sum();
    let (settled, log) = settle(scrambled, &all_repair_ops(), 25, 25, all_violations);
    let after_violations = all_violations(&settled);
    let after_total_severity: f64 = after_violations.iter().map(|v| v.severity).sum();

    assert!(
        !log.is_empty(),
        "settle() should dispatch at least one repair event"
    );
    assert!(
        after_total_severity < before_total_severity,
        "settle() should reduce total severity ({:.3} -> {:.3})",
        before_total_severity,
        after_total_severity
    );
}

#[test]
fn zero_mass_renal_gfr_violation_severity_is_finite() {
    let mut state = PhysiologicalState::baseline();
    state.renal.functioning_mass = 0.0;
    state.renal.gfr = 40.0;

    let renal_gfr = all_violations(&state)
        .into_iter()
        .find(|v| v.subsystem == "renal" && v.variable == "gfr")
        .expect("expected renal gfr violation at zero mass");

    assert!(
        renal_gfr.severity.is_finite(),
        "renal gfr severity should remain finite at zero mass"
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
