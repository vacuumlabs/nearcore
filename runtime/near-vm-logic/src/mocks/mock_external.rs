use crate::dependencies::ExternalError;
use crate::types::{AccountId, Balance, Gas, PublicKey};
use crate::External;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Range;
use std::collections::{BTreeMap, HashMap};
use std::intrinsics::transmute;

/// Encapsulates fake iterator. Optionally stores, if this iterator is built from prefix.
struct FakeIterator {
    iterator: Range<'static, Vec<u8>, Vec<u8>>,
    prefix: Option<Vec<u8>>,
}

#[derive(Default)]
/// Emulates the trie and the mock handling code.
pub struct MockedExternal {
    pub fake_trie: BTreeMap<Vec<u8>, Vec<u8>>,
    iterators: HashMap<u64, FakeIterator>,
    next_iterator_index: u64,
    receipts: Vec<Receipt>,
}

impl MockedExternal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get calls to receipt create that were performed during contract call.
    pub fn get_receipt_create_calls(&self) -> &Vec<Receipt> {
        &self.receipts
    }
}

impl External for MockedExternal {
    fn storage_set(&mut self, key: &[u8], value: &[u8]) -> Result<Option<Vec<u8>>, ExternalError> {
        Ok(self.fake_trie.insert(key.to_vec(), value.to_vec()))
    }

    fn storage_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ExternalError> {
        Ok(self.fake_trie.get(key).cloned())
    }

    fn storage_remove(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, ExternalError> {
        Ok(self.fake_trie.remove(key))
    }

    fn storage_has_key(&mut self, key: &[u8]) -> Result<bool, ExternalError> {
        Ok(self.fake_trie.contains_key(key))
    }

    fn storage_iter(&mut self, prefix: &[u8]) -> Result<u64, ExternalError> {
        let res = self.next_iterator_index;
        let iterator = self.fake_trie.range(prefix.to_vec()..);
        let iterator = unsafe {
            transmute::<Range<'_, Vec<u8>, Vec<u8>>, Range<'static, Vec<u8>, Vec<u8>>>(iterator)
        };
        self.iterators.insert(
            self.next_iterator_index,
            FakeIterator { iterator, prefix: Some(prefix.to_vec()) },
        );
        self.next_iterator_index += 1;
        Ok(res)
    }

    fn storage_iter_range(&mut self, start: &[u8], end: &[u8]) -> Result<u64, ExternalError> {
        let res = self.next_iterator_index;
        let iterator = self.fake_trie.range(start.to_vec()..end.to_vec());
        let iterator = unsafe {
            transmute::<Range<'_, Vec<u8>, Vec<u8>>, Range<'static, Vec<u8>, Vec<u8>>>(iterator)
        };
        self.iterators.insert(self.next_iterator_index, FakeIterator { iterator, prefix: None });
        self.next_iterator_index += 1;
        Ok(res)
    }

    fn storage_iter_next(
        &mut self,
        iterator_idx: u64,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>, ExternalError> {
        match self.iterators.get_mut(&iterator_idx) {
            Some(FakeIterator { iterator, prefix }) => match iterator.next() {
                Some((k, v)) => {
                    if let Some(prefix) = prefix {
                        if k.starts_with(prefix) {
                            Ok(Some((k.clone(), v.clone())))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(Some((k.clone(), v.clone())))
                    }
                }
                None => Ok(None),
            },
            None => Err(ExternalError::InvalidIteratorIndex),
        }
    }

    fn storage_iter_drop(&mut self, iterator_idx: u64) -> Result<(), ExternalError> {
        if self.iterators.remove(&iterator_idx).is_none() {
            Err(ExternalError::InvalidIteratorIndex)
        } else {
            Ok(())
        }
    }

    fn create_receipt(
        &mut self,
        receipt_indices: Vec<u64>,
        receiver_id: String,
    ) -> Result<u64, ExternalError> {
        if receipt_indices.iter().any(|el| *el >= self.receipts.len() as u64) {
            return Err(ExternalError::InvalidReceiptIndex);
        }
        let res = self.receipts.len() as u64;
        self.receipts.push(Receipt { receipt_indices, receiver_id, actions: vec![] });
        Ok(res)
    }

    fn append_action_create_account(&mut self, receipt_index: u64) -> Result<(), ExternalError> {
        self.receipts.get_mut(receipt_index as usize).unwrap().actions.push(Action::CreateAccount);
        Ok(())
    }

    fn append_action_deploy_contract(
        &mut self,
        receipt_index: u64,
        code: Vec<u8>,
    ) -> Result<(), ExternalError> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::DeployContract(DeployContractAction { code }));
        Ok(())
    }

    fn append_action_function_call(
        &mut self,
        receipt_index: u64,
        method_name: Vec<u8>,
        arguments: Vec<u8>,
        attached_deposit: u128,
        prepaid_gas: u64,
    ) -> Result<(), ExternalError> {
        self.receipts.get_mut(receipt_index as usize).unwrap().actions.push(Action::FunctionCall(
            FunctionCallAction {
                method_name,
                args: arguments,
                deposit: attached_deposit,
                gas: prepaid_gas,
            },
        ));
        Ok(())
    }

    fn append_action_transfer(
        &mut self,
        receipt_index: u64,
        amount: u128,
    ) -> Result<(), ExternalError> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::Transfer(TransferAction { deposit: amount }));
        Ok(())
    }

    fn append_action_stake(
        &mut self,
        receipt_index: u64,
        stake: u128,
        public_key: Vec<u8>,
    ) -> Result<(), ExternalError> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::Stake(StakeAction { stake, public_key }));
        Ok(())
    }

    fn append_action_add_key_with_full_access(
        &mut self,
        receipt_index: u64,
        public_key: Vec<u8>,
        nonce: u64,
    ) -> Result<(), ExternalError> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::AddKeyWithFullAccess(AddKeyWithFullAccessAction { public_key, nonce }));
        Ok(())
    }

    fn append_action_add_key_with_function_call(
        &mut self,
        receipt_index: u64,
        public_key: Vec<u8>,
        nonce: u64,
        allowance: Option<u128>,
        receiver_id: String,
        method_names: Vec<Vec<u8>>,
    ) -> Result<(), ExternalError> {
        self.receipts.get_mut(receipt_index as usize).unwrap().actions.push(
            Action::AddKeyWithFunctionCall(AddKeyWithFunctionCallAction {
                public_key,
                nonce,
                allowance,
                receiver_id,
                method_names,
            }),
        );
        Ok(())
    }

    fn append_action_delete_key(
        &mut self,
        receipt_index: u64,
        public_key: Vec<u8>,
    ) -> Result<(), ExternalError> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::DeleteKey(DeleteKeyAction { public_key }));
        Ok(())
    }

    fn append_action_delete_account(
        &mut self,
        receipt_index: u64,
        beneficiary_id: String,
    ) -> Result<(), ExternalError> {
        self.receipts
            .get_mut(receipt_index as usize)
            .unwrap()
            .actions
            .push(Action::DeleteAccount(DeleteAccountAction { beneficiary_id }));
        Ok(())
    }

    fn sha256(&self, data: &[u8]) -> Result<Vec<u8>, ExternalError> {
        let value_hash = sodiumoxide::crypto::hash::sha256::hash(data);
        Ok(value_hash.as_ref().to_vec())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Receipt {
    receipt_indices: Vec<u64>,
    receiver_id: String,
    actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Action {
    CreateAccount,
    DeployContract(DeployContractAction),
    FunctionCall(FunctionCallAction),
    Transfer(TransferAction),
    Stake(StakeAction),
    AddKeyWithFullAccess(AddKeyWithFullAccessAction),
    AddKeyWithFunctionCall(AddKeyWithFunctionCallAction),
    DeleteKey(DeleteKeyAction),
    DeleteAccount(DeleteAccountAction),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeployContractAction {
    pub code: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionCallAction {
    #[serde(with = "crate::serde_with::bytes_as_str")]
    method_name: Vec<u8>,
    #[serde(with = "crate::serde_with::bytes_as_str")]
    args: Vec<u8>,
    gas: Gas,
    deposit: Balance,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransferAction {
    deposit: Balance,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StakeAction {
    stake: Balance,
    #[serde(with = "crate::serde_with::bytes_as_base58")]
    public_key: PublicKey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddKeyWithFullAccessAction {
    #[serde(with = "crate::serde_with::bytes_as_base58")]
    public_key: PublicKey,
    nonce: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddKeyWithFunctionCallAction {
    #[serde(with = "crate::serde_with::bytes_as_base58")]
    public_key: PublicKey,
    nonce: u64,
    allowance: Option<Balance>,
    receiver_id: AccountId,
    #[serde(with = "crate::serde_with::vec_bytes_as_str")]
    method_names: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteKeyAction {
    #[serde(with = "crate::serde_with::bytes_as_base58")]
    public_key: PublicKey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteAccountAction {
    beneficiary_id: AccountId,
}