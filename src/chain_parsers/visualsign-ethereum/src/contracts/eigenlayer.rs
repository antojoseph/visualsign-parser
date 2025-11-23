use alloy_sol_types::{SolCall, sol};
use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldAddressV2,
    SignablePayloadFieldAmountV2, SignablePayloadFieldCommon, SignablePayloadFieldListLayout,
    SignablePayloadFieldPreviewLayout, SignablePayloadFieldTextV2,
};

// EigenLayer contract interfaces
// Based on: https://github.com/Layr-Labs/eigenlayer-contracts

sol! {
    // Strategy Manager - Core restaking contract
    interface IStrategyManager {
        function depositIntoStrategy(
            address strategy,
            address token,
            uint256 amount
        ) external returns (uint256 shares);

        function depositIntoStrategyWithSignature(
            address strategy,
            address token,
            uint256 amount,
            address staker,
            uint256 expiry,
            bytes memory signature
        ) external returns (uint256 shares);

        function addShares(address staker, address strategy, uint256 shares) external;

        function removeDepositShares(address staker, address strategy, uint256 depositSharesToRemove) external;

        function withdrawSharesAsTokens(address staker, address strategy, address token, uint256 shares) external;

        function addStrategiesToDepositWhitelist(address[] calldata strategiesToWhitelist) external;

        function removeStrategiesFromDepositWhitelist(address[] calldata strategiesToRemoveFromWhitelist) external;

        function setStrategyWhitelister(address newStrategyWhitelister) external;
    }

    // Delegation Manager - Operator delegation
    interface IDelegationManager {
        function delegateTo(
            address operator,
            bytes calldata approverSignatureAndExpiry,
            bytes32 approverSalt
        ) external;

        function undelegate(address staker) external returns (bytes32[] memory withdrawalRoots);

        function queueWithdrawals(
            QueuedWithdrawalParams[] calldata queuedWithdrawalParams
        ) external returns (bytes32[] memory);

        function completeQueuedWithdrawal(
            Withdrawal calldata withdrawal,
            address[] calldata tokens,
            uint256 middlewareTimesIndex,
            bool receiveAsTokens
        ) external;

        function completeQueuedWithdrawals(
            Withdrawal[] calldata withdrawals,
            address[][] calldata tokens,
            uint256[] calldata middlewareTimesIndexes,
            bool[] calldata receiveAsTokens
        ) external;

        function registerAsOperator(
            address initDelegationApprover,
            uint32 allocationDelay,
            string calldata metadataURI
        ) external;

        function modifyOperatorDetails(
            address operator,
            address newDelegationApprover
        ) external;

        function updateOperatorMetadataURI(
            address operator,
            string calldata metadataURI
        ) external;

        function redelegate(
            address newOperator,
            bytes calldata newOperatorApproverSig,
            bytes32 approverSalt
        ) external;

        function increaseDelegatedShares(
            address staker,
            address strategy,
            uint256 prevDepositShares,
            uint256 addedShares
        ) external;

        function decreaseDelegatedShares(
            address staker,
            uint256 curDepositShares,
            uint64 beaconChainSlashingFactorDecrease
        ) external;

        function slashOperatorShares(
            address operator,
            address[] calldata strategies,
            uint256[] calldata shareAmounts
        ) external;
    }

    struct QueuedWithdrawalParams {
        address[] strategies;
        uint256[] shares;
        address withdrawer;
    }

    struct Withdrawal {
        address staker;
        address delegatedTo;
        address withdrawer;
        uint256 nonce;
        uint32 startBlock;
        address[] strategies;
        uint256[] shares;
    }

    // AVS Directory - AVS registration
    interface IAVSDirectory {
        function registerOperatorToAVS(
            address operator,
            bytes calldata operatorSignature
        ) external;

        function deregisterOperatorFromAVS(address operator) external;

        function updateAVSMetadataURI(string calldata metadataURI) external;

        function cancelSalt(bytes32 salt) external;
    }

    // Rewards Coordinator - Rewards management
    interface IRewardsCoordinator {
        function createAVSRewardsSubmission(
            RewardsSubmission[] calldata rewardsSubmissions
        ) external;

        function createRewardsForAllSubmission(
            RewardsSubmission[] calldata rewardsSubmissions
        ) external;

        function processClaim(
            RewardsMerkleClaim calldata claim,
            address recipient
        ) external;
    }

    struct RewardsSubmission {
        address[] strategiesAndMultipliers;
        address token;
        uint256 amount;
        uint32 startTimestamp;
        uint32 duration;
    }

    struct RewardsMerkleClaim {
        uint32 rootIndex;
        uint32 earnerIndex;
        bytes earnerTreeProof;
        bytes32[] earnerLeaf;
        bytes32[] tokenIndices;
        bytes32[] tokenTreeProofs;
        bytes32[][] tokenLeaves;
    }

    // EigenPod Manager - Native ETH restaking
    interface IEigenPodManager {
        function createPod() external returns (address);

        function stake(bytes calldata pubkey, bytes calldata signature, bytes32 depositDataRoot) external payable;

        function addShares(address podOwner, uint256 shares) external;

        function removeDepositShares(address podOwner, uint256 depositSharesToRemove) external;

        function withdrawSharesAsTokens(address podOwner, address destination, uint256 shares) external;

        function recordBeaconChainETHBalanceUpdate(
            address podOwner,
            uint256 prevRestakedBalanceWei,
            int256 balanceDeltaWei
        ) external;

        function setPectraForkTimestamp(uint64 timestamp) external;

        function setProofTimestampSetter(address newProofTimestampSetter) external;
    }

    // Allocation Manager - Slashing and allocation
    interface IAllocationManager {
        function modifyAllocations(
            address operator,
            AllocationParams[] calldata allocationParams
        ) external;

        function clearDeallocationQueue(
            address operator,
            address[] calldata strategies,
            uint16[] calldata numToClear
        ) external;

        function registerForOperatorSets(
            address operator,
            RegisterParams calldata params
        ) external;

        function deregisterFromOperatorSets(
            DeregisterParams calldata params
        ) external;

        function createOperatorSets(
            address avs,
            CreateSetParams[] calldata params
        ) external;

        function slashOperator(
            address avs,
            SlashingParams calldata params
        ) external;

        function addStrategiesToOperatorSet(
            address avs,
            uint32 operatorSetId,
            address[] calldata strategies
        ) external;

        function removeStrategiesFromOperatorSet(
            address avs,
            uint32 operatorSetId,
            address[] calldata strategies
        ) external;

        function setAVSRegistrar(address avs, address registrar) external;

        function setAllocationDelay(address operator, uint32 delay) external;

        function updateAVSMetadataURI(address avs, string calldata metadataURI) external;
    }

    struct AllocationParams {
        address avs;
        address[] strategies;
        uint256[] newMagnitudes;
    }

    struct RegisterParams {
        address avs;
        uint32[] operatorSetIds;
        bytes data;
    }

    struct DeregisterParams {
        address operator;
        address avs;
        uint32[] operatorSetIds;
    }

    struct CreateSetParams {
        uint32 operatorSetId;
        address[] strategies;
    }

    struct SlashingParams {
        uint32 operatorSetId;
        address[] strategies;
        uint16[] wadsToSlash;
        string description;
    }

    // Rewards Coordinator - Extended interface
    interface IRewardsCoordinatorExtended {
        function processClaims(
            RewardsMerkleClaim[] calldata claims,
            address recipient
        ) external;

        function createRewardsForAllEarners(
            RewardsSubmission[] calldata rewardsSubmissions
        ) external;

        function submitRoot(
            bytes32 root,
            uint32 rewardsCalculationEndTimestamp
        ) external;

        function setClaimerFor(address claimer) external;

        function createOperatorDirectedAVSRewardsSubmission(
            address avs,
            RewardsSubmission[] calldata rewardsSubmissions
        ) external;

        function createOperatorDirectedOperatorSetRewardsSubmission(
            address avs,
            uint32 operatorSetId,
            RewardsSubmission[] calldata rewardsSubmissions
        ) external;

        function disableRoot(uint32 rootIndex) external;

        function setActivationDelay(uint32 _activationDelay) external;

        function setDefaultOperatorSplit(uint16 split) external;

        function setOperatorAVSSplit(address operator, address avs, uint16 split) external;

        function setOperatorPISplit(address operator, uint16 split) external;

        function setOperatorSetSplit(address avs, uint32 operatorSetId, uint16 split) external;

        function setRewardsForAllSubmitter(address _submitter, bool _newValue) external;

        function setRewardsUpdater(address _rewardsUpdater) external;
    }
}

// Known EigenLayer contract addresses on Ethereum Mainnet
pub struct KnownContracts;

impl KnownContracts {
    pub const DELEGATION_MANAGER: &'static str = "0x39053D51B77DC0d36036Fc1fCc8Cb819df8Ef37A";
    pub const STRATEGY_MANAGER: &'static str = "0x858646372CC42E1A627fcE94aa7A7033e7CF075A";
    pub const EIGENPOD_MANAGER: &'static str = "0x91E677b07F7AF907ec9a428aafA9fc14a0d3A338";
    pub const AVS_DIRECTORY: &'static str = "0x135dda560e946695d6f155dacafc6f1f25c1f5af";
    pub const REWARDS_COORDINATOR: &'static str = "0x7750d328b314EfFa365A0402CcfD489B80B0adda";
    pub const ALLOCATION_MANAGER: &'static str = "0x948a420b8CC1d6BFd0B6087C2E7c344a2CD0bc39";

    // Known strategies with human-readable names
    pub fn get_strategy_name(address: &str) -> Option<&'static str> {
        match address.to_lowercase().as_str() {
            "0x93c4b944d05dfe6df7645a86cd2206016c51564d" => Some("stETH Strategy"),
            "0x54945180db7943c0ed0fee7edab2bd24620256bc" => Some("cbETH Strategy"),
            "0x1bee69b7dfffa4e2d53c2a2df135c388ad25dcd2" => Some("rETH Strategy"),
            "0x9d7ed45ee2e8fc5482fa2428f15c971e6369011d" => Some("ETHx Strategy"),
            "0x13760f50a9d7377e4f20cb8cf9e4c26586c658ff" => Some("ankrETH Strategy"),
            "0xa4c637e0f704745d182e4d38cab7e7485321d059" => Some("OETH Strategy"),
            "0x57ba429517c3473b6d34ca9acd56c0e735b94c02" => Some("osETH Strategy"),
            "0x0fe4f44bee93503346a3ac9ee5a26b130a5796d6" => Some("swETH Strategy"),
            "0x7ca911e83dabf90c90dd3de5411a10f1a6112184" => Some("wBETH Strategy"),
            "0x8ca7a5d6f3acd3a7a8bc468a8cd0fb14b6bd28b6" => Some("sfrxETH Strategy"),
            "0xae60d8180437b5c34bb956822ac2710972584473" => Some("lsETH Strategy"),
            "0x298afb19a105d59e74658c4c334ff360bade6dd2" => Some("mETH Strategy"),
            "0xacb55c530acdb2849e6d4f36992cd8c9d50ed8f7" => Some("EIGEN Strategy"),
            "0xbeac0eeeeeeeeeeeeeeeeeeeeeeeeeeeeeebeac0" => Some("Beacon Chain ETH"),
            _ => None,
        }
    }
}

pub struct EigenLayerVisualizer {}

impl EigenLayerVisualizer {
    pub fn visualize_tx_commands(&self, input: &[u8]) -> Option<SignablePayloadField> {
        if input.len() < 4 {
            return None;
        }

        let selector = &input[..4];

        // Strategy Manager functions
        if selector == IStrategyManager::depositIntoStrategyCall::SELECTOR {
            return self.visualize_deposit_into_strategy(input);
        }

        if selector == IStrategyManager::depositIntoStrategyWithSignatureCall::SELECTOR {
            return self.visualize_deposit_into_strategy_with_signature(input);
        }

        // Delegation Manager functions
        if selector == IDelegationManager::delegateToCall::SELECTOR {
            return self.visualize_delegate_to(input);
        }

        if selector == IDelegationManager::undelegateCall::SELECTOR {
            return self.visualize_undelegate(input);
        }

        if selector == IDelegationManager::queueWithdrawalsCall::SELECTOR {
            return self.visualize_queue_withdrawals(input);
        }

        if selector == IDelegationManager::completeQueuedWithdrawalCall::SELECTOR {
            return self.visualize_complete_queued_withdrawal(input);
        }

        if selector == IDelegationManager::completeQueuedWithdrawalsCall::SELECTOR {
            return self.visualize_complete_queued_withdrawals(input);
        }

        if selector == IDelegationManager::registerAsOperatorCall::SELECTOR {
            return self.visualize_register_as_operator(input);
        }

        if selector == IDelegationManager::modifyOperatorDetailsCall::SELECTOR {
            return self.visualize_modify_operator_details(input);
        }

        if selector == IDelegationManager::updateOperatorMetadataURICall::SELECTOR {
            return self.visualize_update_operator_metadata(input);
        }

        if selector == IDelegationManager::redelegateCall::SELECTOR {
            return self.visualize_redelegate(input);
        }

        // AVS Directory functions
        if selector == IAVSDirectory::registerOperatorToAVSCall::SELECTOR {
            return self.visualize_register_operator_to_avs(input);
        }

        if selector == IAVSDirectory::deregisterOperatorFromAVSCall::SELECTOR {
            return self.visualize_deregister_operator_from_avs(input);
        }

        if selector == IAVSDirectory::updateAVSMetadataURICall::SELECTOR {
            return self.visualize_update_avs_metadata_uri(input);
        }

        // Rewards Coordinator functions
        if selector == IRewardsCoordinator::createAVSRewardsSubmissionCall::SELECTOR {
            return self.visualize_create_avs_rewards_submission(input);
        }

        if selector == IRewardsCoordinator::processClaimCall::SELECTOR {
            return self.visualize_process_claim(input);
        }

        if selector == IRewardsCoordinatorExtended::processClaimsCall::SELECTOR {
            return self.visualize_process_claims(input);
        }

        if selector == IRewardsCoordinatorExtended::createRewardsForAllEarnersCall::SELECTOR {
            return self.visualize_create_rewards_for_all_earners(input);
        }

        if selector == IRewardsCoordinatorExtended::submitRootCall::SELECTOR {
            return self.visualize_submit_root(input);
        }

        if selector == IRewardsCoordinatorExtended::setClaimerForCall::SELECTOR {
            return self.visualize_set_claimer_for(input);
        }

        // EigenPod Manager functions
        if selector == IEigenPodManager::createPodCall::SELECTOR {
            return self.visualize_create_pod(input);
        }

        if selector == IEigenPodManager::stakeCall::SELECTOR {
            return self.visualize_stake(input);
        }

        // Allocation Manager functions
        if selector == IAllocationManager::modifyAllocationsCall::SELECTOR {
            return self.visualize_modify_allocations(input);
        }

        if selector == IAllocationManager::registerForOperatorSetsCall::SELECTOR {
            return self.visualize_register_for_operator_sets(input);
        }

        if selector == IAllocationManager::deregisterFromOperatorSetsCall::SELECTOR {
            return self.visualize_deregister_from_operator_sets(input);
        }

        if selector == IAllocationManager::createOperatorSetsCall::SELECTOR {
            return self.visualize_create_operator_sets(input);
        }

        if selector == IAllocationManager::slashOperatorCall::SELECTOR {
            return self.visualize_slash_operator(input);
        }

        if selector == IAllocationManager::clearDeallocationQueueCall::SELECTOR {
            return self.visualize_clear_deallocation_queue(input);
        }

        // Strategy Manager - Additional methods
        if selector == IStrategyManager::addSharesCall::SELECTOR {
            return self.visualize_strategy_add_shares(input);
        }

        if selector == IStrategyManager::removeDepositSharesCall::SELECTOR {
            return self.visualize_strategy_remove_deposit_shares(input);
        }

        if selector == IStrategyManager::withdrawSharesAsTokensCall::SELECTOR {
            return self.visualize_strategy_withdraw_shares_as_tokens(input);
        }

        if selector == IStrategyManager::addStrategiesToDepositWhitelistCall::SELECTOR {
            return self.visualize_add_strategies_to_whitelist(input);
        }

        if selector == IStrategyManager::removeStrategiesFromDepositWhitelistCall::SELECTOR {
            return self.visualize_remove_strategies_from_whitelist(input);
        }

        if selector == IStrategyManager::setStrategyWhitelisterCall::SELECTOR {
            return self.visualize_set_strategy_whitelister(input);
        }

        // Delegation Manager - Additional methods
        if selector == IDelegationManager::increaseDelegatedSharesCall::SELECTOR {
            return self.visualize_increase_delegated_shares(input);
        }

        if selector == IDelegationManager::decreaseDelegatedSharesCall::SELECTOR {
            return self.visualize_decrease_delegated_shares(input);
        }

        if selector == IDelegationManager::slashOperatorSharesCall::SELECTOR {
            return self.visualize_slash_operator_shares(input);
        }

        // EigenPodManager - Additional methods
        if selector == IEigenPodManager::addSharesCall::SELECTOR {
            return self.visualize_eigenpod_add_shares(input);
        }

        if selector == IEigenPodManager::removeDepositSharesCall::SELECTOR {
            return self.visualize_eigenpod_remove_deposit_shares(input);
        }

        if selector == IEigenPodManager::withdrawSharesAsTokensCall::SELECTOR {
            return self.visualize_eigenpod_withdraw_shares_as_tokens(input);
        }

        if selector == IEigenPodManager::recordBeaconChainETHBalanceUpdateCall::SELECTOR {
            return self.visualize_record_beacon_chain_balance_update(input);
        }

        if selector == IEigenPodManager::setPectraForkTimestampCall::SELECTOR {
            return self.visualize_set_pectra_fork_timestamp(input);
        }

        if selector == IEigenPodManager::setProofTimestampSetterCall::SELECTOR {
            return self.visualize_set_proof_timestamp_setter(input);
        }

        // AVSDirectory - Additional methods
        if selector == IAVSDirectory::cancelSaltCall::SELECTOR {
            return self.visualize_cancel_salt(input);
        }

        // RewardsCoordinator - Additional methods
        if selector == IRewardsCoordinatorExtended::createOperatorDirectedAVSRewardsSubmissionCall::SELECTOR {
            return self.visualize_create_operator_directed_avs_rewards(input);
        }

        if selector == IRewardsCoordinatorExtended::createOperatorDirectedOperatorSetRewardsSubmissionCall::SELECTOR {
            return self.visualize_create_operator_directed_operator_set_rewards(input);
        }

        if selector == IRewardsCoordinatorExtended::disableRootCall::SELECTOR {
            return self.visualize_disable_root(input);
        }

        if selector == IRewardsCoordinatorExtended::setActivationDelayCall::SELECTOR {
            return self.visualize_set_activation_delay(input);
        }

        if selector == IRewardsCoordinatorExtended::setDefaultOperatorSplitCall::SELECTOR {
            return self.visualize_set_default_operator_split(input);
        }

        if selector == IRewardsCoordinatorExtended::setOperatorAVSSplitCall::SELECTOR {
            return self.visualize_set_operator_avs_split(input);
        }

        if selector == IRewardsCoordinatorExtended::setOperatorPISplitCall::SELECTOR {
            return self.visualize_set_operator_pi_split(input);
        }

        if selector == IRewardsCoordinatorExtended::setOperatorSetSplitCall::SELECTOR {
            return self.visualize_set_operator_set_split(input);
        }

        if selector == IRewardsCoordinatorExtended::setRewardsForAllSubmitterCall::SELECTOR {
            return self.visualize_set_rewards_for_all_submitter(input);
        }

        if selector == IRewardsCoordinatorExtended::setRewardsUpdaterCall::SELECTOR {
            return self.visualize_set_rewards_updater(input);
        }

        // AllocationManager - Additional methods
        if selector == IAllocationManager::addStrategiesToOperatorSetCall::SELECTOR {
            return self.visualize_add_strategies_to_operator_set(input);
        }

        if selector == IAllocationManager::removeStrategiesFromOperatorSetCall::SELECTOR {
            return self.visualize_remove_strategies_from_operator_set(input);
        }

        if selector == IAllocationManager::setAVSRegistrarCall::SELECTOR {
            return self.visualize_set_avs_registrar(input);
        }

        if selector == IAllocationManager::setAllocationDelayCall::SELECTOR {
            return self.visualize_set_allocation_delay(input);
        }

        if selector == IAllocationManager::updateAVSMetadataURICall::SELECTOR {
            return self.visualize_allocation_update_avs_metadata(input);
        }

        None
    }

    // Strategy Manager visualizers
    fn visualize_deposit_into_strategy(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::depositIntoStrategyCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        // Strategy address with name if known
        let strategy_addr = format!("{:?}", call.strategy);
        let strategy_name = KnownContracts::get_strategy_name(&strategy_addr)
            .unwrap_or("Unknown Strategy");

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: strategy_addr.clone(),
                    label: "Strategy".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: strategy_addr,
                    name: strategy_name.to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("EigenLayer".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        // Token address
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.token),
                    label: "Token".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.token),
                    name: "Deposit Token".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        // Amount
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.amount.to_string(),
                    label: "Amount".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: call.amount.to_string(),
                    abbreviation: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Deposit {} tokens into {} ({:?})",
                    call.amount, strategy_name, call.strategy
                ),
                label: "EigenLayer Deposit".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deposit Into Strategy".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Deposit {} tokens into {}", call.amount, strategy_name),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_deposit_into_strategy_with_signature(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::depositIntoStrategyWithSignatureCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        let strategy_addr = format!("{:?}", call.strategy);
        let strategy_name = KnownContracts::get_strategy_name(&strategy_addr)
            .unwrap_or("Unknown Strategy");

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: strategy_addr.clone(),
                    label: "Strategy".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: strategy_addr,
                    name: strategy_name.to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("EigenLayer".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.amount.to_string(),
                    label: "Amount".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: call.amount.to_string(),
                    abbreviation: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Deposit {} tokens into {} with signature",
                    call.amount, strategy_name
                ),
                label: "EigenLayer Deposit (Signed)".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deposit With Signature".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Deposit {} tokens into {}", call.amount, strategy_name),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // Delegation Manager visualizers
    fn visualize_delegate_to(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::delegateToCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "EigenLayer Operator".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Delegate stake to operator {:?}", call.operator),
                label: "EigenLayer Delegate".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Delegate To Operator".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Delegate to {:?}", call.operator),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_undelegate(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::undelegateCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Undelegate staker {:?}", call.staker),
                label: "EigenLayer Undelegate".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Undelegate".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Remove delegation from operator".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_queue_withdrawals(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::queueWithdrawalsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.queuedWithdrawalParams.len().to_string(),
                    label: "Number of Withdrawals".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.queuedWithdrawalParams.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        // Display details for each withdrawal
        for (i, params) in call.queuedWithdrawalParams.iter().enumerate() {
            details.push(AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("{} strategies", params.strategies.len()),
                        label: format!("Withdrawal {} Strategies", i + 1),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: format!("{} strategies", params.strategies.len()),
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            });

            details.push(AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::AddressV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("{:?}", params.withdrawer),
                        label: format!("Withdrawal {} Recipient", i + 1),
                    },
                    address_v2: SignablePayloadFieldAddressV2 {
                        address: format!("{:?}", params.withdrawer),
                        name: "Withdrawer".to_string(),
                        asset_label: "".to_string(),
                        memo: None,
                        badge_text: None,
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            });
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Queue {} withdrawal(s)",
                    call.queuedWithdrawalParams.len()
                ),
                label: "EigenLayer Queue Withdrawals".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Queue Withdrawals".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Queue {} withdrawal(s)", call.queuedWithdrawalParams.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_complete_queued_withdrawal(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::completeQueuedWithdrawalCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.withdrawal.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.withdrawal.staker),
                    name: "Staker".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.withdrawal.strategies.len().to_string(),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.withdrawal.strategies.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: if call.receiveAsTokens { "Yes" } else { "No" }.to_string(),
                    label: "Receive as Tokens".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: if call.receiveAsTokens { "Yes" } else { "No" }.to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Complete withdrawal from {} strategies",
                    call.withdrawal.strategies.len()
                ),
                label: "EigenLayer Complete Withdrawal".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Complete Withdrawal".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("From {} strategies", call.withdrawal.strategies.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_complete_queued_withdrawals(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::completeQueuedWithdrawalsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.withdrawals.len().to_string(),
                    label: "Number of Withdrawals".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.withdrawals.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        let total_strategies: usize = call.withdrawals.iter()
            .map(|w| w.strategies.len())
            .sum();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: total_strategies.to_string(),
                    label: "Total Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: total_strategies.to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Complete {} withdrawal(s) from {} strategies",
                    call.withdrawals.len(), total_strategies
                ),
                label: "EigenLayer Complete Withdrawals".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Complete Multiple Withdrawals".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} withdrawals", call.withdrawals.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // AVS Directory visualizers
    fn visualize_register_operator_to_avs(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::registerOperatorToAVSCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Register operator {:?} to AVS", call.operator),
                label: "EigenLayer AVS Registration".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Register Operator to AVS".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Register operator to this AVS".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_deregister_operator_from_avs(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::deregisterOperatorFromAVSCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Deregister operator {:?} from AVS", call.operator),
                label: "EigenLayer AVS Deregistration".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deregister Operator from AVS".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Deregister operator from this AVS".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_update_avs_metadata_uri(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::updateAVSMetadataURICall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.metadataURI.clone(),
                    label: "Metadata URI".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.metadataURI.clone(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Update AVS metadata URI to {}", call.metadataURI),
                label: "EigenLayer Update AVS Metadata".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Update AVS Metadata".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Update AVS metadata URI".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // Rewards Coordinator visualizers
    fn visualize_create_avs_rewards_submission(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinator::createAVSRewardsSubmissionCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.rewardsSubmissions.len().to_string(),
                    label: "Number of Submissions".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.rewardsSubmissions.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        // Display each reward submission
        for (i, submission) in call.rewardsSubmissions.iter().enumerate() {
            details.push(AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::AddressV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("{:?}", submission.token),
                        label: format!("Reward {} Token", i + 1),
                    },
                    address_v2: SignablePayloadFieldAddressV2 {
                        address: format!("{:?}", submission.token),
                        name: "Reward Token".to_string(),
                        asset_label: "".to_string(),
                        memo: None,
                        badge_text: None,
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            });

            details.push(AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::AmountV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: submission.amount.to_string(),
                        label: format!("Reward {} Amount", i + 1),
                    },
                    amount_v2: SignablePayloadFieldAmountV2 {
                        amount: submission.amount.to_string(),
                        abbreviation: None,
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            });
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Submit {} AVS reward(s)",
                    call.rewardsSubmissions.len()
                ),
                label: "EigenLayer AVS Rewards".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Submit AVS Rewards".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} reward submission(s)", call.rewardsSubmissions.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_process_claim(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinator::processClaimCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.recipient),
                    label: "Recipient".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.recipient),
                    name: "Reward Recipient".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.claim.rootIndex.to_string(),
                    label: "Root Index".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.claim.rootIndex.to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Claim rewards for {:?}", call.recipient),
                label: "EigenLayer Claim Rewards".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Claim Rewards".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Claim for {:?}", call.recipient),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // EigenPod Manager visualizers
    fn visualize_create_pod(&self, _input: &[u8]) -> Option<SignablePayloadField> {
        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Create new EigenPod for native ETH restaking".to_string(),
                label: "EigenLayer Create Pod".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Create EigenPod".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Create new EigenPod for native ETH staking".to_string(),
                }),
                condensed: None,
                expanded: None,
            },
        })
    }

    fn visualize_stake(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::stakeCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        let pubkey_hex = format!("0x{}", hex::encode(&call.pubkey.0));
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: pubkey_hex.clone(),
                    label: "Validator Pubkey".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: pubkey_hex,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Stake 32 ETH to beacon chain validator via EigenPod".to_string(),
                label: "EigenLayer Stake".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Stake to Beacon Chain".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Stake 32 ETH via EigenPod".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // Allocation Manager visualizers
    fn visualize_modify_allocations(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::modifyAllocationsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.allocationParams.len().to_string(),
                    label: "Number of Allocations".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.allocationParams.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Modify {} allocation(s) for operator {:?}",
                    call.allocationParams.len(), call.operator
                ),
                label: "EigenLayer Modify Allocations".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Modify Allocations".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} allocation(s)", call.allocationParams.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // New DelegationManager visualizers
    fn visualize_register_as_operator(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::registerAsOperatorCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.initDelegationApprover),
                    label: "Delegation Approver".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.initDelegationApprover),
                    name: "Delegation Approver".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.allocationDelay.to_string(),
                    label: "Allocation Delay".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.allocationDelay.to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.metadataURI.clone(),
                    label: "Metadata URI".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.metadataURI.clone(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Register as EigenLayer Operator".to_string(),
                label: "EigenLayer Register Operator".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Register as Operator".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Register to become an EigenLayer operator".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_modify_operator_details(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::modifyOperatorDetailsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.newDelegationApprover),
                    label: "New Delegation Approver".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.newDelegationApprover),
                    name: "New Approver".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Modify operator details".to_string(),
                label: "EigenLayer Modify Operator".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Modify Operator Details".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Update operator configuration".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_update_operator_metadata(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::updateOperatorMetadataURICall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.metadataURI.clone(),
                    label: "New Metadata URI".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.metadataURI.clone(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Update operator metadata URI".to_string(),
                label: "EigenLayer Update Metadata".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Update Operator Metadata".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Update operator metadata URI".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_redelegate(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::redelegateCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.newOperator),
                    label: "New Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.newOperator),
                    name: "New Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Redelegate to operator {:?}", call.newOperator),
                label: "EigenLayer Redelegate".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Redelegate".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Switch to a new operator".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // New AllocationManager visualizers
    fn visualize_register_for_operator_sets(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::registerForOperatorSetsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.params.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.params.avs),
                    name: "AVS Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.params.operatorSetIds.len().to_string(),
                    label: "Number of Operator Sets".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.params.operatorSetIds.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Register for {} operator set(s)",
                    call.params.operatorSetIds.len()
                ),
                label: "EigenLayer Register for Operator Sets".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Register for Operator Sets".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} operator set(s)", call.params.operatorSetIds.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_deregister_from_operator_sets(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::deregisterFromOperatorSetsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.params.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.params.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.params.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.params.avs),
                    name: "AVS Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.params.operatorSetIds.len().to_string(),
                    label: "Number of Operator Sets".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.params.operatorSetIds.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Deregister from {} operator set(s)",
                    call.params.operatorSetIds.len()
                ),
                label: "EigenLayer Deregister from Operator Sets".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deregister from Operator Sets".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} operator set(s)", call.params.operatorSetIds.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_create_operator_sets(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::createOperatorSetsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.params.len().to_string(),
                    label: "Number of Operator Sets".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.params.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Create {} operator set(s) for AVS", call.params.len()),
                label: "EigenLayer Create Operator Sets".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Create Operator Sets".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} operator set(s)", call.params.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_slash_operator(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::slashOperatorCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.params.operatorSetId.to_string(),
                    label: "Operator Set ID".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.params.operatorSetId.to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.params.strategies.len().to_string(),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.params.strategies.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.params.description.clone(),
                    label: "Description".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.params.description.clone(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Slash operator in set {}", call.params.operatorSetId),
                label: "EigenLayer Slash Operator".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Slash Operator".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Operator Set {}", call.params.operatorSetId),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_clear_deallocation_queue(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::clearDeallocationQueueCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.strategies.len().to_string(),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.strategies.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Clear deallocation queue for {} strategies",
                    call.strategies.len()
                ),
                label: "EigenLayer Clear Deallocation Queue".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Clear Deallocation Queue".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} strategies", call.strategies.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // New RewardsCoordinator visualizers
    fn visualize_process_claims(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::processClaimsCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.recipient),
                    label: "Recipient".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.recipient),
                    name: "Reward Recipient".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.claims.len().to_string(),
                    label: "Number of Claims".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.claims.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Process {} reward claim(s)", call.claims.len()),
                label: "EigenLayer Batch Claim Rewards".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Process Multiple Claims".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} claim(s)", call.claims.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_create_rewards_for_all_earners(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::createRewardsForAllEarnersCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.rewardsSubmissions.len().to_string(),
                    label: "Number of Submissions".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.rewardsSubmissions.len().to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Create rewards for all earners ({} submission(s))",
                    call.rewardsSubmissions.len()
                ),
                label: "EigenLayer Rewards for All".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Create Rewards for All Earners".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("{} submission(s)", call.rewardsSubmissions.len()),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_submit_root(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::submitRootCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        let root_hex = format!("0x{}", hex::encode(call.root));
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: root_hex.clone(),
                    label: "Merkle Root".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: root_hex,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.rewardsCalculationEndTimestamp.to_string(),
                    label: "Calculation End Timestamp".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.rewardsCalculationEndTimestamp.to_string(),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Submit rewards merkle root".to_string(),
                label: "EigenLayer Submit Root".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Submit Rewards Root".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Submit merkle root for rewards distribution".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_claimer_for(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setClaimerForCall::abi_decode(input).ok()?;

        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.claimer),
                    label: "Claimer".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.claimer),
                    name: "Claimer Address".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Set claimer to {:?}", call.claimer),
                label: "EigenLayer Set Claimer".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Claimer".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Set address that can claim rewards".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // Strategy Manager - Additional visualizers
    fn visualize_strategy_add_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::addSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        let strategy_addr = format!("{:?}", call.strategy);
        let strategy_name = KnownContracts::get_strategy_name(&strategy_addr).unwrap_or("Unknown Strategy");
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: strategy_addr.clone(),
                    label: "Strategy".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: strategy_addr,
                    name: strategy_name.to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("EigenLayer".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.shares),
                    label: "Shares".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.shares),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Add shares to staker".to_string(),
                label: "EigenLayer Add Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Add shares to staker's strategy balance".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_strategy_remove_deposit_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::removeDepositSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        let strategy_addr = format!("{:?}", call.strategy);
        let strategy_name = KnownContracts::get_strategy_name(&strategy_addr).unwrap_or("Unknown Strategy");
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: strategy_addr.clone(),
                    label: "Strategy".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: strategy_addr,
                    name: strategy_name.to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("EigenLayer".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.depositSharesToRemove),
                    label: "Shares to Remove".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.depositSharesToRemove),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Remove deposit shares".to_string(),
                label: "EigenLayer Remove Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Remove Deposit Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Remove shares from staker's deposit".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_strategy_withdraw_shares_as_tokens(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::withdrawSharesAsTokensCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        let strategy_addr = format!("{:?}", call.strategy);
        let strategy_name = KnownContracts::get_strategy_name(&strategy_addr).unwrap_or("Unknown Strategy");
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: strategy_addr.clone(),
                    label: "Strategy".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: strategy_addr,
                    name: strategy_name.to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("EigenLayer".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.token),
                    label: "Token".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.token),
                    name: "Token".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.shares),
                    label: "Shares".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.shares),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Withdraw shares as tokens".to_string(),
                label: "EigenLayer Withdraw Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Withdraw Shares as Tokens".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Convert shares to tokens and withdraw".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_add_strategies_to_whitelist(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::addStrategiesToDepositWhitelistCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} strategies", call.strategiesToWhitelist.len()),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} strategies", call.strategiesToWhitelist.len()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Add strategies to whitelist".to_string(),
                label: "EigenLayer Add Strategies".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add Strategies to Whitelist".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Enable strategies for deposits".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_remove_strategies_from_whitelist(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::removeStrategiesFromDepositWhitelistCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} strategies", call.strategiesToRemoveFromWhitelist.len()),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} strategies", call.strategiesToRemoveFromWhitelist.len()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Remove strategies from whitelist".to_string(),
                label: "EigenLayer Remove Strategies".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Remove Strategies from Whitelist".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Disable strategies for deposits".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_strategy_whitelister(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::setStrategyWhitelisterCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.newStrategyWhitelister),
                    label: "New Whitelister".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.newStrategyWhitelister),
                    name: "Strategy Whitelister".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set strategy whitelister".to_string(),
                label: "EigenLayer Set Whitelister".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Strategy Whitelister".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Change strategy whitelist manager".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // Delegation Manager - Additional visualizers
    fn visualize_increase_delegated_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::increaseDelegatedSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        let strategy_addr = format!("{:?}", call.strategy);
        let strategy_name = KnownContracts::get_strategy_name(&strategy_addr).unwrap_or("Unknown Strategy");
        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: strategy_addr.clone(),
                    label: "Strategy".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: strategy_addr,
                    name: strategy_name.to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("EigenLayer".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.addedShares),
                    label: "Added Shares".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.addedShares),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Increase delegated shares".to_string(),
                label: "EigenLayer Increase Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Increase Delegated Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Add shares to delegation".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_decrease_delegated_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::decreaseDelegatedSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.staker),
                    label: "Staker".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.staker),
                    name: "Staker".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.curDepositShares),
                    label: "Current Deposit Shares".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.curDepositShares),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Decrease delegated shares".to_string(),
                label: "EigenLayer Decrease Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Decrease Delegated Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Remove shares from delegation".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_slash_operator_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::slashOperatorSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} strategies", call.strategies.len()),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} strategies", call.strategies.len()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Slash operator shares".to_string(),
                label: "EigenLayer Slash Operator".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Slash Operator Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Slash operator for misbehavior".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // EigenPodManager - Additional visualizers
    fn visualize_eigenpod_add_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::addSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.podOwner),
                    label: "Pod Owner".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.podOwner),
                    name: "Pod Owner".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.shares),
                    label: "Shares".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.shares),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Add EigenPod shares".to_string(),
                label: "EigenLayer Add Pod Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add EigenPod Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Add beacon chain ETH shares to pod".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_eigenpod_remove_deposit_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::removeDepositSharesCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.podOwner),
                    label: "Pod Owner".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.podOwner),
                    name: "Pod Owner".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.depositSharesToRemove),
                    label: "Shares to Remove".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.depositSharesToRemove),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Remove EigenPod deposit shares".to_string(),
                label: "EigenLayer Remove Pod Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Remove EigenPod Deposit Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Remove deposit shares from pod".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_eigenpod_withdraw_shares_as_tokens(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::withdrawSharesAsTokensCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.podOwner),
                    label: "Pod Owner".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.podOwner),
                    name: "Pod Owner".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.destination),
                    label: "Destination".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.destination),
                    name: "Destination".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.shares),
                    label: "Shares".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: format!("{}", call.shares),
                    abbreviation: Some("shares".to_string()),
                    
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Withdraw EigenPod shares as tokens".to_string(),
                label: "EigenLayer Withdraw Pod Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Withdraw EigenPod Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Convert pod shares to ETH and withdraw".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_record_beacon_chain_balance_update(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::recordBeaconChainETHBalanceUpdateCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.podOwner),
                    label: "Pod Owner".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.podOwner),
                    name: "Pod Owner".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.balanceDeltaWei),
                    label: "Balance Delta".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} wei", call.balanceDeltaWei),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Record beacon chain balance update".to_string(),
                label: "EigenLayer Balance Update".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Record Beacon Chain Balance".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Update beacon chain ETH balance".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_pectra_fork_timestamp(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::setPectraForkTimestampCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.timestamp),
                    label: "Timestamp".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}", call.timestamp),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set Pectra fork timestamp".to_string(),
                label: "EigenLayer Set Fork Time".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Pectra Fork Timestamp".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure Pectra upgrade timing".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_proof_timestamp_setter(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::setProofTimestampSetterCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.newProofTimestampSetter),
                    label: "New Setter".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.newProofTimestampSetter),
                    name: "Proof Timestamp Setter".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set proof timestamp setter".to_string(),
                label: "EigenLayer Set Proof Setter".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Proof Timestamp Setter".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Change proof timestamp manager".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // AVSDirectory - Additional visualizers
    fn visualize_cancel_salt(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::cancelSaltCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.salt),
                    label: "Salt".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{:?}", call.salt),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Cancel salt".to_string(),
                label: "EigenLayer Cancel Salt".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Cancel Salt".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Invalidate signature salt".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // RewardsCoordinator - Additional visualizers
    fn visualize_create_operator_directed_avs_rewards(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::createOperatorDirectedAVSRewardsSubmissionCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} submissions", call.rewardsSubmissions.len()),
                    label: "Number of Submissions".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} submissions", call.rewardsSubmissions.len()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Create operator-directed AVS rewards".to_string(),
                label: "EigenLayer AVS Rewards".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Operator-Directed AVS Rewards".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Submit rewards for AVS operators".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_create_operator_directed_operator_set_rewards(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::createOperatorDirectedOperatorSetRewardsSubmissionCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.operatorSetId),
                    label: "Operator Set ID".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}", call.operatorSetId),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Create operator set rewards".to_string(),
                label: "EigenLayer Operator Set Rewards".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Operator Set Rewards".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Submit rewards for operator set".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_disable_root(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::disableRootCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.rootIndex),
                    label: "Root Index".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}", call.rootIndex),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Disable rewards root".to_string(),
                label: "EigenLayer Disable Root".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Disable Rewards Root".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Invalidate a rewards merkle root".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_activation_delay(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setActivationDelayCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} seconds", call._activationDelay),
                    label: "Activation Delay".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} seconds", call._activationDelay),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set activation delay".to_string(),
                label: "EigenLayer Set Delay".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Activation Delay".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure rewards activation timing".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_default_operator_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setDefaultOperatorSplitCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}%", call.split as f64 / 100.0),
                    label: "Split".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}%", call.split as f64 / 100.0),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set default operator split".to_string(),
                label: "EigenLayer Set Split".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Default Operator Split".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure default rewards split".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_operator_avs_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setOperatorAVSSplitCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}%", call.split as f64 / 100.0),
                    label: "Split".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}%", call.split as f64 / 100.0),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set operator AVS split".to_string(),
                label: "EigenLayer Operator Split".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Operator AVS Split".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure operator-AVS rewards split".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_operator_pi_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setOperatorPISplitCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}%", call.split as f64 / 100.0),
                    label: "Split".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}%", call.split as f64 / 100.0),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set operator PI split".to_string(),
                label: "EigenLayer PI Split".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Operator PI Split".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure programmatic incentives split".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_operator_set_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setOperatorSetSplitCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.operatorSetId),
                    label: "Operator Set ID".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}", call.operatorSetId),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}%", call.split as f64 / 100.0),
                    label: "Split".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}%", call.split as f64 / 100.0),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set operator set split".to_string(),
                label: "EigenLayer Set Split".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Operator Set Split".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure operator set rewards split".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_rewards_for_all_submitter(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setRewardsForAllSubmitterCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call._submitter),
                    label: "Submitter".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call._submitter),
                    name: "Rewards Submitter".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: if call._newValue { "Enabled".to_string() } else { "Disabled".to_string() },
                    label: "Status".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: if call._newValue { "Enabled".to_string() } else { "Disabled".to_string() },
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set rewards submitter".to_string(),
                label: "EigenLayer Set Submitter".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Rewards For All Submitter".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Enable/disable rewards submitter".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_rewards_updater(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setRewardsUpdaterCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call._rewardsUpdater),
                    label: "Rewards Updater".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call._rewardsUpdater),
                    name: "Rewards Updater".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set rewards updater".to_string(),
                label: "EigenLayer Set Updater".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Rewards Updater".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Change rewards updater address".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    // AllocationManager - Additional visualizers
    fn visualize_add_strategies_to_operator_set(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::addStrategiesToOperatorSetCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.operatorSetId),
                    label: "Operator Set ID".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}", call.operatorSetId),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} strategies", call.strategies.len()),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} strategies", call.strategies.len()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Add strategies to operator set".to_string(),
                label: "EigenLayer Add Strategies".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add Strategies to Operator Set".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Enable strategies for operator set".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_remove_strategies_from_operator_set(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::removeStrategiesFromOperatorSetCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{}", call.operatorSetId),
                    label: "Operator Set ID".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{}", call.operatorSetId),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} strategies", call.strategies.len()),
                    label: "Number of Strategies".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} strategies", call.strategies.len()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Remove strategies from operator set".to_string(),
                label: "EigenLayer Remove Strategies".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Remove Strategies from Operator Set".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Disable strategies for operator set".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_avs_registrar(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::setAVSRegistrarCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.registrar),
                    label: "Registrar".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.registrar),
                    name: "AVS Registrar".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: None,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set AVS registrar".to_string(),
                label: "EigenLayer Set Registrar".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set AVS Registrar".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Change AVS registration manager".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_set_allocation_delay(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::setAllocationDelayCall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.operator),
                    label: "Operator".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.operator),
                    name: "Operator".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("Operator".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} seconds", call.delay),
                    label: "Delay".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} seconds", call.delay),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set allocation delay".to_string(),
                label: "EigenLayer Set Delay".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Set Allocation Delay".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Configure operator allocation delay".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }

    fn visualize_allocation_update_avs_metadata(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::updateAVSMetadataURICall::abi_decode(input).ok()?;
        let mut details = Vec::new();

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{:?}", call.avs),
                    label: "AVS".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: format!("{:?}", call.avs),
                    name: "AVS".to_string(),
                    asset_label: "".to_string(),
                    memo: None,
                    badge_text: Some("AVS".to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        details.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: call.metadataURI.clone(),
                    label: "Metadata URI".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: call.metadataURI,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Update AVS metadata URI".to_string(),
                label: "EigenLayer Update Metadata".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Update AVS Metadata URI".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Update AVS metadata location".to_string(),
                }),
                condensed: None,
                expanded: Some(SignablePayloadFieldListLayout { fields: details }),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, Bytes, U256};

    #[test]
    fn test_visualize_deposit_into_strategy() {
        let call = IStrategyManager::depositIntoStrategyCall {
            strategy: Address::from([0x93, 0xc4, 0xb9, 0x44, 0xd0, 0x5d, 0xfe, 0x6d, 0xf7, 0x64, 0x5a, 0x86, 0xcd, 0x22, 0x06, 0x01, 0x6c, 0x51, 0x56, 0x4d]), // stETH strategy
            token: Address::from([0xae, 0x7a, 0xb9, 0x65, 0x20, 0xde, 0x3a, 0x18, 0xe5, 0xe1, 0x11, 0xb5, 0xea, 0xab, 0x09, 0x52, 0x12, 0xfe, 0xb5, 0xe3]), // stETH
            amount: U256::from(1000000000000000000u64), // 1 ETH
        };
        let input = IStrategyManager::depositIntoStrategyCall::abi_encode(&call);

        let result = EigenLayerVisualizer {}.visualize_tx_commands(&input);
        assert!(result.is_some());

        if let Some(SignablePayloadField::PreviewLayout { preview_layout, .. }) = result {
            assert!(preview_layout.title.is_some());
            let title = preview_layout.title.unwrap();
            assert_eq!(title.text, "EigenLayer: Deposit Into Strategy");
        }
    }

    #[test]
    fn test_visualize_delegate_to() {
        let call = IDelegationManager::delegateToCall {
            operator: Address::from([0x11; 20]),
            approverSignatureAndExpiry: Bytes::from(vec![0x01, 0x02, 0x03]),
            approverSalt: [0u8; 32].into(),
        };
        let input = IDelegationManager::delegateToCall::abi_encode(&call);

        let result = EigenLayerVisualizer {}.visualize_tx_commands(&input);
        assert!(result.is_some());
    }

    #[test]
    fn test_visualize_create_pod() {
        let call = IEigenPodManager::createPodCall {};
        let input = IEigenPodManager::createPodCall::abi_encode(&call);

        let result = EigenLayerVisualizer {}.visualize_tx_commands(&input);
        assert!(result.is_some());

        if let Some(SignablePayloadField::PreviewLayout { preview_layout, .. }) = result {
            assert!(preview_layout.title.is_some());
            let title = preview_layout.title.unwrap();
            assert_eq!(title.text, "EigenLayer: Create EigenPod");
        }
    }

    #[test]
    fn test_invalid_selector() {
        let input = vec![0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03, 0x04];
        let result = EigenLayerVisualizer {}.visualize_tx_commands(&input);
        assert!(result.is_none());
    }

    #[test]
    fn test_too_short_input() {
        let input = vec![0x01, 0x02, 0x03];
        let result = EigenLayerVisualizer {}.visualize_tx_commands(&input);
        assert!(result.is_none());
    }
}
