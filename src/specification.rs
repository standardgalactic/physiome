#[derive(Clone, Debug, PartialEq)]
pub struct ObservableSpec {
    pub name: &'static str,
    pub unit: &'static str,
    pub admissible_lo: f64,
    pub admissible_hi: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepairSpec {
    pub subsystem: &'static str,
    pub name: &'static str,
    pub triggers: &'static [&'static str],
    pub reads: &'static [&'static str],
    pub writes: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubsystemSpecification {
    pub subsystem: &'static str,
    pub clock_interval_seconds: f64,
    pub observables: &'static [ObservableSpec],
    pub repairs: &'static [RepairSpec],
}

#[derive(Clone, Debug, PartialEq)]
pub struct CouplingContract {
    pub subsystem: &'static str,
    pub allowed_reads: &'static [&'static str],
    pub allowed_writes: &'static [&'static str],
}

pub fn validate_coupling_contracts(
    contracts: &[CouplingContract],
    repairs: &[RepairSpec],
) -> Result<(), String> {
    for repair in repairs {
        let Some(contract) = contracts.iter().find(|c| c.subsystem == repair.subsystem) else {
            return Err(format!(
                "missing coupling contract for repair {}",
                repair.name
            ));
        };
        for read in repair.reads {
            if read.starts_with(contract.subsystem) {
                continue;
            }
            if !contract.allowed_reads.contains(read) {
                return Err(format!(
                    "repair {} reads disallowed field {} for subsystem {}",
                    repair.name, read, contract.subsystem
                ));
            }
        }
        for write in repair.writes {
            if write.starts_with(contract.subsystem) {
                continue;
            }
            if !contract.allowed_writes.contains(write) {
                return Err(format!(
                    "repair {} writes disallowed field {} for subsystem {}",
                    repair.name, write, contract.subsystem
                ));
            }
        }
    }
    Ok(())
}
