mod config;

use config::{CETUS_CONFIG, Config, PoolScriptV2Functions, SwapA2BIndexes, SwapB2AIndexes};

use crate::core::{CommandVisualizer, SuiIntegrationConfig, VisualizerContext, VisualizerKind};
use crate::utils::{SuiCoin, get_tx_type_arg, truncate_address};

use sui_json_rpc_types::{SuiCommand, SuiProgrammableMoveCall};

use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon,
    SignablePayloadFieldListLayout, SignablePayloadFieldPreviewLayout, SignablePayloadFieldTextV2,
    errors::VisualSignError,
    field_builders::{create_address_field, create_amount_field, create_text_field},
};

pub struct CetusVisualizer;

impl CommandVisualizer for CetusVisualizer {
    fn visualize_tx_commands(
        &self,
        context: &VisualizerContext,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let Some(SuiCommand::MoveCall(pwc)) = context.commands().get(context.command_index())
        else {
            return Err(VisualSignError::MissingData(
                "Expected a `MoveCall` for Cetus parsing".into(),
            ));
        };

        let function = match pwc.function.as_str().try_into() {
            Ok(function) => function,
            Err(e) => return Err(VisualSignError::DecodeError(e)),
        };

        match function {
            PoolScriptV2Functions::SwapB2A => self.handle_swap(false, context, pwc),
            PoolScriptV2Functions::SwapA2B => self.handle_swap(true, context, pwc),
        }
    }

    fn get_config(&self) -> Option<&dyn SuiIntegrationConfig> {
        Some(CETUS_CONFIG.get_or_init(Config::new))
    }

    fn kind(&self) -> VisualizerKind {
        VisualizerKind::Dex("Cetus")
    }
}

impl CetusVisualizer {
    fn handle_swap(
        &self,
        is_a2b: bool,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let (input_coin, output_coin): (SuiCoin, SuiCoin) = if is_a2b {
            (
                get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default(),
                get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default(),
            )
        } else {
            (
                get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default(),
                get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default(),
            )
        };

        let (by_amount_in, amount, amount_limit) = if is_a2b {
            (
                SwapA2BIndexes::get_by_amount_in(context.inputs(), &pwc.arguments)?,
                SwapA2BIndexes::get_amount(context.inputs(), &pwc.arguments)?,
                SwapA2BIndexes::get_amount_limit(context.inputs(), &pwc.arguments)?,
            )
        } else {
            (
                SwapB2AIndexes::get_by_amount_in(context.inputs(), &pwc.arguments)?,
                SwapB2AIndexes::get_amount(context.inputs(), &pwc.arguments)?,
                SwapB2AIndexes::get_amount_limit(context.inputs(), &pwc.arguments)?,
            )
        };

        let (primary_label, primary_symbol, limit_label, limit_symbol) = if by_amount_in {
            (
                "Amount In",
                input_coin.symbol(),
                "Min Out",
                output_coin.symbol(),
            )
        } else {
            (
                "Amount Out",
                output_coin.symbol(),
                "Max In",
                input_coin.symbol(),
            )
        };

        let list_layout_fields = vec![
            create_address_field(
                "User Address",
                &context.sender().to_string(),
                None,
                None,
                None,
                None,
            )?,
            create_amount_field(primary_label, &amount.to_string(), primary_symbol)?,
            create_text_field("Input Coin", &input_coin.to_string())?,
            create_amount_field(limit_label, &amount_limit.to_string(), limit_symbol)?,
            create_text_field("Output Coin", &output_coin.to_string())?,
        ];

        {
            let title_text = if by_amount_in {
                format!(
                    "CetusAMM Swap: {} {} → {}",
                    amount,
                    input_coin.symbol(),
                    output_coin.symbol()
                )
            } else {
                format!(
                    "CetusAMM Swap: {} {} ← {}",
                    amount,
                    output_coin.symbol(),
                    input_coin.symbol()
                )
            };
            let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

            let condensed = SignablePayloadFieldListLayout {
                fields: vec![create_text_field(
                    "Summary",
                    &format!(
                        "Swap {} to {} ({}: {})",
                        input_coin.symbol(),
                        output_coin.symbol(),
                        limit_label,
                        amount_limit
                    ),
                )?],
            };

            let expanded = SignablePayloadFieldListLayout {
                fields: list_layout_fields,
            };

            Ok(vec![AnnotatedPayloadField {
                static_annotation: None,
                dynamic_annotation: None,
                signable_payload_field: SignablePayloadField::PreviewLayout {
                    common: SignablePayloadFieldCommon {
                        fallback_text: title_text.clone(),
                        label: "CetusAMM Swap Command".to_string(),
                    },
                    preview_layout: SignablePayloadFieldPreviewLayout {
                        title: Some(SignablePayloadFieldTextV2 { text: title_text }),
                        subtitle: Some(SignablePayloadFieldTextV2 {
                            text: subtitle_text,
                        }),
                        condensed: Some(condensed),
                        expanded: Some(expanded),
                    },
                },
            }])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::payload_from_b64;

    use visualsign::test_utils::{
        assert_has_field, assert_has_field_with_context, assert_has_field_with_value,
        assert_has_field_with_value_with_context,
    };

    const CETUS_SWAP_LABEL: &str = "CetusAMM Swap Command";

    #[test]
    fn test_cetus_amm_swap_b2a_commands() {
        // https://suivision.xyz/txblock/7Je4yeXMvvEHFcRSTD4WYv3eSsaDk2zqvdoSxWXdUYGx
        let test_data = "AQAAAAAACQEAEXs/ewhS1RZrUZQ2xQEliCJn40SK4PvEV75r2SGFMXhjUsAjAAAAACBSKqlrLdPXYeuzckz31NAkeSO09qmNPv/pkWggJMTC2QAIuMbAAQAAAAABAdqkYpJjLDxNjzHyPqD5s2oo/zZ36WhJgORDhAOmej2PLgUYAAAAAAAAAQFK94o+ni1sq8pdp5wea/9ImVZqQhMh/DtaYZZkAXpg1nkOqBoAAAAAAQABAQAIuMbAAQAAAAAACI0+GgMAAAAAABCvMxuoMn+7NbHE/v8AAAAAAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAMCAQAAAQEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgRjb2luBHplcm8BB9ujRnLjDLBlsfk+OrVTGHaP1v72bBWULJ98uEbi+QDnBHVzZGMEVVNEQwAAALLbcUL6gyEKfXjZwSrEnAQ7PLvUgiJP6m49oAqlpa4tDnBvb2xfc2NyaXB0X3YyCHN3YXBfYjJhAgfbo0Zy4wywZbH5Pjq1Uxh2j9b+9mwVlCyffLhG4vkA5wR1c2RjBFVTREMAB7eETiiahBDlD7PKSNaeuc8p4n0iPvkDU/4b2OJ/+PP4BGNvaW4EQ09JTgAJAQIAAQMAAgEAAgAAAQQAAQUAAQYAAQcAAQgArltnUkfA5IdctLm9N6YO1bz4kng0TThA3StCbiinZoUBZI8YcdbCiGOtIFCZV/M9U6lZTgf3lg6t7feHRsBBqR1jUsAjAAAAACCmwR6aeqn8D632smpzU9fbDhP3vPOQhgc806IrzekPH65bZ1JHwOSHXLS5vTemDtW8+JJ4NE04QN0rQm4op2aFBQIAAAAAAAC8YDQAAAAAAAABYQAdbFpPHuOPe/TYRMttj4FSzAN1ErZdI75GooTkFmiIVkvCM+lnSS3pR/qQt6j7K3gsrtBExfgOL/dffWapvuMEyeP1ig9kZWEaY4lMw99QxRTo2PcUhKsb1gquOOAGXP8=";

        let payload = payload_from_b64(test_data);
        assert_has_field(&payload, CETUS_SWAP_LABEL);

        assert_has_field_with_value(
            &payload,
            "User Address",
            "0xae5b675247c0e4875cb4b9bd37a60ed5bcf89278344d3840dd2b426e28a76685",
        );
        assert_has_field_with_value(&payload, "Amount In", "29411000");
        assert_has_field_with_value(
            &payload,
            "Input Coin",
            "0xb7844e289a8410e50fb3ca48d69eb9cf29e27d223ef90353fe1bd8e27ff8f3f8::coin::COIN",
        );
        assert_has_field_with_value(&payload, "Min Out", "52051597");
        assert_has_field_with_value(
            &payload,
            "Output Coin",
            "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",
        );
    }

    #[test]
    fn test_cetus_amm_swap_a2b_commands() {
        // https://suivision.xyz/txblock/7t6iLtevYDEpXrr3rhpmDcwf8cMMV1sgspppvvnXiguR
        let test_data = "AQAAAAAACgEAkfGWz0JGPLt14gdQVPgAPvGv100NtFt2InDcGDyMZQRIPXMkAAAAACBzf79a+nciTqmPBgQycQyP7VMyWjP2waulu8LKtlZ2ggEA4pyQKsylAKpoN702neQpT4smpbXaopiWRMOhnNQk+dxIPXMkAAAAACDPA2LUvkZAkhsDL9IAPA5XEMTFk44RZFMN/UrpVT0aOwAIqBEHZwAAAAABAdqkYpJjLDxNjzHyPqD5s2oo/zZ36WhJgORDhAOmej2PLgUYAAAAAAAAAQFR6IO6fAtWaibLyKlM0z6wq9QYp3zB5grSL9mx8pzSq/uacRYAAAAAAQABAAAIAIhSanQAAAAACKgRB2cAAAAAABBQOwEAAQAAAAAAAAAAAAAAAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAQDAQAAAQEBAAIBAAABAQIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACBGNvaW4EemVybwEHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAAALLbcUL6gyEKfXjZwSrEnAQ7PLvUgiJP6m49oAqlpa4tDnBvb2xfc2NyaXB0X3YyCHN3YXBfYTJiAgfbo0Zy4wywZbH5Pjq1Uxh2j9b+9mwVlCyffLhG4vkA5wR1c2RjBFVTREMABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA3N1aQNTVUkACQEDAAEEAAIBAAICAAEFAAEGAAEHAAEIAAEJABxoihUeyy/EpwFkjSZ1URYLuz/mqwyC1srAaO1syYLJAoo43JFE0wNBKCG9zE6pVWaENgEg5OfSW8ZgI9xfwZ0JSD1zJAAAAAAgHKrS7Xzyr+wSIwY1SfiwUh3kR/gsbnB5wy14YgB8JlUweXl6kiml0me3PakkjYuFIPJ+CJMElcVq6NGPtGy26Ug9cyQAAAAAINXWx8S5GTFIRWp1oY/IAkEhRVrywXZhCYVXXzVPcQ7fHGiKFR7LL8SnAWSNJnVRFgu7P+arDILWysBo7WzJgsn0AQAAAAAAAOhyLwAAAAAAAAFhAIB1hvQj0FnB2h+j3lZjYL1en1K3A7ITWXhVpj1Oslz0FVgkC3Es9xS5JGDgXByYelNgSJ4bFzB+Sn+9LwOJVAKAiyGXhnmh12WYynVXlQH2doDZ0v5LCrXENXPauhOQWQ==";

        let payload = payload_from_b64(test_data);
        assert_has_field(&payload, CETUS_SWAP_LABEL);

        assert_has_field_with_value(
            &payload,
            "User Address",
            "0x1c688a151ecb2fc4a701648d267551160bbb3fe6ab0c82d6cac068ed6cc982c9",
        );
        assert_has_field_with_value(&payload, "Max In", "1728516520");
        assert_has_field_with_value(
            &payload,
            "Input Coin",
            "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC",
        );
        assert_has_field_with_value(&payload, "Amount Out", "500000000000");
        assert_has_field_with_value(&payload, "Output Coin", "0x2::sui::SUI");
    }

    #[test]
    fn test_cetus_amm_aggregated() {
        use serde::Deserialize;
        use std::collections::HashMap;

        #[derive(Debug, Deserialize)]
        struct Operation {
            data: String,
            asserts: HashMap<String, String>,
        }

        #[derive(Debug, Deserialize)]
        struct Category {
            label: String,
            operations: HashMap<String, Operation>,
        }

        #[derive(Debug, Deserialize)]
        struct AggregatedTestData {
            explorer_tx_prefix: String,
            #[serde(flatten)]
            categories: HashMap<String, Category>,
        }

        let json_str = include_str!("aggregated_test_data.json");
        let data: AggregatedTestData =
            serde_json::from_str(json_str).expect("invalid aggregated_test_data.json");

        for (name, category) in data.categories.iter() {
            let label = &category.label;
            for (op_id, op) in category.operations.iter() {
                let payload = payload_from_b64(&op.data);
                let test_context = format!(
                    "Test name: {name}. Tx id: {}{op_id}",
                    data.explorer_tx_prefix
                );

                assert_has_field_with_context(&payload, label, &test_context);
                for (field, expected) in op.asserts.iter() {
                    assert_has_field_with_value_with_context(
                        &payload,
                        field,
                        expected,
                        &test_context,
                    );
                }
            }
        }
    }
}
