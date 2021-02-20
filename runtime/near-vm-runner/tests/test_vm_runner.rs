use near_primitives::hash::hash;
use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_vm_errors::{FunctionCallError, HostError, VMError};
use near_vm_logic::mocks::mock_external::MockedExternal;
use near_vm_logic::types::ReturnData;
use near_vm_logic::{External, VMConfig, VMContext, VMKind};
use near_vm_runner::{preload_vm_calls, run_preloaded, ContractCallPrepareRequest, ContractCallContext};

use crate::test_utils::{LATEST_PROTOCOL_VERSION};

pub mod test_utils;

const TEST_CONTRACT_1: &'static [u8] = include_bytes!("../tests/res/test_contract_rs.wasm");
const TEST_CONTRACT_2: &'static [u8] = include_bytes!("../tests/res/test_contract_ts.wasm");

fn default_vm_context() -> VMContext {
    return VMContext {
        current_account_id: "alice".to_string(),
        signer_account_id: "bob".to_string(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "carol".to_string(),
        input: vec![],
        block_index: 1,
        block_timestamp: 1586796191203000000,
        account_balance: 10u128.pow(25),
        account_locked_balance: 0,
        storage_usage: 100,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view: false,
        output_data_receivers: vec![],
        epoch_height: 1,
    };
}

#[test]
pub fn test_vm_runner() {
    let code1 = TEST_CONTRACT_1;
    let code2 = TEST_CONTRACT_2;

    let mut fake_external = MockedExternal::new();

    let context = default_vm_context();
    let config = VMConfig::default();
    let fees = RuntimeFeesConfig::default();
    let promise_results = vec![];
    let mut requests = Vec::new();
    let call_context = ContractCallContext::new(2);
    for _ in 0..3 {
        requests.push(ContractCallPrepareRequest {
            code_hash: hash(code1),
            code: code1.to_vec(),
            method_name: b"log_something".to_vec(),
        });
        requests.push(ContractCallPrepareRequest {
            code_hash: hash(code2),
            code: code2.to_vec(),
            method_name: b"log_something".to_vec(),
        });
    }
    // TODO: provide meaningful cache ASAP.
    let calls = preload_vm_calls(call_context.clone(), requests, None, &config, VMKind::Wasmer);
    for prepared in &calls {
        println!("calling {:?}", prepared.handle);
        let result = run_preloaded(
            call_context.clone(),
            prepared,
            &mut fake_external,
            context.clone(),
            &config,
            &fees,
            &promise_results,
            LATEST_PROTOCOL_VERSION,
            None,
        );
        println!("result is {:?}", result);
    }
}
