//! Repair operators (declarative, not control-flow), the repair loop
//! that dispatches them against detected violations, and the
//! Continuation trait that lets each subsystem run on its own clock.

use crate::constraint::Violation;
use crate::state::PhysiologicalState;

#[derive(Clone, Debug)]
pub struct RepairEvent {
    pub op_name: &'static str,
    pub target: Violation,
    pub t: f64,
}

/// A repair operator is data: a name, a predicate for which violations
/// it's a candidate response to, and a pure state transition. Adding a
/// new corrective pathway means adding a new RepairOp value — the
/// engine (`step`, below) never changes.
pub struct RepairOp {
    pub name: &'static str,
    pub applies_to: fn(&Violation) -> bool,
    pub apply: fn(&PhysiologicalState, &Violation) -> PhysiologicalState,
}

/// Given a state and a list of already-detected violations, repeatedly
/// take the highest-severity violation and apply EVERY RepairOp whose
/// applies_to matches it (not just the first) — a single derangement
/// can legitimately trigger multiple simultaneous compensatory
/// responses, e.g. an elevated cytokine_level driving both
/// immune_resolution (pulling cytokine back down) and fever_response
/// (raising core_temp) at once. Continue until admissible or the
/// iteration budget runs out. Pure: returns new state + log.
pub fn step(
    state: &PhysiologicalState,
    ops: &[RepairOp],
    violations: Vec<Violation>,
    max_iterations: usize,
) -> (PhysiologicalState, Vec<RepairEvent>) {
    let mut current = state.clone();
    let mut remaining = violations;
    let mut log = Vec::new();

    for _ in 0..max_iterations {
        remaining.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
        let Some(worst) = remaining.first().cloned() else {
            break;
        };
        let matching: Vec<&RepairOp> = ops.iter().filter(|op| (op.applies_to)(&worst)).collect();
        if matching.is_empty() {
            // No admissible repair known for this violation; stop trying
            // it this step rather than looping forever. An unhandled
            // violation is information: a region of physiology with no
            // modeled repair pathway, surfaced rather than hidden.
            break;
        }
        for op in matching {
            current = (op.apply)(&current, &worst);
            log.push(RepairEvent {
                op_name: op.name,
                target: worst.clone(),
                t: current.t,
            });
        }
        remaining.retain(|v| v.variable != worst.variable);
    }

    (current, log)
}

// ---------------------------------------------------------------------
// Continuation: physiology doesn't share one clock. Baroreflex acts
// over seconds, insulin over minutes, hepatic clearance over tens of
// minutes, renal regulation over hours, hematopoiesis over weeks.
// Rather than advancing every subsystem by one global dt, each
// subsystem declares when it next wants to be advanced, and a
// scheduler always advances whichever is soonest.
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub exercise_intensity: f64, // 0.0-1.0
    pub meal_glucose_load: f64,  // g
    pub ambient_temp: f64,       // C
    pub pathogen_load: f64,      // 0.0-1.0, drives immune response
    pub hemorrhage_rate: f64,    // mL/min, drives coagulation/hematologic response
}

/// Deterministic noise source. Held explicitly so a full run is
/// reproducible from (initial_state, inputs_sequence, seed).
#[derive(Clone, Debug)]
pub struct Perturbation {
    pub seed: u64,
}

pub trait Continuation {
    /// Seconds until this subsystem next wants to be advanced, given
    /// the current state (may depend on state — e.g. faster cadence
    /// under stress — but must not depend on when this subsystem last
    /// fired; the scheduler tracks that separately, see step_until).
    fn interval(&self, current: &PhysiologicalState) -> f64;

    /// Advance just this subsystem by dt, returning the full updated
    /// PhysiologicalState (other subsystems' fields pass through
    /// unchanged).
    fn advance(&self, state: &PhysiologicalState, dt: f64, inputs: &Inputs) -> PhysiologicalState;
}

/// Event-driven scheduler: each continuation has its own next-fire time,
/// tracked here rather than recomputed as `global_t + interval` on every
/// iteration — the latter would let the fastest clock's next-fire time
/// always be soonest and monopolize every tick, starving slower
/// subsystems. Repeatedly advance whichever continuation's next-fire
/// time is earliest, then re-check admissibility. Adding a subsystem
/// means adding one Continuation impl and one entry in `continuations`
/// — this function never changes.
pub fn step_until(
    mut state: PhysiologicalState,
    continuations: &[&dyn Continuation],
    ops: &[RepairOp],
    inputs: &Inputs,
    horizon: f64,
    max_repair_iterations: usize,
    violations_after: impl Fn(&PhysiologicalState) -> Vec<Violation>,
) -> (PhysiologicalState, Vec<RepairEvent>) {
    let mut log = Vec::new();
    let mut next_fire: Vec<f64> = continuations
        .iter()
        .map(|c| state.t + c.interval(&state))
        .collect();

    while state.t < horizon {
        let Some((idx, &next_t)) = next_fire
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        else {
            break;
        };
        let dt = (next_t - state.t).max(0.0);
        state = continuations[idx].advance(&state, dt, inputs);
        state.t = next_t;
        next_fire[idx] = state.t + continuations[idx].interval(&state);

        let violations = violations_after(&state);
        let (repaired, mut events) = step(&state, ops, violations, max_repair_iterations);
        state = repaired;
        log.append(&mut events);
    }

    (state, log)
}
