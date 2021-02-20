mod cache;
mod errors;
mod imports;
mod memory;
pub mod prepare;
mod runner;
mod wasmer_runner;
#[cfg(feature = "wasmtime_vm")]
mod wasmtime_runner;
mod preload;

pub use near_vm_errors::VMError;
pub use runner::compile_module;
pub use runner::run;
pub use runner::run_vm;
pub use runner::run_vm_profiled;
pub use runner::with_vm_variants;
pub use preload::preload_vm_calls;
pub use preload::run_preloaded;
pub use preload::{ContractCallPrepareRequest, ContractCallPrepareResult, ContractCallContext};

#[cfg(feature = "costs_counting")]
pub use near_vm_logic::EXT_COSTS_COUNTER;
