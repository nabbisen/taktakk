//! Module runtime port: sandboxed Wasm execution.

use crate::error::CoreResult;

/// Input provided to a Wasm module step.
#[derive(Debug)]
pub struct ModuleInput {
    /// Serialized lesson state (JSON or CBOR).
    pub state_bytes: Vec<u8>,
    /// Locale tag (BCP 47).
    pub locale: String,
}

/// Output produced by a Wasm module step.
#[derive(Debug)]
pub struct ModuleOutput {
    /// Updated serialized state, to be persisted as resume point.
    pub state_bytes: Vec<u8>,
    /// Whether the step is considered complete.
    pub step_complete: bool,
}

/// Sandboxed Wasm module executor.
pub trait ModuleRuntime: Send + Sync {
    /// Execute one interactive step of a Wasm module.
    ///
    /// The runtime must verify the object hash before execution and must
    /// enforce fuel, time, and memory limits.
    fn execute_step(
        &self,
        wasm_object_hash: &str,
        input: ModuleInput,
    ) -> CoreResult<ModuleOutput>;
}
