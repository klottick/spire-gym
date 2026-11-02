//! Read-only diagnostic validation entry point for the native fuzz harness.
use super::super::Run;

pub fn fuzz_validate(run: &Run) -> Result<(), String> {
    match (run.state.active_combat.as_ref(), run.state.combat.as_ref()) {
        (Some(_), Some(combat)) => {
            combat.fuzz_validate_runtime().map_err(|error| format!("combat runtime validator: {error:?}"))?;
            combat.validate_permanent_mutations_for_run().map_err(|error| format!("permanent mutation validator: {error:?}"))?;
            combat.validate_permanent_upgrades_for_run().map_err(|error| format!("permanent upgrade validator: {error:?}"))?;
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err("Run active-combat owner mismatch".into()),
    }
}

impl Run {
    pub fn fuzz_validate(&self) -> Result<(), String> { fuzz_validate(self) }
}
