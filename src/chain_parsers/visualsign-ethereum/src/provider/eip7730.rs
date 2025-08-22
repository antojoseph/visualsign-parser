use crate::registry::{CommandVisualizer, VisualizerContext, decode_calldata};
use visualsign::{SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldTextV2};

pub struct Eip7730Visualizer;

impl CommandVisualizer for Eip7730Visualizer {
    fn visualize_tx_commands(&self, context: &VisualizerContext) -> Option<SignablePayloadField> {
        let decoded = decode_calldata(context.calldata)?;
        if decoded.is_empty() {
            return None;
        }
        if decoded.len() == 1 {
            // A single decoded field is already suitably granular.
            return Some(decoded.into_iter().next().unwrap());
        }
        // Summarize multiple decoded fields by listing their labels.
        let labels: Vec<String> = decoded.iter().map(|f| f.label().to_string()).collect();
        let summary_text = format!("Decoded Input Fields: {}", labels.join(", "));
        Some(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: summary_text.clone(),
                label: "Decoded Input".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text: summary_text },
        })
    }
}
