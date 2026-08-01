//! physiome: an admissibility/repair-driven, pure-functional whole-body
//! physiology simulator. Eleven subsystems, one domain-independent
//! engine (constraint.rs, repair.rs). Adding a twelfth subsystem means
//! adding one file under src/subsystems/ plus a few lines of wiring
//! here and in state.rs — the engine itself never changes.

pub mod constraint;
pub mod repair;
pub mod state;
pub mod subsystems;

pub use constraint::{AdmissibilityBoundary, Constraint, ObservableBoundary, Violation};
pub use repair::{step, step_until, Continuation, Inputs, Perturbation, RepairEvent, RepairOp};
pub use state::{all_violations, PhysiologicalState};

use subsystems::{
    cardiovascular, endocrine, gi, hematologic, hepatic, immune, metabolic, nervous, renal,
    respiratory, thermal,
};

/// Every repair operator currently defined, across all eleven
/// subsystems. Pass this to `step` or `step_until`.
pub fn all_repair_ops() -> Vec<RepairOp> {
    vec![
        cardiovascular::baroreflex(),
        renal::raas(),
        hepatic::urea_cycle_upregulation(),
        hepatic::synthetic_upregulation(),
        gi::motility_correction(),
        nervous::autonomic_rebalance(),
        immune::immune_resolution(),
        immune::fever_response(),
        hematologic::coagulation_response(),
        hematologic::erythropoiesis(),
        endocrine::insulin_response(),
        metabolic::glucagon_response(),
        respiratory::ventilation_response(),
        thermal::thermoregulation(),
    ]
}

/// Every subsystem's clock, for use with `step_until`.
pub fn all_continuations() -> (
    cardiovascular::CardiovascularClock,
    renal::RenalClock,
    hepatic::HepaticClock,
    gi::GiClock,
    nervous::NervousClock,
    immune::ImmuneClock,
    hematologic::HematologicClock,
    endocrine::EndocrineClock,
    metabolic::MetabolicClock,
    respiratory::RespiratoryClock,
    thermal::ThermalClock,
) {
    (
        cardiovascular::CardiovascularClock,
        renal::RenalClock,
        hepatic::HepaticClock,
        gi::GiClock,
        nervous::NervousClock,
        immune::ImmuneClock,
        hematologic::HematologicClock,
        endocrine::EndocrineClock,
        metabolic::MetabolicClock,
        respiratory::RespiratoryClock,
        thermal::ThermalClock,
    )
}
