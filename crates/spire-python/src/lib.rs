//! Native bindings for Spire Lab, exposed as `spire_lab._native`.
//!
//! The PyO3 module is compiled only under the `extension` feature, which
//! maturin activates for extension builds (see `pyproject.toml`). Plain
//! `cargo test --workspace` compiles the crate without PyO3, so it links
//! with only the Rust toolchain installed.
//!
//! Build metadata is emitted by `build.rs`; see `build_info()`.

/// The extension name reported by [`build_info`].
pub const EXTENSION_NAME: &str = "spire-python";

/// The revision (git SHA or `unknown`) this crate was compiled from.
pub const REVISION: &str = env!("SPIRE_REVISION");

/// The Rust compiler version this crate was compiled with, e.g. `1.99.0`.
pub const RUST_VERSION: &str = env!("SPIRE_RUST_VERSION");

mod export_guard;
#[cfg(test)]
mod tests;
#[cfg(feature = "extension")]
mod config;
#[cfg(feature = "extension")]
mod env;
#[cfg(feature = "extension")]
mod errors;
#[cfg(feature = "extension")]
mod numeric;

#[cfg(feature = "extension")]
use pyo3::prelude::*;
#[cfg(feature = "extension")]
use pyo3::types::PyDict;

/// Build metadata for the compiled native extension.
#[cfg(feature = "extension")]
#[pyfunction]
fn build_info(py: Python) -> PyResult<Py<PyDict>> {
    let info = PyDict::new(py);
    info.set_item("extension", EXTENSION_NAME)?;
    info.set_item("revision", REVISION)?;
    info.set_item("rust_version", RUST_VERSION)?;
    Ok(info.into())
}

/// The `spire_lab._native` module.
#[cfg(feature = "extension")]
#[pymodule]
fn _native(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(build_info, m)?)?;
    m.add_class::<env::NativeCombatPool>()?;
    errors::register(m)?;
    Ok(())
}
