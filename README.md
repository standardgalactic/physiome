[Physiome](https://standardgalactic.github.io/physiome/physiome.pdf)

An admissibility- and repair-driven whole-body physiology simulator, written in Rust with a pure functional core.

Rather than modeling homeostasis as a fixed system of coupled differential equations (the approach taken by HumMod and similar simulators), physiome treats each physiological variable as carrying an *admissibility boundary*: a range of values the rest of the body will accept as valid. When a variable drifts outside its boundary, the engine dispatches *repair operators*, pure state transitions representing the corrective physiology (baroreflex, RAAS, thermoregulation, and so on) that pull it back. Subsystems run on independent clocks rather than one global timestep, since cardiovascular reflexes, hormonal regulation, and hematopoiesis do not share a timescale.

## Structure

```
src/
  lib.rs           module wiring; all_repair_ops(), all_continuations()
  constraint.rs     Constraint, AdmissibilityBoundary, ObservableBoundary (domain-independent engine)
  repair.rs         RepairOp, step(), settle(), Continuation trait, step_until() scheduler (domain-independent engine)
  state.rs          PhysiologicalState struct-of-structs, baseline(), all_violations()
  specification.rs  subsystem template metadata + coupling contract checks
  subsystems/
    cardiovascular.rs, renal.rs, hepatic.rs, gi.rs, nervous.rs,
    immune.rs, hematologic.rs, endocrine.rs, metabolic.rs,
    respiratory.rs, thermal.rs, microcirculation.rs, lymphatic.rs,
    musculoskeletal.rs, integumentary.rs, skeletal.rs, reproductive.rs
examples/
  infection_scenario.rs, hemorrhage_scenario.rs, heat_stress_scenario.rs,
  metabolic_challenge_scenario.rs, endocrine_stress_scenario.rs
tests/
  infection_scenario_test.rs   baseline admissibility + stress + nephrectomy/stenosis/Ang II invariants
```

## Design

Every subsystem implements two traits:

- `ObservableBoundary` exposes the subsystem's current observable variables and their admissible ranges, so the repair engine can iterate over subsystems uniformly without knowing anything domain-specific about any of them.
- `Continuation` declares how often the subsystem wants to be advanced (`interval`) and how to advance it (`advance`). The scheduler in `step_until` always advances whichever subsystem's next-fire time is soonest, rather than forcing every subsystem onto one global timestep.

A `RepairOp` is data, not a branch in the engine: a name, a predicate for which violations it responds to, and a pure `State -> State` transition. Adding a new corrective pathway means adding a new `RepairOp` value; the dispatch loop in `repair::step` never changes. Multiple ops can respond to the same violation (an elevated cytokine level triggers both `immune_resolution`, pulling it back down, and `fever_response`, raising `thermal::core_temp`); `step` applies every matching op, not just the first.

Cross-subsystem coupling happens only through the shared `PhysiologicalState`: a repair op can read another subsystem's field (hematologic's coagulation response reads hepatic's `clotting_factors`) and write to a third (immune's fever_response writes to thermal's `core_temp`), but subsystem modules never call each other directly.

State-dependent admissibility is supported where physiology demands it: thermal admissibility shifts with cytokine-driven fever set-point, and renal admissibility now scales with `renal.functioning_mass` so nephrectomy-like scenarios can degrade gracefully without being judged against two-kidney GFR bounds.

## Running it

```bash
cargo build
cargo run --example infection_scenario
cargo run --example hemorrhage_scenario
cargo run --example heat_stress_scenario
cargo run --example metabolic_challenge_scenario
cargo run --example endocrine_stress_scenario
cargo test
```

The examples run reproducible stress scenarios from a healthy baseline and print final state summaries. The tests assert baseline admissibility, validate subsystem specification/coupling contracts, and enforce scenario-level physiological invariants.

## Current subsystems

Seventeen are implemented: cardiovascular, renal, hepatic, gastrointestinal, nervous (autonomic), immune, hematologic, endocrine, metabolic, respiratory, thermal, microcirculation, lymphatic, musculoskeletal, integumentary, skeletal, and reproductive. Each carries its own admissible ranges, at least one repair operator, and its own clock.

## Status and roadmap

This is an early architectural sketch, not a validated physiology model. The full project motivation, architecture rationale, implementation details, experimental results (including three real bugs found and fixed during development), and a roadmap toward a complete organ-system specification are written up in `docs/physiome.tex`.
