use crate::commands::visualizers::{
    CommandVisualizer, StakeWithdrawVisualizer, TokenTransferVisualizer, CetusAmmVisualizer, visualize_with_any,
};

use sui_json_rpc_types::{
    SuiCallArg, SuiCommand, SuiTransactionBlockData, SuiTransactionBlockDataAPI,
    SuiTransactionBlockKind,
};
use visualsign::SignablePayloadField;

pub fn add_tx_commands(
    fields: &mut Vec<SignablePayloadField>,
    block_data: &SuiTransactionBlockData,
) {
    let mut commands_fields: Vec<SignablePayloadField> = vec![];

    let tx_inputs: &Vec<SuiCallArg> = match &block_data.transaction() {
        SuiTransactionBlockKind::ProgrammableTransaction(tx) => &tx.inputs,
        _ => return,
    };

    let tx_commands: &Vec<SuiCommand> = match &block_data.transaction() {
        SuiTransactionBlockKind::ProgrammableTransaction(tx) => &tx.commands,
        _ => return,
    };

    let sender = *block_data.sender();

    // Create instances of all visualizers
    let token_visualizer = TokenTransferVisualizer;
    let stake_visualizer = StakeWithdrawVisualizer;
    let cetus_amm_visualizer = CetusAmmVisualizer;

    // List of all available visualizers
    let visualizers: &[&dyn CommandVisualizer] = &[&token_visualizer, &stake_visualizer, &cetus_amm_visualizer];

    // Process each command with the appropriate visualizer
    for (command_index, _) in tx_commands.iter().enumerate() {
        if let Some(field) = visualize_with_any(
            visualizers,
            &sender,
            &command_index,
            &tx_commands,
            &tx_inputs,
        ) {
            commands_fields.push(field);
        }
    }

    // println!("Commands fields: {:#?}", commands_fields);

    fields.extend(commands_fields);
}
