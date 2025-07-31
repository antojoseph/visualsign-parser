use crate::commands::utils::{get_index, Coin};
use crate::commands::visualizers::CommandVisualizer;
use crate::visualiser::helper_field::{create_address_field, create_amount_field, create_text_field};

use sui_types::base_types::{ObjectID, SuiAddress};

use move_core_types::runtime_value::MoveValue;

use sui_json::{MoveTypeLayout, SuiJsonValue};
use sui_json_rpc_types::{SuiArgument, SuiCallArg, SuiCommand};

use visualsign::{
    SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldListLayout,
};
pub struct CetusAmmVisualizer;

impl CommandVisualizer for CetusAmmVisualizer {
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

        if pwc.function.contains("swap_b2a") {
            // We can't receive the token amount from the tx data
            let token1_amount = get_token1_amount(inputs, &pwc.arguments).unwrap_or_default();
            let token1_coin = get_token_1_coin(&pwc.type_arguments).unwrap_or_default();
            let token2_coin = get_token_2_coin(&pwc.type_arguments).unwrap_or_default();

            return Some(SignablePayloadField::ListLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: "CetusAMM Swap Command".to_string(),
                    label: "CetusAMM Swap Command".to_string(),
                },
                list_layout: SignablePayloadFieldListLayout {
                    fields: vec![
                        create_address_field("From", &sender.to_string(), None, None, None, None),
                        create_address_field("To", &sender.to_string(), None, None, None, None),
                        create_amount_field("Coin 1 Amount", &token1_amount.to_string(), &token1_coin.get_label()),
                        create_text_field("Coin 1", &token1_coin.get_label()),
                        create_text_field("Coin 2", &token2_coin.get_label()),
                    ],
                },
            });
        }

        None
    }

    fn can_handle(&self, command: &SuiCommand) -> bool {
        if let SuiCommand::MoveCall(pwc) = command {
            pwc.package
                == ObjectID::from_hex_literal(
                    "0xb2db7142fa83210a7d78d9c12ac49c043b3cbbd482224fea6e3da00aa5a5ae2d",
                )
                .unwrap()
                && pwc.function.contains("swap_b2a")
        } else {
            false
        }
    }
}

fn get_token_1_coin(type_args: &Vec<String>) -> Option<Coin> {
    if type_args.len() == 0 {
        return None;
    }

    Some(Coin::from_string(&type_args[0]))
}

fn get_token_2_coin(type_args: &Vec<String>) -> Option<Coin> {
    if type_args.len() == 1 {
        return None;
    }

    Some(Coin::from_string(&type_args[1]))
}

fn get_token1_amount(
    inputs: &Vec<SuiCallArg>,
    args: &[SuiArgument],
) -> Option<u64> {
    // TODO: Failed to deconstruct inputs, receive result below. We need to fix tx data decoding
    // Pure(
    //     SuiPureValue {
    //         value_type: None, // HERE! it is not U64
    //         value: [184,198,192,1,0,0,0,0],
    //     },
    // ),
    let amount_input = inputs.get(get_index(&args, Some(5))? as usize)?;
    let Ok(MoveValue::U64(decoded_value)) =
        SuiJsonValue::to_move_value(&amount_input.pure()?.to_json_value(), &MoveTypeLayout::U64)
    else {
        return None;
    };
    Some(decoded_value)
}
