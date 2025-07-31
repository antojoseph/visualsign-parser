use sui_types::base_types::SuiAddress;
use sui_json_rpc_types::{SuiCommand, SuiCallArg};
use visualsign::SignablePayloadField;

/// Trait for visualizing Sui transaction commands
pub trait CommandVisualizer {
    /// Visualize a specific command in a transaction
    /// 
    /// # Arguments
    /// * `sender` - The address sending the transaction
    /// * `command_index` - Index of the command to visualize
    /// * `commands` - All commands in the transaction
    /// * `inputs` - All input arguments for the transaction
    /// 
    /// # Returns
    /// * `Some(SignablePayloadField)` if the command can be visualized
    /// * `None` if the command is not supported by this visualizer
    fn visualize_tx_commands(
        &self,
        sender: &SuiAddress,
        command_index: &usize,
        commands: &Vec<SuiCommand>,
        inputs: &Vec<SuiCallArg>,
    ) -> Option<SignablePayloadField>;

    /// Check if this visualizer can handle the given command
    fn can_handle(&self, command: &SuiCommand) -> bool;
}

/// Helper function to try multiple visualizers in order
pub fn visualize_with_any(
    visualizers: &[&dyn CommandVisualizer],
    sender: &SuiAddress,
    command_index: &usize,
    commands: &Vec<SuiCommand>,
    inputs: &Vec<SuiCallArg>,
) -> Option<SignablePayloadField> {
    let command = commands.get(*command_index)?;
    
    for visualizer in visualizers {
        if visualizer.can_handle(command) {
            if let Some(result) = visualizer.visualize_tx_commands(sender, command_index, commands, inputs) {
                return Some(result);
            }
        }
    }
    
    None
}