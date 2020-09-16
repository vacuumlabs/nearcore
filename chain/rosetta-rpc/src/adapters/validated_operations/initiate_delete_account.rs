use super::ValidatedOperation;

pub(crate) struct InitiateDeleteAccountOperation {
    pub(crate) sender_account: crate::models::AccountIdentifier,
}

impl super::ValidatedOperation for InitiateDeleteAccountOperation {
    const OPERATION_TYPE: crate::models::OperationType =
        crate::models::OperationType::InitiateDeleteAccount;

    fn into_operation(
        self,
        operation_identifier: crate::models::OperationIdentifier,
    ) -> crate::models::Operation {
        crate::models::Operation {
            operation_identifier,

            account: self.sender_account,
            amount: None,
            metadata: None,

            related_operations: None,
            type_: Self::OPERATION_TYPE,
            status: crate::models::OperationStatusKind::Success,
        }
    }
}

impl std::convert::TryFrom<crate::models::Operation> for InitiateDeleteAccountOperation {
    type Error = crate::errors::ErrorKind;

    fn try_from(operation: crate::models::Operation) -> Result<Self, Self::Error> {
        Self::validate_operation_type(operation.type_)?;
        Ok(Self { sender_account: operation.account })
    }
}