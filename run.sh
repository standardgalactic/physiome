#!/usr/bin/env bash
# Convenience script for trying out physiome.
# Run from anywhere; it cd's to its own directory first.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

echo "== cargo build =="
cargo build

echo
echo "== cargo run --example infection_scenario =="
cargo run --example infection_scenario

echo
echo "== cargo run --example hemorrhage_scenario =="
cargo run --example hemorrhage_scenario

echo
echo "== cargo run --example heat_stress_scenario =="
cargo run --example heat_stress_scenario

echo
echo "== cargo run --example metabolic_challenge_scenario =="
cargo run --example metabolic_challenge_scenario

echo
echo "== cargo run --example endocrine_stress_scenario =="
cargo run --example endocrine_stress_scenario

echo
echo "== cargo test =="
cargo test

echo
echo "Done. Try 'cargo run --example infection_scenario' on its own to re-run just the scenario."
