//! Admissibility: each observable variable carries a range of values the
//! rest of the system will accept from it, plus a severity function used
//! to rank competing repair candidates when several constraints are
//! violated at once.

use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    pub lo: f64,
    pub hi: f64,
}

impl Constraint {
    pub fn new(lo: f64, hi: f64) -> Self {
        Self { lo, hi }
    }

    /// 0.0 if admissible; otherwise a positive severity proportional to
    /// how far outside the admissible range the value sits, normalized
    /// by the width of the range so severities are comparable across
    /// variables with very different units (mmHg vs mEq/L vs mg/dL...).
    pub fn severity(&self, value: f64) -> f64 {
        let width = self.hi - self.lo;
        if value < self.lo {
            (self.lo - value) / width
        } else if value > self.hi {
            (value - self.hi) / width
        } else {
            0.0
        }
    }

    pub fn is_admissible(&self, value: f64) -> bool {
        self.severity(value) == 0.0
    }
}

/// The admissibility boundary a subsystem exposes: named variable ->
/// constraint. Other subsystems consult this before treating one of
/// this subsystem's outputs as a valid input.
pub type AdmissibilityBoundary = HashMap<&'static str, Constraint>;

#[derive(Clone, Debug)]
pub struct Violation {
    pub subsystem: &'static str,
    pub variable: &'static str,
    pub severity: f64,
}

/// Every subsystem implements this once. The repair engine then
/// iterates over subsystems uniformly — it never needs to know that
/// "cardiovascular" or "hepatic" exist as concepts. This is what keeps
/// the engine domain-independent as the subsystem count grows.
pub trait ObservableBoundary {
    /// This subsystem's current observable variables, by name.
    fn observables(&self) -> HashMap<&'static str, f64>;
    fn boundary() -> AdmissibilityBoundary;
    fn subsystem_name() -> &'static str;
}

/// Optional extension for subsystems whose admissible boundaries depend
/// on whole-body context (state-dependent set points).
pub trait StatefulObservableBoundary<S> {
    fn observables(&self) -> HashMap<&'static str, f64>;
    fn boundary_for(state: &S) -> AdmissibilityBoundary;
    fn subsystem_name() -> &'static str;
}

pub fn detect_violations(
    boundary: &AdmissibilityBoundary,
    subsystem: &'static str,
    values: &HashMap<&'static str, f64>,
) -> Vec<Violation> {
    boundary
        .iter()
        .filter_map(|(var, constraint)| {
            let value = *values.get(var)?;
            let s = constraint.severity(value);
            (s > 0.0).then(|| Violation {
                subsystem,
                variable: var,
                severity: s,
            })
        })
        .collect()
}

/// Detect violations for any subsystem implementing ObservableBoundary,
/// without the caller having to assemble a HashMap or pick the right
/// boundary function by hand.
pub fn detect_violations_for<S: ObservableBoundary>(state: &S) -> Vec<Violation> {
    detect_violations(&S::boundary(), S::subsystem_name(), &state.observables())
}

pub fn detect_violations_for_stateful<S, C>(state: &S, context: &C) -> Vec<Violation>
where
    S: StatefulObservableBoundary<C>,
{
    detect_violations(
        &S::boundary_for(context),
        S::subsystem_name(),
        &state.observables(),
    )
}

/// Convenience: run detect_violations_for across every subsystem in the
/// slice and flatten the results. Callers pass a small adapter closure
/// per subsystem since Rust can't iterate heterogeneous types directly;
/// see `state::all_violations` for the concrete wiring.
pub fn collect_violations(per_subsystem: Vec<Vec<Violation>>) -> Vec<Violation> {
    per_subsystem.into_iter().flatten().collect()
}
