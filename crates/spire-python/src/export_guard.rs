//! Fail closed while the production operation lease still owns delivery.
use spire_env::PoolOperation;

#[cfg_attr(not(feature = "extension"), allow(dead_code))]
pub(crate) fn export_committed<T, E: std::fmt::Display>(
    operation: &mut PoolOperation<'_>,
    export: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let result = export();
    if let Err(error) = &result {
        operation.mark_export_fault(&error.to_string());
    }
    result
}

/// Explicitly drained evidence belongs to this binding until successful delivery.
/// Conversion borrows it; an error must neither drop it nor replay gameplay.
#[cfg_attr(not(feature = "extension"), allow(dead_code))]
pub(crate) fn export_retained<T, U, E>(
    pending: &mut Vec<T>, export: impl FnOnce(&[T]) -> Result<U, E>,
) -> Result<U, E> {
    let result = export(pending);
    if result.is_ok() { pending.clear(); }
    result
}
