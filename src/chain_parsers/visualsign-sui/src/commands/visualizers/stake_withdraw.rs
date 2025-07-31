use crate::commands::utils::get_index;
use crate::commands::visualizers::CommandVisualizer;
use crate::visualiser::helper_field::{
    create_address_field, create_amount_field,
};

use sui_types::base_types::{ObjectID, SuiAddress};

use move_core_types::runtime_value::MoveValue;

use sui_json::{MoveTypeLayout, SuiJsonValue};
use sui_json_rpc_types::{SuiArgument, SuiCallArg, SuiCommand};

use visualsign::{
    SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldListLayout,
};

/// Visualizer for staking and withdraw commands
pub struct StakeWithdrawVisualizer;

impl CommandVisualizer for StakeWithdrawVisualizer {
    fn visualize_tx_commands(
        &self,
        sender: &SuiAddress,
        command_index: &usize,
        commands: &Vec<SuiCommand>,
        inputs: &Vec<SuiCallArg>,
    ) -> Option<SignablePayloadField> {
        let Some(SuiCommand::MoveCall(pwc)) = commands.get(*command_index) else {
            return None;
        };

        if pwc.function.contains("add_stake") {
            let amount = get_stake_amount(commands, inputs, &pwc.arguments).unwrap_or_default();
            let receiver = get_stake_receiver(inputs, &pwc.arguments).unwrap_or_default();

            return Some(SignablePayloadField::ListLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: "Stake Command".to_string(),
                    label: "Stake Command".to_string(),
                },
                list_layout: SignablePayloadFieldListLayout {
                    fields: vec![
                        create_address_field("From", &sender.to_string(), None, None, None, None),
                        create_address_field(
                            "Validator",
                            &receiver.to_string(),
                            None,
                            None,
                            None,
                            None,
                        ),
                        create_amount_field("Amount", &amount.to_string(), "MIST"),
                    ],
                },
            });
        }

        if pwc.function.contains("withdraw_stake") {
            // TODO: from the TX data impossible to receive the Validator address and withdraw amount
            return Some(SignablePayloadField::ListLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: "Withdraw Command".to_string(),
                    label: "Withdraw Command".to_string(),
                },
                list_layout: SignablePayloadFieldListLayout {
                    fields: vec![create_address_field(
                        "From",
                        &sender.to_string(),
                        None,
                        None,
                        None,
                        None,
                    )],
                },
            });
        }

        None
    }

    fn can_handle(&self, command: &SuiCommand) -> bool {
        if let SuiCommand::MoveCall(pwc) = command {
            pwc.package == ObjectID::from_hex_literal("0x3").unwrap()
                && (pwc.function.contains("add_stake") || pwc.function.contains("withdraw_stake"))
        } else {
            false
        }
    }
}

fn get_stake_receiver(inputs: &Vec<SuiCallArg>, args: &[SuiArgument]) -> Option<SuiAddress> {
    let receiver_input = inputs.get(get_index(&args, Some(args.len() - 1))? as usize)?;

    receiver_input.pure()?.to_sui_address().ok()
}

fn get_stake_amount(
    commands: &Vec<SuiCommand>,
    inputs: &Vec<SuiCallArg>,
    args: &[SuiArgument],
) -> Option<u64> {
    let result_command = commands.get(get_index(&args, Some(1))? as usize)?;

    match result_command {
        SuiCommand::SplitCoins(_, input_coin_args) => {
            let amount_arg = inputs.get(get_index(input_coin_args, Some(0))? as usize)?;
            let Ok(MoveValue::U64(decoded_value)) = SuiJsonValue::to_move_value(
                &amount_arg.pure()?.to_json_value(),
                &MoveTypeLayout::U64,
            ) else {
                return None;
            };
            Some(decoded_value)
        }
        _ => None,
    }
}
