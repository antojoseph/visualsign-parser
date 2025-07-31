use sui_json_rpc_types::{
    SuiTransactionBlockData, SuiTransactionBlockDataAPI, SuiTransactionBlockKind,
};

/// Determine transaction title based on type
pub fn determine_transaction_type_string(tx_data: &SuiTransactionBlockData) -> String {
    match &tx_data.transaction() {
        SuiTransactionBlockKind::ProgrammableTransaction(_) => "Programmable Transaction",
        SuiTransactionBlockKind::ChangeEpoch(_) => "Change Epoch",
        SuiTransactionBlockKind::Genesis(_) => "Genesis Transaction",
        SuiTransactionBlockKind::ConsensusCommitPrologue(_) => "Consensus Commit",
        SuiTransactionBlockKind::AuthenticatorStateUpdate(_) => "Authenticator State Update",
        SuiTransactionBlockKind::RandomnessStateUpdate(_) => "Randomness State Update",
        SuiTransactionBlockKind::EndOfEpochTransaction(_) => "End of Epoch Transaction",
        SuiTransactionBlockKind::ConsensusCommitPrologueV2(_) => "Consensus Commit Prologue V2",
        SuiTransactionBlockKind::ConsensusCommitPrologueV3(_) => "Consensus Commit Prologue V3",
        SuiTransactionBlockKind::ConsensusCommitPrologueV4(_) => "Consensus Commit Prologue V4",
        SuiTransactionBlockKind::ProgrammableSystemTransaction(_) => {
            "Programmable System Transaction"
        }
    }
    .to_string()
}
