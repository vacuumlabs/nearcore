use threadpool::ThreadPool;
use near_primitives::hash::CryptoHash;
use std::sync::{Arc, RwLock};
use near_vm_errors::VMError;
use near_primitives::types::CompiledContractCache;
use near_vm_logic::{VMConfig, VMKind, External, VMContext, ProtocolVersion, VMOutcome};
use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_vm_logic::types::PromiseResult;
use near_vm_logic::profile::ProfileData;

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

pub fn preload_vm_calls<'a>(
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

pub fn run_preloaded<'a>(
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
            let context = call_context.read().unwrap();

        }
        None => panic!("Must be valid"),
    }
    (None, None)
}
