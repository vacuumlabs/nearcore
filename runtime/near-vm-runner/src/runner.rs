use near_primitives::hash::CryptoHash;
use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_primitives::{
    config::VMConfig, profile::ProfileData, types::CompiledContractCache, version::ProtocolVersion,
};
use near_vm_errors::{CompilationError, FunctionCallError, VMError};
use near_vm_logic::types::PromiseResult;
use near_vm_logic::{External, VMContext, VMKind, VMOutcome};
use std::sync::Arc;
use borsh::maybestd::sync::RwLock;
use threadpool::ThreadPool;

#[derive(Debug)]
struct CallInner {
    // smth here
    foo: i32,
}

#[derive(Debug, Clone)]
pub struct ContractCallPrepareRequest {
    pub code_hash: CryptoHash,
    pub code: Vec<u8>,
    pub method_name: Vec<u8>,
}

#[derive(Debug)]
pub struct ContractCallContext {
    prepared: Vec<CallInner>,
    pool: ThreadPool,
}

#[derive(Debug, Clone)]
struct ExecRequest {
    context: Arc<RwLock<ContractCallContext>>,
    request: ContractCallPrepareRequest,
    index: u32,
}

unsafe impl Send for ContractCallPrepareRequest {}
unsafe impl Send for ExecRequest {}

impl ContractCallContext {
    pub fn new(num_threads: usize) -> Arc<RwLock<ContractCallContext>> {
        Arc::new(RwLock::new(ContractCallContext {
            prepared: Vec::new(),
            pool: ThreadPool::new(num_threads),
        }))
    }

    pub fn schedule(self: &mut ContractCallContext,
                    context: Arc<RwLock<ContractCallContext>>,
                    request: ContractCallPrepareRequest, index: u32) {
        let copy_request = request.clone();
        self.pool.execute(move || {
            prepare_in_thread(index, copy_request);
        });
    }
}

impl Drop for ContractCallContext {
    fn drop(&mut self) {
        println!("Closing call context");
    }
}

pub struct ContractCallPrepareResult {
    pub context: Arc<RwLock<ContractCallContext>>,
    pub handle: Option<u32>,
    pub error: Option<VMError>,
}

fn prepare_in_thread(index: u32, request: ContractCallPrepareRequest) {
    println!("{} exec on {:?}", index, request.code_hash);
}

/// `run` does the following:
/// - deserializes and validate the `code` binary (see `prepare::prepare_contract`)
/// - injects gas counting into
/// - adds fee to VMLogic's GasCounter for size of contract
/// - instantiates (links) `VMLogic` externs with the imports of the binary
/// - calls the `method_name` with `context.input`
///   - updates `ext` with new receipts, created during the execution
///   - counts burnt and used gas
///   - counts how accounts storage usage increased by the call
///   - collects logs
///   - sets the return data
///  returns result as `VMOutcome`
pub fn run<'a>(
    code_hash: Vec<u8>,
    code: &[u8],
    method_name: &[u8],
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    current_protocol_version: ProtocolVersion,
    cache: Option<&'a dyn CompiledContractCache>,
    #[cfg(feature = "costs_counting")] profile: Option<&ProfileData>,
) -> (Option<VMOutcome>, Option<VMError>) {
    #[cfg(feature = "costs_counting")]
    if let Some(profile) = profile {
        return run_vm_profiled(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            VMKind::default(),
            profile.clone(),
            current_protocol_version,
            cache,
        );
    }
    run_vm(
        code_hash,
        code,
        method_name,
        ext,
        context,
        wasm_config,
        fees_config,
        promise_results,
        VMKind::default(),
        current_protocol_version,
        cache,
    )
}
pub fn run_vm<'a>(
    code_hash: Vec<u8>,
    code: &[u8],
    method_name: &[u8],
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    vm_kind: VMKind,
    current_protocol_version: ProtocolVersion,
    cache: Option<&'a dyn CompiledContractCache>,
) -> (Option<VMOutcome>, Option<VMError>) {
    use crate::wasmer_runner::run_wasmer;
    #[cfg(feature = "wasmtime_vm")]
    use crate::wasmtime_runner::wasmtime_runner::run_wasmtime;
    match vm_kind {
        VMKind::Wasmer => run_wasmer(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            None,
            current_protocol_version,
            cache,
        ),
        #[cfg(feature = "wasmtime_vm")]
        VMKind::Wasmtime => run_wasmtime(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            None,
            current_protocol_version,
            cache,
        ),
        #[cfg(not(feature = "wasmtime_vm"))]
        VMKind::Wasmtime => {
            panic!("Wasmtime is not supported, compile with '--features wasmtime_vm'")
        }
    }
}

pub fn run_vm_profiled<'a>(
    code_hash: Vec<u8>,
    code: &[u8],
    method_name: &[u8],
    ext: &mut dyn External,
    context: VMContext,
    wasm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    vm_kind: VMKind,
    profile: ProfileData,
    current_protocol_version: ProtocolVersion,
    cache: Option<&'a dyn CompiledContractCache>,
) -> (Option<VMOutcome>, Option<VMError>) {
    use crate::wasmer_runner::run_wasmer;
    #[cfg(feature = "wasmtime_vm")]
    use crate::wasmtime_runner::wasmtime_runner::run_wasmtime;
    let (outcome, error) = match vm_kind {
        VMKind::Wasmer => run_wasmer(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            Some(profile.clone()),
            current_protocol_version,
            cache,
        ),
        #[cfg(feature = "wasmtime_vm")]
        VMKind::Wasmtime => run_wasmtime(
            code_hash,
            code,
            method_name,
            ext,
            context,
            wasm_config,
            fees_config,
            promise_results,
            Some(profile.clone()),
            current_protocol_version,
            cache,
        ),
        #[cfg(not(feature = "wasmtime_vm"))]
        VMKind::Wasmtime => {
            panic!("Wasmtime is not supported, compile with '--features wasmtime_vm'")
        }
    };
    match &outcome {
        Some(VMOutcome { burnt_gas, .. }) => profile.set_burnt_gas(*burnt_gas),
        _ => (),
    };
    (outcome, error)
}
/// `precompile` compiles WASM contract to a VM specific format and stores result into the `cache`.
/// Further execution with the same cache will result in compilation avoidance and reusing cached
/// result. `wasm_config` is required as during compilation we decide if gas metering shall be
/// embedded in the native code, and so we take that into account when computing database key.
#[allow(dead_code)]
pub fn precompile<'a>(
    code: &[u8],
    code_hash: &CryptoHash,
    wasm_config: &'a VMConfig,
    cache: &'a dyn CompiledContractCache,
    vm_kind: VMKind,
) -> Option<VMError> {
    use crate::cache::compile_and_serialize_wasmer;
    match vm_kind {
        VMKind::Wasmer => {
            let result = compile_and_serialize_wasmer(code, wasm_config, code_hash, cache);
            result.err()
        }
        VMKind::Wasmtime => Some(VMError::FunctionCallError(FunctionCallError::CompilationError(
            CompilationError::UnsupportedCompiler {
                msg: "Precompilation not supported in Wasmtime yet".to_string(),
            },
        ))),
    }
}

pub fn with_vm_variants(runner: fn(VMKind) -> ()) {
    runner(VMKind::Wasmer);
    #[cfg(feature = "wasmtime_vm")]
    runner(VMKind::Wasmtime);
}

/// Used for testing cost of compiling a module
pub fn compile_module(vm_kind: VMKind, code: &Vec<u8>) -> bool {
    match vm_kind {
        VMKind::Wasmer => {
            use crate::wasmer_runner::compile_module;
            compile_module(code)
        }
        #[cfg(feature = "wasmtime_vm")]
        VMKind::Wasmtime => {
            use crate::wasmtime_runner::compile_module;
            compile_module(code)
        }
        #[cfg(not(feature = "wasmtime_vm"))]
        VMKind::Wasmtime => {
            panic!("Wasmtime is not supported, compile with '--features wasmtime_vm'")
        }
    };
    false
}

pub fn prepare_vm_calls<'a>(
    call_context: Arc<RwLock<ContractCallContext>>,
    requests: Vec<ContractCallPrepareRequest>,
    cache: Option<Arc<RwLock<dyn CompiledContractCache>>>,
    vm_config: &VMConfig,
    vm_kind: VMKind,
) -> Vec<ContractCallPrepareResult> {
    let mut result: Vec<ContractCallPrepareResult> = Vec::new();
    let mut mut_context = call_context.write().unwrap();
    for request in requests {
        let index = mut_context.prepared.len() as u32;
        mut_context.prepared.push(CallInner { foo: 42 });
        mut_context.schedule(call_context.clone(), request, index);
        result.push(ContractCallPrepareResult { context: call_context.clone(), handle: Some(index), error: None });
    }
    result
}

pub fn run_prepared<'a>(
    call_context: Arc<RwLock<ContractCallContext>>,
    prepared: &ContractCallPrepareResult,
    ext: &mut dyn External,
    context: VMContext,
    vm_config: &'a VMConfig,
    fees_config: &'a RuntimeFeesConfig,
    promise_results: &'a [PromiseResult],
    current_protocol_version: ProtocolVersion,
    profile: Option<&ProfileData>,
) -> (Option<VMOutcome>, Option<VMError>) {
    match &prepared.error {
        Some(error) => return (None, Some(error.clone())),
        _ => {}
    }
    match prepared.handle {
        Some(handle) => {
            println!("handle is {}", handle);
        }
        None => panic!("Must be valid"),
    }
    (None, None)
}
