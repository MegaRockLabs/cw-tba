use cosmwasm_std::ensure;
use cw_tba::RegistryParams;
use crate::error::ContractError;



pub fn validate_params(
    params: &RegistryParams,
) -> Result<(), ContractError> {
    ensure!(!params.allowed_code_ids.is_empty(), ContractError::InvalidCodeIds {});
    ensure!(!params.creation_fees.is_empty(), ContractError::InvalidCreationFees {});
    ensure!(!params.creation_fees.iter().any(|c| c.amount.is_zero()), ContractError::InvalidCreationFees {});
    Ok(())
}