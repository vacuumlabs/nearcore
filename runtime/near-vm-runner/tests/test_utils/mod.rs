use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use wabt::Wat2Wasm;

use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_primitives::types::CompiledContractCache;
use near_primitives::version::ProtocolVersion;
use near_vm_errors::VMError;
use near_vm_logic::mocks::mock_external::MockedExternal;
use near_vm_logic::{VMConfig, VMContext, VMKind, VMOutcome};
use near_vm_runner::run_vm;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const CURRENT_ACCOUNT_ID: &str = "alice";
pub const SIGNER_ACCOUNT_ID: &str = "bob";
pub const SIGNER_ACCOUNT_PK: [u8; 3] = [0, 1, 2];
pub const PREDECESSOR_ACCOUNT_ID: &str = "carol";

pub const LATEST_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::MAX;

pub fn create_context(input: Vec<u8>) -> VMContext {
    VMContext {
        current_account_id: CURRENT_ACCOUNT_ID.to_owned(),
        signer_account_id: SIGNER_ACCOUNT_ID.to_owned(),
        signer_account_pk: Vec::from(&SIGNER_ACCOUNT_PK[..]),
        predecessor_account_id: PREDECESSOR_ACCOUNT_ID.to_owned(),
        input,
        block_index: 10,
        block_timestamp: 42,
        epoch_height: 1,
        account_balance: 2u128,
        account_locked_balance: 0,
        storage_usage: 12,
        attached_deposit: 2u128,
        prepaid_gas: 10_u64.pow(14),
        random_seed: vec![0, 1, 2],
        is_view: false,
        output_data_receivers: vec![],
    }
}

pub fn make_simple_contract_call_with_gas_vm(
    code: &[u8],
    method_name: &[u8],
    prepaid_gas: u64,
    vm_kind: VMKind,
) -> (Option<VMOutcome>, Option<VMError>) {
    let mut fake_external = MockedExternal::new();
    let mut context = create_context(vec![]);
    context.prepaid_gas = prepaid_gas;
    let config = VMConfig::default();
    let fees = RuntimeFeesConfig::default();

    let promise_results = vec![];

    let mut hash = DefaultHasher::new();
    code.hash(&mut hash);
    let code_hash = hash.finish().to_le_bytes().to_vec();
    run_vm(
        code_hash,
        code,
        method_name,
        &mut fake_external,
        context,
        &config,
        &fees,
        &promise_results,
        vm_kind,
        LATEST_PROTOCOL_VERSION,
        None,
    )
}

pub fn make_simple_contract_call_with_gas(
    code: &[u8],
    method_name: &[u8],
    prepaid_gas: u64,
) -> (Option<VMOutcome>, Option<VMError>) {
    make_simple_contract_call_with_gas_vm(code, method_name, prepaid_gas, VMKind::default())
}

pub fn make_simple_contract_call(
    code: &[u8],
    method_name: &[u8],
) -> (Option<VMOutcome>, Option<VMError>) {
    make_simple_contract_call_with_gas(code, method_name, 10u64.pow(14))
}

pub fn make_simple_contract_call_vm(
    code: &[u8],
    method_name: &[u8],
    vm_kind: VMKind,
) -> (Option<VMOutcome>, Option<VMError>) {
    make_simple_contract_call_with_gas_vm(code, method_name, 10u64.pow(14), vm_kind)
}

pub fn wat2wasm_no_validate(wat: &str) -> Vec<u8> {
    Wat2Wasm::new().validate(false).convert(wat).unwrap().as_ref().to_vec()
}

pub struct MockCompiledContractCache {
    pub store: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl CompiledContractCache for MockCompiledContractCache {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
        self.store.lock().unwrap().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
        match self.store.lock().unwrap().get(key) {
            Some(value) => Ok(Some(value.clone())),
            None => Ok(None),
        }
    }
}

pub fn make_cached_contract_call_vm(
    cache: &mut dyn CompiledContractCache,
    code: &[u8],
    method_name: &[u8],
    prepaid_gas: u64,
    vm_kind: VMKind,
) -> (Option<VMOutcome>, Option<VMError>) {
    let mut fake_external = MockedExternal::new();
    let mut context = create_context(vec![]);
    let config = VMConfig::default();
    let fees = RuntimeFeesConfig::default();
    let promise_results = vec![];
    context.prepaid_gas = prepaid_gas;
    let mut hash = DefaultHasher::new();
    code.hash(&mut hash);
    let code_hash = hash.finish().to_le_bytes().to_vec();

    run_vm(
        code_hash,
        &code,
        method_name,
        &mut fake_external,
        context.clone(),
        &config,
        &fees,
        &promise_results,
        vm_kind,
        LATEST_PROTOCOL_VERSION,
        Some(cache),
    )
}
