//! Repair operators (declarative, not control-flow), the repair loop
//! that dispatches them against detected violations, and the
//! Continuation trait that lets each subsystem run on its own clock.

use std::collections::HashSet;

use crate::constraint::Violation;
use crate::state::PhysiologicalState;

#[derive(Clone, Debug)]
pub struct RepairEvent {
    pub op_name: &'static str,
    pub target: Violation,
    pub t: f64,
    pub skipped_due_to_conflict: bool,
}

/// A repair operator is data: a name, a predicate for which violations
/// it's a candidate response to, and a pure state transition. Adding a
/// new corrective pathway means adding a new RepairOp value — the
/// engine (`step`, below) never changes.
pub struct RepairOp {
    pub name: &'static str,
    pub applies_to: fn(&Violation) -> bool,
    pub apply: fn(&PhysiologicalState, &Violation) -> PhysiologicalState,
    pub writes: &'static [&'static str],
}

/// Given a state and a list of already-detected violations, repeatedly
/// take the highest-severity violation and apply EVERY RepairOp whose
/// applies_to matches it (not just the first). Operators with
/// overlapping write targets are conflict-checked; once a field is
/// written for the current violation, another op trying to write the
/// same field is skipped for that iteration.
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
            break;
        }

        let mut writes_seen: HashSet<&'static str> = HashSet::new();
        for op in matching {
            let conflicts = op.writes.iter().any(|w| writes_seen.contains(w));
            if conflicts {
                log.push(RepairEvent {
                    op_name: op.name,
                    target: worst.clone(),
                    t: current.t,
                    skipped_due_to_conflict: true,
                });
                continue;
            }

            for w in op.writes {
                writes_seen.insert(w);
            }
            current = (op.apply)(&current, &worst);
            log.push(RepairEvent {
                op_name: op.name,
                target: worst.clone(),
                t: current.t,
                skipped_due_to_conflict: false,
            });
        }

        remaining.retain(|v| v.variable != worst.variable);
    }

    (current, log)
}

#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub exercise_intensity: f64,
    pub meal_glucose_load: f64,
    pub ambient_temp: f64,
    pub pathogen_load: f64,
    pub hemorrhage_rate: f64,
    pub renal_artery_stenosis: f64, // 0.0-1.0 flow restriction
    pub exogenous_angiotensin_ii: f64, // exogenous agonist dose (relative units)
}

/// Deterministic perturbation source. Kept explicit so runs are
/// reproducible from (initial_state, inputs, seed).
#[derive(Clone, Debug)]
pub struct Perturbation {
    pub seed: u64,
}

impl Perturbation {
    pub fn sample(&self, t: f64, channel: &str) -> f64 {
        let t_hash = t.to_bits().wrapping_mul(0x9E3779B97F4A7C15);
        let mut h = self.seed ^ t_hash ^ hash_channel(channel);
        h ^= h >> 12;
        h ^= h << 25;
        h ^= h >> 27;
        let v = h.wrapping_mul(0x2545F4914F6CDD1D);
        (v as f64 / u64::MAX as f64) * 2.0 - 1.0
    }
}

fn hash_channel(channel: &str) -> u64 {
    channel.bytes().fold(1469598103934665603_u64, |acc, b| {
        (acc ^ b as u64).wrapping_mul(1099511628211)
    })
}

pub trait Continuation {
    fn interval(&self, current: &PhysiologicalState) -> f64;
    fn advance(
        &self,
        state: &PhysiologicalState,
        dt: f64,
        inputs: &Inputs,
        perturbation: &Perturbation,
    ) -> PhysiologicalState;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HierarchyLevel {
    Organ = 0,
    Refinement = 1,
    Cellular = 2,
}

pub trait HierarchicalContinuation: Continuation {
    fn hierarchy_level(&self) -> HierarchyLevel;
    fn parent_subsystem(&self) -> Option<&'static str>;
}

pub struct ContinuationEntry<'a> {
    pub subsystem: &'static str,
    pub continuation: &'a dyn HierarchicalContinuation,
}

pub fn step_until(
    mut state: PhysiologicalState,
    continuations: &[&dyn Continuation],
    ops: &[RepairOp],
    inputs: &Inputs,
    perturbation: &Perturbation,
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
        state = continuations[idx].advance(&state, dt, inputs, perturbation);
        state.t = next_t;
        next_fire[idx] = state.t + continuations[idx].interval(&state);

        let violations = violations_after(&state);
        let (repaired, mut events) = step(&state, ops, violations, max_repair_iterations);
        state = repaired;
        log.append(&mut events);
    }

    (state, log)
}

pub fn step_until_hierarchical(
    mut state: PhysiologicalState,
    continuations: &[ContinuationEntry<'_>],
    ops: &[RepairOp],
    inputs: &Inputs,
    perturbation: &Perturbation,
    horizon: f64,
    max_repair_iterations: usize,
    violations_after: impl Fn(&PhysiologicalState) -> Vec<Violation>,
) -> (PhysiologicalState, Vec<RepairEvent>) {
    let mut log = Vec::new();
    let mut next_fire: Vec<f64> = continuations
        .iter()
        .map(|c| state.t + c.continuation.interval(&state))
        .collect();

    while state.t < horizon {
        let Some((idx, &next_t)) = next_fire.iter().enumerate().min_by(|a, b| {
            let tcmp = a.1.partial_cmp(b.1).unwrap();
            if tcmp != std::cmp::Ordering::Equal {
                return tcmp;
            }
            let la = continuations[a.0].continuation.hierarchy_level();
            let lb = continuations[b.0].continuation.hierarchy_level();
            la.cmp(&lb)
        }) else {
            break;
        };

        let dt = (next_t - state.t).max(0.0);
        state = continuations[idx]
            .continuation
            .advance(&state, dt, inputs, perturbation);
        state.t = next_t;
        next_fire[idx] = state.t + continuations[idx].continuation.interval(&state);

        let violations = violations_after(&state);
        let (repaired, mut events) = step(&state, ops, violations, max_repair_iterations);
        state = repaired;
        log.append(&mut events);
    }

    (state, log)
}

pub fn settle(
    mut state: PhysiologicalState,
    ops: &[RepairOp],
    max_passes: usize,
    max_repair_iterations: usize,
    violations_after: impl Fn(&PhysiologicalState) -> Vec<Violation>,
) -> (PhysiologicalState, Vec<RepairEvent>) {
    let mut log = Vec::new();

    for _ in 0..max_passes {
        let violations = violations_after(&state);
        if violations.is_empty() {
            break;
        }
        let before = violations.len();
        let (next, mut events) = step(&state, ops, violations, max_repair_iterations);
        let after = violations_after(&next).len();
        state = next;
        log.append(&mut events);

        if after >= before {
            break;
        }
    }

    (state, log)
}
