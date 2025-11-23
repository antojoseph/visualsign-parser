use alloy_sol_types::{SolCall, sol};
use alloy_primitives::U256;
use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldAddressV2,
    SignablePayloadFieldAmountV2, SignablePayloadFieldCommon, SignablePayloadFieldDivider,
    SignablePayloadFieldDynamicAnnotation, SignablePayloadFieldListLayout,
    SignablePayloadFieldNumber, SignablePayloadFieldPreviewLayout,
    SignablePayloadFieldStaticAnnotation, SignablePayloadFieldTextV2, DividerStyle,
};
use crate::fmt::format_ether;

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

    // Token addresses
    pub const STETH_TOKEN: &'static str = "0xae7ab96520de3a18e5e111b5eaab095312d7fe84";
    pub const CBETH_TOKEN: &'static str = "0xbe9895146f7af43049ca1c1ae358b0541ea49704";
    pub const RETH_TOKEN: &'static str = "0xae78736cd615f374d3085123a210448e74fc6393";
    pub const ETHX_TOKEN: &'static str = "0xa35b1b31ce002fbf2058d22f30f95d405200a15b";
    pub const ANKRETH_TOKEN: &'static str = "0xe95a203b1a91a908f9b9ce46459d101078c2c3cb";
    pub const OETH_TOKEN: &'static str = "0x856c4efb76c1d1ae02e20ceb03a2a6a08b0b8dc3";
    pub const OSETH_TOKEN: &'static str = "0xf1c9acdc66974dfb6decb12aa385b9cd01190e38";
    pub const SWETH_TOKEN: &'static str = "0xf951e335afb289353dc249e82926178eac7ded78";
    pub const WBETH_TOKEN: &'static str = "0xa2e3356610840701bdf5611a53974510ae27e2e1";
    pub const SFRXETH_TOKEN: &'static str = "0xac3e018457b222d93114458476f3e3416abbe38f";
    pub const LSETH_TOKEN: &'static str = "0x8c1bed5b9a0928467c9b1341da1d7bd5e10b6549";
    pub const METH_TOKEN: &'static str = "0xd5f7838f5c461feff7fe49ea5ebaf7728bb0adfa";
    pub const EIGEN_TOKEN: &'static str = "0xec53bf9167f50cdeb3ae105f56099aaab9061f83";

    // Known strategies with human-readable names and metadata
    pub fn get_strategy_info(address: &str) -> Option<StrategyInfo> {
        match address.to_lowercase().as_str() {
            "0x93c4b944d05dfe6df7645a86cd2206016c51564d" => Some(StrategyInfo {
                name: "stETH Strategy",
                token_symbol: "stETH",
                token_address: Self::STETH_TOKEN,
                description: "Lido Staked ETH",
                is_verified: true,
            }),
            "0x54945180db7943c0ed0fee7edab2bd24620256bc" => Some(StrategyInfo {
                name: "cbETH Strategy",
                token_symbol: "cbETH",
                token_address: Self::CBETH_TOKEN,
                description: "Coinbase Wrapped Staked ETH",
                is_verified: true,
            }),
            "0x1bee69b7dfffa4e2d53c2a2df135c388ad25dcd2" => Some(StrategyInfo {
                name: "rETH Strategy",
                token_symbol: "rETH",
                token_address: Self::RETH_TOKEN,
                description: "Rocket Pool ETH",
                is_verified: true,
            }),
            "0x9d7ed45ee2e8fc5482fa2428f15c971e6369011d" => Some(StrategyInfo {
                name: "ETHx Strategy",
                token_symbol: "ETHx",
                token_address: Self::ETHX_TOKEN,
                description: "Stader ETHx",
                is_verified: true,
            }),
            "0x13760f50a9d7377e4f20cb8cf9e4c26586c658ff" => Some(StrategyInfo {
                name: "ankrETH Strategy",
                token_symbol: "ankrETH",
                token_address: Self::ANKRETH_TOKEN,
                description: "Ankr Staked ETH",
                is_verified: true,
            }),
            "0xa4c637e0f704745d182e4d38cab7e7485321d059" => Some(StrategyInfo {
                name: "OETH Strategy",
                token_symbol: "OETH",
                token_address: Self::OETH_TOKEN,
                description: "Origin ETH",
                is_verified: true,
            }),
            "0x57ba429517c3473b6d34ca9acd56c0e735b94c02" => Some(StrategyInfo {
                name: "osETH Strategy",
                token_symbol: "osETH",
                token_address: Self::OSETH_TOKEN,
                description: "StakeWise osETH",
                is_verified: true,
            }),
            "0x0fe4f44bee93503346a3ac9ee5a26b130a5796d6" => Some(StrategyInfo {
                name: "swETH Strategy",
                token_symbol: "swETH",
                token_address: Self::SWETH_TOKEN,
                description: "Swell ETH",
                is_verified: true,
            }),
            "0x7ca911e83dabf90c90dd3de5411a10f1a6112184" => Some(StrategyInfo {
                name: "wBETH Strategy",
                token_symbol: "wBETH",
                token_address: Self::WBETH_TOKEN,
                description: "Binance Wrapped Beacon ETH",
                is_verified: true,
            }),
            "0x8ca7a5d6f3acd3a7a8bc468a8cd0fb14b6bd28b6" => Some(StrategyInfo {
                name: "sfrxETH Strategy",
                token_symbol: "sfrxETH",
                token_address: Self::SFRXETH_TOKEN,
                description: "Frax Staked ETH",
                is_verified: true,
            }),
            "0xae60d8180437b5c34bb956822ac2710972584473" => Some(StrategyInfo {
                name: "lsETH Strategy",
                token_symbol: "lsETH",
                token_address: Self::LSETH_TOKEN,
                description: "Liquid Staked ETH",
                is_verified: true,
            }),
            "0x298afb19a105d59e74658c4c334ff360bade6dd2" => Some(StrategyInfo {
                name: "mETH Strategy",
                token_symbol: "mETH",
                token_address: Self::METH_TOKEN,
                description: "Mantle Staked ETH",
                is_verified: true,
            }),
            "0xacb55c530acdb2849e6d4f36992cd8c9d50ed8f7" => Some(StrategyInfo {
                name: "EIGEN Strategy",
                token_symbol: "EIGEN",
                token_address: Self::EIGEN_TOKEN,
                description: "EIGEN Token",
                is_verified: true,
            }),
            "0xbeac0eeeeeeeeeeeeeeeeeeeeeeeeeeeeeebeac0" => Some(StrategyInfo {
                name: "Beacon Chain ETH",
                token_symbol: "ETH",
                token_address: "0x0000000000000000000000000000000000000000",
                description: "Native ETH via Beacon Chain",
                is_verified: true,
            }),
            _ => None,
        }
    }

    // Legacy function for backward compatibility
    pub fn get_strategy_name(address: &str) -> Option<&'static str> {
        Self::get_strategy_info(address).map(|info| info.name)
    }

    // Get token symbol from token address
    pub fn get_token_symbol(address: &str) -> Option<&'static str> {
        match address.to_lowercase().as_str() {
            a if a == Self::STETH_TOKEN => Some("stETH"),
            a if a == Self::CBETH_TOKEN => Some("cbETH"),
            a if a == Self::RETH_TOKEN => Some("rETH"),
            a if a == Self::ETHX_TOKEN => Some("ETHx"),
            a if a == Self::ANKRETH_TOKEN => Some("ankrETH"),
            a if a == Self::OETH_TOKEN => Some("OETH"),
            a if a == Self::OSETH_TOKEN => Some("osETH"),
            a if a == Self::SWETH_TOKEN => Some("swETH"),
            a if a == Self::WBETH_TOKEN => Some("wBETH"),
            a if a == Self::SFRXETH_TOKEN => Some("sfrxETH"),
            a if a == Self::LSETH_TOKEN => Some("lsETH"),
            a if a == Self::METH_TOKEN => Some("mETH"),
            a if a == Self::EIGEN_TOKEN => Some("EIGEN"),
            _ => None,
        }
    }

    // Check if an address is a core EigenLayer contract
    pub fn is_core_contract(address: &str) -> bool {
        matches!(
            address.to_lowercase().as_str(),
            a if a == Self::DELEGATION_MANAGER.to_lowercase()
                || a == Self::STRATEGY_MANAGER.to_lowercase()
                || a == Self::EIGENPOD_MANAGER.to_lowercase()
                || a == Self::AVS_DIRECTORY.to_lowercase()
                || a == Self::REWARDS_COORDINATOR.to_lowercase()
                || a == Self::ALLOCATION_MANAGER.to_lowercase()
        )
    }

    // Get contract name
    pub fn get_contract_name(address: &str) -> Option<&'static str> {
        match address.to_lowercase().as_str() {
            a if a == Self::DELEGATION_MANAGER.to_lowercase() => Some("DelegationManager"),
            a if a == Self::STRATEGY_MANAGER.to_lowercase() => Some("StrategyManager"),
            a if a == Self::EIGENPOD_MANAGER.to_lowercase() => Some("EigenPodManager"),
            a if a == Self::AVS_DIRECTORY.to_lowercase() => Some("AVSDirectory"),
            a if a == Self::REWARDS_COORDINATOR.to_lowercase() => Some("RewardsCoordinator"),
            a if a == Self::ALLOCATION_MANAGER.to_lowercase() => Some("AllocationManager"),
            _ => None,
        }
    }
}

// Strategy information structure
pub struct StrategyInfo {
    pub name: &'static str,
    pub token_symbol: &'static str,
    pub token_address: &'static str,
    pub description: &'static str,
    pub is_verified: bool,
}

pub struct EigenLayerVisualizer {}

impl EigenLayerVisualizer {
    // Helper function to create a divider
    fn create_divider() -> AnnotatedPayloadField {
        AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::Divider {
                common: SignablePayloadFieldCommon {
                    fallback_text: "".to_string(),
                    label: "".to_string(),
                },
                divider: SignablePayloadFieldDivider {
                    style: DividerStyle::THIN,
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        }
    }

    // Helper function to create an enhanced address field with all features
    fn create_address_field(
        label: &str,
        address: &str,
        name: Option<&str>,
        asset_label: Option<&str>,
        memo: Option<&str>,
        badge_text: Option<&str>,
    ) -> AnnotatedPayloadField {
        AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: address.to_string(),
                    label: label.to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: address.to_string(),
                    name: name.unwrap_or("").to_string(),
                    asset_label: asset_label.unwrap_or("").to_string(),
                    memo: memo.map(|s| s.to_string()),
                    badge_text: badge_text.map(|s| s.to_string()),
                },
            },
            static_annotation: None,
            dynamic_annotation: None,
        }
    }

    // Helper function to create an amount field with formatting and annotations
    fn create_amount_field(
        label: &str,
        amount_wei: U256,
        token_symbol: Option<&str>,
        static_annotation: Option<&str>,
        dynamic_annotation: Option<SignablePayloadFieldDynamicAnnotation>,
    ) -> AnnotatedPayloadField {
        let formatted_amount = format_ether(amount_wei);

        AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: amount_wei.to_string(),
                    label: label.to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: formatted_amount,
                    abbreviation: token_symbol.map(|s| s.to_string()),
                },
            },
            static_annotation: static_annotation.map(|text| SignablePayloadFieldStaticAnnotation {
                text: text.to_string(),
            }),
            dynamic_annotation,
        }
    }

    // Helper function to create a number field
    fn create_number_field(
        label: &str,
        number: &str,
        static_annotation: Option<&str>,
    ) -> AnnotatedPayloadField {
        AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::Number {
                common: SignablePayloadFieldCommon {
                    fallback_text: number.to_string(),
                    label: label.to_string(),
                },
                number: SignablePayloadFieldNumber {
                    number: number.to_string(),
                },
            },
            static_annotation: static_annotation.map(|text| SignablePayloadFieldStaticAnnotation {
                text: text.to_string(),
            }),
            dynamic_annotation: None,
        }
    }

    // Helper function to create a text field
    fn create_text_field(
        label: &str,
        text: &str,
        static_annotation: Option<&str>,
    ) -> AnnotatedPayloadField {
        AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: text.to_string(),
                    label: label.to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: text.to_string(),
                },
            },
            static_annotation: static_annotation.map(|text| SignablePayloadFieldStaticAnnotation {
                text: text.to_string(),
            }),
            dynamic_annotation: None,
        }
    }

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

        let strategy_addr = format!("{:?}", call.strategy);
        let token_addr = format!("{:?}", call.token);

        // Get strategy info
        let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
        let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
        let token_symbol = strategy_info.as_ref()
            .map(|i| i.token_symbol)
            .or_else(|| KnownContracts::get_token_symbol(&token_addr));

        // Condensed view - just key info
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            None,
            Some("Verified"),
        ));
        condensed_fields.push(Self::create_amount_field(
            "Amount",
            call.amount,
            token_symbol,
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Strategy with full info
        expanded_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            strategy_info.as_ref().map(|i| i.description),
            Some("Verified"),
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Token address
        let token_name = token_symbol.unwrap_or("Token");
        expanded_fields.push(Self::create_address_field(
            "Token",
            &token_addr,
            Some(token_name),
            token_symbol,
            None,
            None,
        ));

        // Amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Amount",
            call.amount,
            token_symbol,
            Some("Subject to EigenLayer withdrawal delay"),
            Some(SignablePayloadFieldDynamicAnnotation {
                field_type: "strategy_apy".to_string(),
                id: strategy_addr.clone(),
                params: vec!["eigenlayer".to_string()],
            }),
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Deposit {} into {}",
                    format_ether(call.amount), strategy_name
                ),
                label: "EigenLayer Deposit".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deposit Into Strategy".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Deposit {} {} into {}", format_ether(call.amount), token_symbol.unwrap_or("tokens"), strategy_name),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_deposit_into_strategy_with_signature(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::depositIntoStrategyWithSignatureCall::abi_decode(input).ok()?;

        let strategy_addr = format!("{:?}", call.strategy);
        let staker_addr = format!("{:?}", call.staker);
        let token_addr = format!("{:?}", call.token);

        // Get strategy info
        let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
        let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
        let token_symbol = strategy_info.as_ref()
            .map(|i| i.token_symbol)
            .or_else(|| KnownContracts::get_token_symbol(&token_addr));

        // Condensed view
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            None,
            Some("Verified"),
        ));
        condensed_fields.push(Self::create_amount_field(
            "Amount",
            call.amount,
            token_symbol,
            None,
            None,
        ));

        // Expanded view
        let mut expanded_fields = vec![];

        expanded_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            Some("Address staking on behalf of"),
            None,
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            strategy_info.as_ref().map(|i| i.description),
            Some("Verified"),
        ));

        expanded_fields.push(Self::create_address_field(
            "Token",
            &token_addr,
            Some(token_symbol.unwrap_or("Token")),
            token_symbol,
            None,
            None,
        ));

        expanded_fields.push(Self::create_amount_field(
            "Amount",
            call.amount,
            token_symbol,
            Some("Deposit signed by staker; subject to withdrawal delay"),
            Some(SignablePayloadFieldDynamicAnnotation {
                field_type: "strategy_apy".to_string(),
                id: strategy_addr.clone(),
                params: vec!["eigenlayer".to_string()],
            }),
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Deposit {} into {} with signature",
                    format_ether(call.amount), strategy_name
                ),
                label: "EigenLayer Deposit (Signed)".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deposit With Signature".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Deposit {} {} into {}", format_ether(call.amount), token_symbol.unwrap_or("tokens"), strategy_name),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // Delegation Manager visualizers
    fn visualize_delegate_to(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::delegateToCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("EigenLayer Operator"),
                None,
                None,
                Some("Operator"),
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("EigenLayer Operator"),
                None,
                Some("Delegate your staked assets to this operator"),
                Some("Operator"),
            ),
        ];

        // Add static annotation about delegation
        expanded_fields[0].static_annotation = Some(SignablePayloadFieldStaticAnnotation {
            text: "Operator will manage your staked assets".to_string(),
        });

        // Add dynamic annotation for operator reputation
        expanded_fields[0].dynamic_annotation = Some(SignablePayloadFieldDynamicAnnotation {
            field_type: "operator_reputation".to_string(),
            id: operator_addr.clone(),
            params: vec!["eigenlayer".to_string()],
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Delegate stake to operator {}", operator_addr),
                label: "EigenLayer Delegate".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Delegate To Operator".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Delegate your staked assets to an operator".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_undelegate(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::undelegateCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.staker);

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Staker",
                &staker_addr,
                Some("Staker Address"),
                None,
                None,
                None,
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Staker",
                &staker_addr,
                Some("Staker Address"),
                None,
                Some("Remove delegation from operator"),
                None,
            ),
        ];

        expanded_fields[0].static_annotation = Some(SignablePayloadFieldStaticAnnotation {
            text: "Initiates withdrawal delay period".to_string(),
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Undelegate staker {}", staker_addr),
                label: "EigenLayer Undelegate".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Undelegate".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Remove delegation from operator".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_queue_withdrawals(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::queueWithdrawalsCall::abi_decode(input).ok()?;

        let num_withdrawals = call.queuedWithdrawalParams.len();

        // Condensed view
        let mut condensed_fields = vec![
            Self::create_number_field(
                "Number of Withdrawals",
                &num_withdrawals.to_string(),
                None,
            ),
        ];

        // Add first withdrawal recipient if present
        if let Some(first_param) = call.queuedWithdrawalParams.first() {
            let withdrawer_addr = format!("{:?}", first_param.withdrawer);
            condensed_fields.push(Self::create_address_field(
                "Recipient",
                &withdrawer_addr,
                Some("Withdrawer"),
                None,
                None,
                None,
            ));
        }

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_number_field(
                "Number of Withdrawals",
                &num_withdrawals.to_string(),
                Some("Each withdrawal requires a waiting period"),
            ),
        ];

        // Display details for each withdrawal
        for (i, params) in call.queuedWithdrawalParams.iter().enumerate() {
            if i > 0 {
                expanded_fields.push(Self::create_divider());
            }

            expanded_fields.push(Self::create_number_field(
                &format!("Withdrawal {} - Strategies", i + 1),
                &params.strategies.len().to_string(),
                None,
            ));

            // Add total shares for this withdrawal
            let total_shares: U256 = params.shares.iter().sum();
            if total_shares > U256::ZERO {
                expanded_fields.push(Self::create_amount_field(
                    &format!("Withdrawal {} - Total Shares", i + 1),
                    total_shares,
                    None,
                    None,
                    None,
                ));
            }

            let withdrawer_addr = format!("{:?}", params.withdrawer);
            expanded_fields.push(Self::create_address_field(
                &format!("Withdrawal {} - Recipient", i + 1),
                &withdrawer_addr,
                Some("Withdrawer"),
                None,
                Some("Address that can complete this withdrawal"),
                None,
            ));
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Queue {} withdrawal(s)", num_withdrawals),
                label: "EigenLayer Queue Withdrawals".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Queue Withdrawals".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Queue {} withdrawal(s) for processing", num_withdrawals),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_complete_queued_withdrawal(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::completeQueuedWithdrawalCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.withdrawal.staker);
        let num_strategies = call.withdrawal.strategies.len();
        let total_shares: U256 = call.withdrawal.shares.iter().sum();

        // Condensed view
        let mut condensed_fields = vec![
            Self::create_address_field(
                "Staker",
                &staker_addr,
                Some("Staker"),
                None,
                None,
                None,
            ),
            Self::create_number_field(
                "Strategies",
                &num_strategies.to_string(),
                None,
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Staker",
                &staker_addr,
                Some("Staker"),
                None,
                Some("Original staker of these assets"),
                None,
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Number of Strategies",
            &num_strategies.to_string(),
            Some("Completing withdrawal from multiple strategies"),
        ));

        if total_shares > U256::ZERO {
            expanded_fields.push(Self::create_amount_field(
                "Total Shares",
                total_shares,
                None,
                None,
                None,
            ));
        }

        // Create a text field for receiveAsTokens using AnnotatedPayloadField directly
        expanded_fields.push(AnnotatedPayloadField {
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: if call.receiveAsTokens { "Yes" } else { "No" }.to_string(),
                    label: "Receive as Tokens".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: if call.receiveAsTokens { "Yes" } else { "No (restake)" }.to_string(),
                },
            },
            static_annotation: Some(SignablePayloadFieldStaticAnnotation {
                text: if call.receiveAsTokens {
                    "Assets will be sent as ERC20 tokens".to_string()
                } else {
                    "Assets will remain staked".to_string()
                },
            }),
            dynamic_annotation: None,
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Complete withdrawal from {} strategies", num_strategies),
                label: "EigenLayer Complete Withdrawal".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Complete Queued Withdrawal".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Finalize withdrawal from {} strategies", num_strategies),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_complete_queued_withdrawals(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::completeQueuedWithdrawalsCall::abi_decode(input).ok()?;

        let num_withdrawals = call.withdrawals.len();
        let total_strategies: usize = call.withdrawals.iter().map(|w| w.strategies.len()).sum();

        // Condensed view
        let condensed_fields = vec![
            Self::create_number_field(
                "Number of Withdrawals",
                &num_withdrawals.to_string(),
                None,
            ),
            Self::create_number_field(
                "Total Strategies",
                &total_strategies.to_string(),
                None,
            ),
        ];

        // Expanded view
        let expanded_fields = vec![
            Self::create_number_field(
                "Number of Withdrawals",
                &num_withdrawals.to_string(),
                Some("Batch completing multiple queued withdrawals"),
            ),
            Self::create_number_field(
                "Total Strategies",
                &total_strategies.to_string(),
                Some("Total strategies across all withdrawals"),
            ),
        ];

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!(
                    "Complete {} withdrawal(s) from {} strategies",
                    num_withdrawals, total_strategies
                ),
                label: "EigenLayer Complete Withdrawals".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Complete Multiple Withdrawals".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Batch complete {} withdrawal(s)", num_withdrawals),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // AVS Directory visualizers
    fn visualize_register_operator_to_avs(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::registerOperatorToAVSCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator"),
                None,
                None,
                Some("Operator"),
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator Address"),
                None,
                Some("Register this operator to provide services for this AVS"),
                Some("Operator"),
            ),
        ];

        expanded_fields[0].static_annotation = Some(SignablePayloadFieldStaticAnnotation {
            text: "Operator will provide validation services to this AVS".to_string(),
        });

        expanded_fields[0].dynamic_annotation = Some(SignablePayloadFieldDynamicAnnotation {
            field_type: "operator_reputation".to_string(),
            id: operator_addr.clone(),
            params: vec!["eigenlayer".to_string()],
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Register operator {} to AVS", operator_addr),
                label: "EigenLayer AVS Registration".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Register Operator to AVS".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Enroll operator in Actively Validated Service".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_deregister_operator_from_avs(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::deregisterOperatorFromAVSCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator"),
                None,
                None,
                Some("Operator"),
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator Address"),
                None,
                Some("Remove this operator from AVS service"),
                Some("Operator"),
            ),
        ];

        expanded_fields[0].static_annotation = Some(SignablePayloadFieldStaticAnnotation {
            text: "Operator will stop providing services to this AVS".to_string(),
        });

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Deregister operator {} from AVS", operator_addr),
                label: "EigenLayer AVS Deregistration".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Deregister Operator from AVS".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Remove operator from Actively Validated Service".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_update_avs_metadata_uri(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::updateAVSMetadataURICall::abi_decode(input).ok()?;

        // Condensed view
        let condensed_fields = vec![
            AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: call.metadataURI.clone(),
                        label: "New Metadata URI".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: if call.metadataURI.len() > 50 {
                            format!("{}...", &call.metadataURI[..47])
                        } else {
                            call.metadataURI.clone()
                        },
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            },
        ];

        // Expanded view
        let expanded_fields = vec![
            AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: call.metadataURI.clone(),
                        label: "New Metadata URI".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: call.metadataURI.clone(),
                    },
                },
                static_annotation: Some(SignablePayloadFieldStaticAnnotation {
                    text: "URL containing AVS service details, terms, and contact information".to_string(),
                }),
                dynamic_annotation: None,
            },
        ];

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Update AVS metadata URI".to_string(),
                label: "EigenLayer Update AVS Metadata".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Update AVS Metadata".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Change AVS metadata URI for service information".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // Rewards Coordinator visualizers
    fn visualize_create_avs_rewards_submission(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinator::createAVSRewardsSubmissionCall::abi_decode(input).ok()?;

        // CONDENSED
        let condensed_fields = vec![
            Self::create_text_field(
                "Submissions",
                &format!("{} submission{}", call.rewardsSubmissions.len(), if call.rewardsSubmissions.len() == 1 { "" } else { "s" }),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_text_field(
                "Total Submissions",
                &call.rewardsSubmissions.len().to_string(),
                Some("Number of reward submissions being created for AVS operators"),
            ),
        ];

        // Display each reward submission
        for (i, submission) in call.rewardsSubmissions.iter().enumerate() {
            expanded_fields.push(Self::create_divider());

            let token_addr = format!("{:?}", submission.token);
            expanded_fields.push(Self::create_address_field(
                &format!("Reward {} Token", i + 1),
                &token_addr,
                None,
                None,
                None,
                None,
            ));

            expanded_fields.push(Self::create_amount_field(
                &format!("Reward {} Amount", i + 1),
                submission.amount,
                None,
                None,
                None,
            ));
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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_process_claim(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinator::processClaimCall::abi_decode(input).ok()?;

        let recipient_addr = format!("{:?}", call.recipient);

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Recipient",
                &recipient_addr,
                Some("Reward Recipient"),
                None,
                None,
                None,
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Recipient",
                &recipient_addr,
                Some("Reward Recipient"),
                None,
                Some("Address receiving claimed rewards"),
                None,
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Root Index",
            &call.claim.rootIndex.to_string(),
            Some("Merkle root identifier for this rewards claim"),
        ));

        expanded_fields.push(Self::create_number_field(
            "Earner Index",
            &call.claim.earnerIndex.to_string(),
            Some("Earner's position in the merkle tree"),
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Claim rewards for {}", recipient_addr),
                label: "EigenLayer Claim Rewards".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Process Rewards Claim".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Claim accumulated rewards using merkle proof".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
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

        let operator_addr = format!("{:?}", call.operator);
        let num_allocations = call.allocationParams.len();

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator Address"),
                None,
                None,
                Some("Operator"),
            ),
            Self::create_number_field(
                "Allocations",
                &num_allocations.to_string(),
                None,
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator Address"),
                None,
                Some("Operator modifying their allocations across AVS"),
                Some("Operator"),
            ),
        ];

        expanded_fields[0].dynamic_annotation = Some(SignablePayloadFieldDynamicAnnotation {
            field_type: "operator_reputation".to_string(),
            id: operator_addr.clone(),
            params: vec!["eigenlayer".to_string()],
        });

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Number of Allocations",
            &num_allocations.to_string(),
            Some("Allocating stake across multiple AVS and strategies"),
        ));

        // Display details for each allocation
        for (i, params) in call.allocationParams.iter().enumerate() {
            if i > 0 && i < 3 {  // Limit to first 3 to avoid overwhelming
                expanded_fields.push(Self::create_divider());

                let avs_addr = format!("{:?}", params.avs);
                expanded_fields.push(Self::create_address_field(
                    &format!("Allocation {} - AVS", i + 1),
                    &avs_addr,
                    Some("AVS"),
                    None,
                    None,
                    Some("AVS"),
                ));

                expanded_fields.push(Self::create_number_field(
                    &format!("Allocation {} - Strategies", i + 1),
                    &params.strategies.len().to_string(),
                    None,
                ));
            }
        }

        if num_allocations > 3 {
            expanded_fields.push(AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("... and {} more allocations", num_allocations - 3),
                        label: "Additional Allocations".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: format!("... and {} more", num_allocations - 3),
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            });
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Modify {} allocation(s) for operator", num_allocations),
                label: "EigenLayer Modify Allocations".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Modify Allocations".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Adjust stake allocation across {} AVS", num_allocations),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // New DelegationManager visualizers
    fn visualize_register_as_operator(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::registerAsOperatorCall::abi_decode(input).ok()?;

        let approver_addr = format!("{:?}", call.initDelegationApprover);

        // Condensed view
        let condensed_fields = vec![
            Self::create_number_field(
                "Allocation Delay",
                &call.allocationDelay.to_string(),
                None,
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Delegation Approver",
                &approver_addr,
                Some("Delegation Approver"),
                None,
                Some("Address that can approve delegations to this operator"),
                None,
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Allocation Delay",
            &call.allocationDelay.to_string(),
            Some("Delay in seconds for allocation modifications"),
        ));

        if !call.metadataURI.is_empty() {
            expanded_fields.push(AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: call.metadataURI.clone(),
                        label: "Metadata URI".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: call.metadataURI.clone(),
                    },
                },
                static_annotation: Some(SignablePayloadFieldStaticAnnotation {
                    text: "URL containing operator details and policies".to_string(),
                }),
                dynamic_annotation: None,
            });
        }

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
                    text: "Become an EigenLayer operator to receive delegations".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_modify_operator_details(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::modifyOperatorDetailsCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);
        let approver_addr = format!("{:?}", call.newDelegationApprover);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Operator",
            &operator_addr,
            Some("Operator"),
            None,
            None,
            Some("Operator"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Operator address
        expanded_fields.push(Self::create_address_field(
            "Operator",
            &operator_addr,
            Some("Operator"),
            None,
            Some("Operator configuration is being updated"),
            Some("Operator"),
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // New delegation approver
        expanded_fields.push(Self::create_address_field(
            "New Delegation Approver",
            &approver_addr,
            Some("Delegation Approver"),
            None,
            Some("This address will approve future delegations"),
            None,
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_update_operator_metadata(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::updateOperatorMetadataURICall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Operator",
            &operator_addr,
            Some("Operator"),
            None,
            None,
            Some("Operator"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Operator address
        expanded_fields.push(Self::create_address_field(
            "Operator",
            &operator_addr,
            Some("Operator"),
            None,
            None,
            Some("Operator"),
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // New metadata URI
        expanded_fields.push(Self::create_text_field(
            "New Metadata URI",
            &call.metadataURI,
            Some("URI containing operator metadata and information"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_redelegate(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::redelegateCall::abi_decode(input).ok()?;

        let new_operator_addr = format!("{:?}", call.newOperator);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "New Operator",
            &new_operator_addr,
            Some("Operator"),
            None,
            None,
            Some("Operator"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // New operator address
        expanded_fields.push(Self::create_address_field(
            "New Operator",
            &new_operator_addr,
            Some("Operator"),
            None,
            Some("Your stake will be delegated to this operator"),
            Some("Operator"),
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Redelegate to new operator".to_string(),
                label: "EigenLayer Redelegate".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Redelegate".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Switch to a new operator".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // New AllocationManager visualizers
    fn visualize_register_for_operator_sets(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::registerForOperatorSetsCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);
        let avs_addr = format!("{:?}", call.params.avs);
        let num_sets = call.params.operatorSetIds.len();

        // Condensed view
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator"),
                None,
                None,
                Some("Operator"),
            ),
            Self::create_address_field(
                "AVS",
                &avs_addr,
                Some("AVS"),
                None,
                None,
                Some("AVS"),
            ),
        ];

        // Expanded view
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                Some("Operator"),
                None,
                Some("Operator registering for AVS operator sets"),
                Some("Operator"),
            ),
        ];

        expanded_fields[0].dynamic_annotation = Some(SignablePayloadFieldDynamicAnnotation {
            field_type: "operator_reputation".to_string(),
            id: operator_addr.clone(),
            params: vec!["eigenlayer".to_string()],
        });

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_address_field(
            "AVS",
            &avs_addr,
            Some("AVS Address"),
            None,
            Some("Actively Validated Service to register with"),
            Some("AVS"),
        ));

        expanded_fields.push(Self::create_number_field(
            "Number of Operator Sets",
            &num_sets.to_string(),
            Some("Operator sets within this AVS"),
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Register for {} operator set(s)", num_sets),
                label: "EigenLayer Register for Operator Sets".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Register for Operator Sets".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Join {} operator set(s) in AVS", num_sets),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
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

        let avs_addr = format!("{:?}", call.avs);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_number_field(
                "Count",
                &call.params.len().to_string(),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Operator Sets to Create",
            &call.params.len().to_string(),
            Some("Number of new operator sets being initialized for this AVS"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_slash_operator(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::slashOperatorCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_text_field(
                "Reason",
                &if call.params.description.len() > 30 {
                    format!("{}...", &call.params.description[..27])
                } else {
                    call.params.description.clone()
                },
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Operator Set ID",
            &call.params.operatorSetId.to_string(),
            None,
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Strategies Affected",
            &call.params.strategies.len().to_string(),
            None,
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Slashing Reason",
            &call.params.description,
            Some("WARNING: Slashing is irreversible and will penalize the operator's stake"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_clear_deallocation_queue(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::clearDeallocationQueueCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
            Self::create_number_field(
                "Strategies",
                &call.strategies.len().to_string(),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Strategies to Clear",
            &call.strategies.len().to_string(),
            Some("Number of strategies being removed from the deallocation queue"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // New RewardsCoordinator visualizers
    fn visualize_process_claims(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::processClaimsCall::abi_decode(input).ok()?;
        let recipient_addr = format!("{:?}", call.recipient);
        let claim_count = call.claims.len();

        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field("Recipient", &recipient_addr, Some("Reward Recipient"), None, None, None));
        condensed_fields.push(Self::create_number_field("Claims", &claim_count.to_string(), None));

        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_address_field("Recipient", &recipient_addr, Some("Reward Recipient"), None, Some("Address receiving the rewards"), None));
        expanded_fields.push(Self::create_divider());
        expanded_fields.push(Self::create_number_field("Claims", &claim_count.to_string(), Some("Number of reward claims being processed")));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon { fallback_text: format!("Process {} reward claims", claim_count), label: "EigenLayer Process Claims".to_string() },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Process Claims".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: format!("Process {} reward claims", claim_count) }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_create_rewards_for_all_earners(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::createRewardsForAllEarnersCall::abi_decode(input).ok()?;
        let submission_count = call.rewardsSubmissions.len();

        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_number_field("Submissions", &submission_count.to_string(), None));

        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_number_field("Submissions", &submission_count.to_string(), Some("Number of reward submissions for all protocol earners")));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon { fallback_text: format!("Create rewards for all earners ({} submissions)", submission_count), label: "EigenLayer Rewards for All".to_string() },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Create Rewards for All Earners".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: format!("{} reward submissions", submission_count) }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_submit_root(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::submitRootCall::abi_decode(input).ok()?;
        let root_hex = format!("0x{}", hex::encode(call.root));

        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_text_field("Merkle Root", &root_hex[..10], None));

        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_text_field("Merkle Root", &root_hex, Some("Root hash for rewards distribution merkle tree")));
        expanded_fields.push(Self::create_divider());
        expanded_fields.push(Self::create_text_field("End Timestamp", &call.rewardsCalculationEndTimestamp.to_string(), Some("Rewards calculation period end time")));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon { fallback_text: "Submit rewards merkle root".to_string(), label: "EigenLayer Submit Root".to_string() },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Submit Rewards Root".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: "Submit merkle root for rewards distribution".to_string() }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_claimer_for(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setClaimerForCall::abi_decode(input).ok()?;
        let claimer_addr = format!("{:?}", call.claimer);

        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field("Claimer", &claimer_addr, Some("Claimer Address"), None, None, None));

        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_address_field("Claimer", &claimer_addr, Some("Claimer Address"), None, Some("This address will be authorized to claim rewards on your behalf"), None));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon { fallback_text: "Set reward claimer".to_string(), label: "EigenLayer Set Claimer".to_string() },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Set Claimer".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: "Set address that can claim rewards".to_string() }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // Strategy Manager - Additional visualizers
    fn visualize_strategy_add_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::addSharesCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.staker);
        let strategy_addr = format!("{:?}", call.strategy);

        // Get strategy info
        let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
        let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
        let token_symbol = strategy_info.as_ref().map(|i| i.token_symbol);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));
        condensed_fields.push(Self::create_amount_field(
            "Shares",
            call.shares,
            Some("shares"),
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Staker address
        expanded_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Strategy with full info
        expanded_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            strategy_info.as_ref().map(|i| i.description),
            Some("Verified"),
        ));

        // Shares amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Shares",
            call.shares,
            Some("shares"),
            Some("Increases staker's strategy balance"),
            None,
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Add {} shares to staker", format_ether(call.shares)),
                label: "EigenLayer Add Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Add {} shares to {}", format_ether(call.shares), strategy_name),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_strategy_remove_deposit_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::removeDepositSharesCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.staker);
        let strategy_addr = format!("{:?}", call.strategy);

        // Get strategy info
        let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
        let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
        let token_symbol = strategy_info.as_ref().map(|i| i.token_symbol);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));
        condensed_fields.push(Self::create_amount_field(
            "Shares to Remove",
            call.depositSharesToRemove,
            Some("shares"),
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Staker address
        expanded_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Strategy with full info
        expanded_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            strategy_info.as_ref().map(|i| i.description),
            Some("Verified"),
        ));

        // Shares amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Shares to Remove",
            call.depositSharesToRemove,
            Some("shares"),
            Some("Reduces staker's deposited shares"),
            None,
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Remove {} shares", format_ether(call.depositSharesToRemove)),
                label: "EigenLayer Remove Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Remove Deposit Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Remove {} shares from {}", format_ether(call.depositSharesToRemove), strategy_name),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_strategy_withdraw_shares_as_tokens(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::withdrawSharesAsTokensCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.staker);
        let strategy_addr = format!("{:?}", call.strategy);
        let token_addr = format!("{:?}", call.token);

        // Get strategy and token info
        let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
        let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
        let token_symbol = strategy_info.as_ref()
            .map(|i| i.token_symbol)
            .or_else(|| KnownContracts::get_token_symbol(&token_addr));

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            None,
            Some("Verified"),
        ));
        condensed_fields.push(Self::create_amount_field(
            "Shares",
            call.shares,
            Some("shares"),
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Staker address
        expanded_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Strategy with full info
        expanded_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            strategy_info.as_ref().map(|i| i.description),
            Some("Verified"),
        ));

        // Token address
        let token_name = token_symbol.unwrap_or("Token");
        expanded_fields.push(Self::create_address_field(
            "Token",
            &token_addr,
            Some(token_name),
            token_symbol,
            None,
            None,
        ));

        // Shares amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Shares",
            call.shares,
            Some("shares"),
            Some("Converts shares to tokens and withdraws"),
            None,
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Withdraw {} shares as tokens", format_ether(call.shares)),
                label: "EigenLayer Withdraw Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Withdraw Shares as Tokens".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Withdraw {} shares from {}", format_ether(call.shares), strategy_name),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_add_strategies_to_whitelist(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::addStrategiesToDepositWhitelistCall::abi_decode(input).ok()?;

        let strategy_count = call.strategiesToWhitelist.len();

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_number_field(
            "Strategies",
            &strategy_count.to_string(),
            Some("Number of strategies to whitelist"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Strategy count
        expanded_fields.push(Self::create_number_field(
            "Strategy Count",
            &strategy_count.to_string(),
            Some("Total strategies being whitelisted"),
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // List each strategy
        for (i, strategy) in call.strategiesToWhitelist.iter().enumerate().take(5) {
            let strategy_addr = format!("{:?}", strategy);
            let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
            let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
            let token_symbol = strategy_info.as_ref().map(|i| i.token_symbol);

            expanded_fields.push(Self::create_address_field(
                &format!("Strategy {}", i + 1),
                &strategy_addr,
                Some(strategy_name),
                token_symbol,
                strategy_info.as_ref().map(|i| i.description),
                Some("Verified"),
            ));
        }

        if strategy_count > 5 {
            expanded_fields.push(Self::create_text_field(
                "Additional Strategies",
                &format!("... and {} more", strategy_count - 5),
                None,
            ));
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Add {} strategies to whitelist", strategy_count),
                label: "EigenLayer Add Strategies".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add Strategies to Whitelist".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Enable {} strategies for deposits", strategy_count),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_remove_strategies_from_whitelist(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::removeStrategiesFromDepositWhitelistCall::abi_decode(input).ok()?;

        let strategy_count = call.strategiesToRemoveFromWhitelist.len();

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_number_field(
            "Strategies",
            &strategy_count.to_string(),
            Some("Number of strategies to remove"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Strategy count
        expanded_fields.push(Self::create_number_field(
            "Strategy Count",
            &strategy_count.to_string(),
            Some("Total strategies being removed from whitelist"),
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // List each strategy
        for (i, strategy) in call.strategiesToRemoveFromWhitelist.iter().enumerate().take(5) {
            let strategy_addr = format!("{:?}", strategy);
            let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
            let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
            let token_symbol = strategy_info.as_ref().map(|i| i.token_symbol);

            expanded_fields.push(Self::create_address_field(
                &format!("Strategy {}", i + 1),
                &strategy_addr,
                Some(strategy_name),
                token_symbol,
                strategy_info.as_ref().map(|i| i.description),
                Some("Verified"),
            ));
        }

        if strategy_count > 5 {
            expanded_fields.push(Self::create_text_field(
                "Additional Strategies",
                &format!("... and {} more", strategy_count - 5),
                None,
            ));
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Remove {} strategies from whitelist", strategy_count),
                label: "EigenLayer Remove Strategies".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Remove Strategies from Whitelist".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Disable {} strategies for deposits", strategy_count),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_strategy_whitelister(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IStrategyManager::setStrategyWhitelisterCall::abi_decode(input).ok()?;

        let whitelister_addr = format!("{:?}", call.newStrategyWhitelister);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "New Whitelister",
            &whitelister_addr,
            Some("Strategy Whitelister"),
            None,
            None,
            Some("Admin"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // New whitelister with warning
        expanded_fields.push(Self::create_address_field(
            "New Whitelister",
            &whitelister_addr,
            Some("Strategy Whitelister"),
            None,
            Some("This address will control which strategies can accept deposits"),
            Some("Admin"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // Delegation Manager - Additional visualizers
    fn visualize_increase_delegated_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::increaseDelegatedSharesCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.staker);
        let strategy_addr = format!("{:?}", call.strategy);

        // Get strategy info
        let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
        let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
        let token_symbol = strategy_info.as_ref().map(|i| i.token_symbol);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));
        condensed_fields.push(Self::create_amount_field(
            "Added Shares",
            call.addedShares,
            Some("shares"),
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Staker address
        expanded_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Strategy with full info
        expanded_fields.push(Self::create_address_field(
            "Strategy",
            &strategy_addr,
            Some(strategy_name),
            token_symbol,
            strategy_info.as_ref().map(|i| i.description),
            Some("Verified"),
        ));

        // Shares amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Added Shares",
            call.addedShares,
            Some("shares"),
            Some("Increases delegated shares for this strategy"),
            None,
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Increase delegated shares by {}", format_ether(call.addedShares)),
                label: "EigenLayer Increase Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Increase Delegated Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Add {} shares to delegation", format_ether(call.addedShares)),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_decrease_delegated_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::decreaseDelegatedSharesCall::abi_decode(input).ok()?;

        let staker_addr = format!("{:?}", call.staker);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));
        condensed_fields.push(Self::create_amount_field(
            "Deposit Shares",
            call.curDepositShares,
            Some("shares"),
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Staker address
        expanded_fields.push(Self::create_address_field(
            "Staker",
            &staker_addr,
            Some("Staker"),
            None,
            None,
            None,
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Shares amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Current Deposit Shares",
            call.curDepositShares,
            Some("shares"),
            Some("Decreases delegated shares for this strategy"),
            None,
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Decrease delegated shares by {}", format_ether(call.curDepositShares)),
                label: "EigenLayer Decrease Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Decrease Delegated Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Remove {} shares from delegation", format_ether(call.curDepositShares)),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_slash_operator_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IDelegationManager::slashOperatorSharesCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);
        let strategy_count = call.strategies.len();

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Operator",
            &operator_addr,
            Some("Operator"),
            None,
            None,
            Some("Operator"),
        ));
        condensed_fields.push(Self::create_number_field(
            "Strategies",
            &strategy_count.to_string(),
            Some("Number of strategies being slashed"),
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Operator address with warning
        expanded_fields.push(Self::create_address_field(
            "Operator",
            &operator_addr,
            Some("Operator"),
            None,
            Some("This operator is being slashed for misbehavior"),
            Some("Operator"),
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Strategy count
        expanded_fields.push(Self::create_number_field(
            "Strategies Affected",
            &strategy_count.to_string(),
            Some("Total strategies being slashed"),
        ));

        // List strategies and amounts
        for (i, (strategy, shares)) in call.strategies.iter().zip(call.shareAmounts.iter()).enumerate().take(3) {
            let strategy_addr = format!("{:?}", strategy);
            let strategy_info = KnownContracts::get_strategy_info(&strategy_addr);
            let strategy_name = strategy_info.as_ref().map(|i| i.name).unwrap_or("Unknown Strategy");
            let token_symbol = strategy_info.as_ref().map(|i| i.token_symbol);

            expanded_fields.push(Self::create_address_field(
                &format!("Strategy {}", i + 1),
                &strategy_addr,
                Some(strategy_name),
                token_symbol,
                None,
                Some("Verified"),
            ));

            expanded_fields.push(Self::create_amount_field(
                &format!("Slash Amount {}", i + 1),
                *shares,
                Some("shares"),
                Some("Shares being slashed"),
                None,
            ));
        }

        if strategy_count > 3 {
            expanded_fields.push(Self::create_text_field(
                "Additional",
                &format!("... and {} more strategies", strategy_count - 3),
                None,
            ));
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Slash operator across {} strategies", strategy_count),
                label: "EigenLayer Slash Operator".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Slash Operator Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Slash operator across {} strategies", strategy_count),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // EigenPodManager - Additional visualizers
    fn visualize_eigenpod_add_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::addSharesCall::abi_decode(input).ok()?;

        let pod_owner_addr = format!("{:?}", call.podOwner);

        // Condensed view - key info only
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field(
            "Pod Owner",
            &pod_owner_addr,
            Some("Pod Owner"),
            None,
            None,
            None,
        ));
        condensed_fields.push(Self::create_amount_field(
            "Shares",
            call.shares,
            Some("shares"),
            None,
            None,
        ));

        // Expanded view - full details
        let mut expanded_fields = vec![];

        // Pod owner address
        expanded_fields.push(Self::create_address_field(
            "Pod Owner",
            &pod_owner_addr,
            Some("Pod Owner"),
            None,
            None,
            None,
        ));

        // Divider
        expanded_fields.push(Self::create_divider());

        // Shares amount with annotation
        expanded_fields.push(Self::create_amount_field(
            "Shares",
            call.shares,
            Some("shares"),
            Some("Beacon chain ETH shares being added to pod"),
            None,
        ));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Add {} EigenPod shares", format_ether(call.shares)),
                label: "EigenLayer Add Pod Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Add EigenPod Shares".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: format!("Add {} shares to pod", format_ether(call.shares)),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_eigenpod_remove_deposit_shares(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::removeDepositSharesCall::abi_decode(input).ok()?;

        let pod_owner_addr = format!("{:?}", call.podOwner);

        // Condensed view
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field("Pod Owner", &pod_owner_addr, Some("Pod Owner"), None, None, None));
        condensed_fields.push(Self::create_amount_field("Shares to Remove", call.depositSharesToRemove, Some("shares"), None, None));

        // Expanded view
        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_address_field("Pod Owner", &pod_owner_addr, Some("Pod Owner"), None, None, None));
        expanded_fields.push(Self::create_divider());
        expanded_fields.push(Self::create_amount_field("Shares to Remove", call.depositSharesToRemove, Some("shares"), Some("Reduces pod deposit shares"), None));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Remove {} EigenPod shares", format_ether(call.depositSharesToRemove)),
                label: "EigenLayer Remove Pod Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Remove EigenPod Deposit Shares".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: format!("Remove {} shares from pod", format_ether(call.depositSharesToRemove)) }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_eigenpod_withdraw_shares_as_tokens(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::withdrawSharesAsTokensCall::abi_decode(input).ok()?;

        let pod_owner_addr = format!("{:?}", call.podOwner);
        let destination_addr = format!("{:?}", call.destination);

        // Condensed view
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field("Destination", &destination_addr, Some("Recipient"), None, None, None));
        condensed_fields.push(Self::create_amount_field("Shares", call.shares, Some("ETH"), None, None));

        // Expanded view
        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_address_field("Pod Owner", &pod_owner_addr, Some("Pod Owner"), None, None, None));
        expanded_fields.push(Self::create_divider());
        expanded_fields.push(Self::create_address_field("Destination", &destination_addr, Some("Recipient"), None, Some("ETH will be sent to this address"), None));
        expanded_fields.push(Self::create_amount_field("Shares", call.shares, Some("ETH"), Some("Converts shares to ETH and withdraws"), None));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Withdraw {} ETH from EigenPod", format_ether(call.shares)),
                label: "EigenLayer Withdraw Pod Shares".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Withdraw EigenPod Shares".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: format!("Withdraw {} ETH from pod", format_ether(call.shares)) }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_record_beacon_chain_balance_update(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::recordBeaconChainETHBalanceUpdateCall::abi_decode(input).ok()?;

        let pod_owner_addr = format!("{:?}", call.podOwner);

        // Condensed view
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field("Pod Owner", &pod_owner_addr, Some("Pod Owner"), None, None, None));
        condensed_fields.push(Self::create_text_field("Balance Delta", &format!("{} wei", call.balanceDeltaWei), None));

        // Expanded view
        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_address_field("Pod Owner", &pod_owner_addr, Some("Pod Owner"), None, None, None));
        expanded_fields.push(Self::create_divider());
        expanded_fields.push(Self::create_text_field("Balance Delta", &format!("{} wei", call.balanceDeltaWei), Some("Change in beacon chain ETH balance")));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Record beacon chain balance update".to_string(),
                label: "EigenLayer Balance Update".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Record Beacon Chain Balance".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: "Update beacon chain ETH balance".to_string() }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_pectra_fork_timestamp(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::setPectraForkTimestampCall::abi_decode(input).ok()?;

        // Condensed view
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_text_field("Timestamp", &call.timestamp.to_string(), Some("Pectra fork activation time")));

        // Expanded view
        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_text_field("Timestamp", &call.timestamp.to_string(), Some("Unix timestamp for Pectra fork activation - affects validator withdrawal credentials")));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set Pectra fork timestamp".to_string(),
                label: "EigenLayer Set Fork Time".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Set Pectra Fork Timestamp".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: "Configure Pectra upgrade timing".to_string() }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_proof_timestamp_setter(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IEigenPodManager::setProofTimestampSetterCall::abi_decode(input).ok()?;

        let setter_addr = format!("{:?}", call.newProofTimestampSetter);

        // Condensed view
        let mut condensed_fields = vec![];
        condensed_fields.push(Self::create_address_field("New Setter", &setter_addr, Some("Proof Timestamp Setter"), None, None, Some("Admin")));

        // Expanded view
        let mut expanded_fields = vec![];
        expanded_fields.push(Self::create_address_field("New Setter", &setter_addr, Some("Proof Timestamp Setter"), None, Some("This address will control proof timestamp settings for beacon chain validation"), Some("Admin")));

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Set proof timestamp setter".to_string(),
                label: "EigenLayer Set Proof Setter".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 { text: "EigenLayer: Set Proof Timestamp Setter".to_string() }),
                subtitle: Some(SignablePayloadFieldTextV2 { text: "Change proof timestamp manager".to_string() }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // AVSDirectory - Additional visualizers
    fn visualize_cancel_salt(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAVSDirectory::cancelSaltCall::abi_decode(input).ok()?;

        let salt_str = format!("{:?}", call.salt);

        // Condensed view
        let condensed_fields = vec![
            AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: salt_str.clone(),
                        label: "Salt".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: if salt_str.len() > 20 {
                            format!("{}...", &salt_str[..17])
                        } else {
                            salt_str.clone()
                        },
                    },
                },
                static_annotation: None,
                dynamic_annotation: None,
            },
        ];

        // Expanded view
        let expanded_fields = vec![
            AnnotatedPayloadField {
                signable_payload_field: SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: salt_str.clone(),
                        label: "Salt Value".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: salt_str,
                    },
                },
                static_annotation: Some(SignablePayloadFieldStaticAnnotation {
                    text: "Invalidate this salt to prevent replay attacks on signed messages".to_string(),
                }),
                dynamic_annotation: None,
            },
        ];

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: "Cancel signature salt".to_string(),
                label: "EigenLayer Cancel Salt".to_string(),
            },
            preview_layout: SignablePayloadFieldPreviewLayout {
                title: Some(SignablePayloadFieldTextV2 {
                    text: "EigenLayer: Cancel Salt".to_string(),
                }),
                subtitle: Some(SignablePayloadFieldTextV2 {
                    text: "Invalidate signature salt for security".to_string(),
                }),
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // RewardsCoordinator - Additional visualizers
    fn visualize_create_operator_directed_avs_rewards(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::createOperatorDirectedAVSRewardsSubmissionCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_text_field(
                "Submissions",
                &format!("{} submission{}", call.rewardsSubmissions.len(), if call.rewardsSubmissions.len() == 1 { "" } else { "s" }),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Total Submissions",
            &call.rewardsSubmissions.len().to_string(),
            Some("Number of reward submissions being created for operators"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_create_operator_directed_operator_set_rewards(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::createOperatorDirectedOperatorSetRewardsSubmissionCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_number_field(
                "Operator Set",
                &call.operatorSetId.to_string(),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Operator Set ID",
            &call.operatorSetId.to_string(),
            Some("The operator set receiving rewards"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_disable_root(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::disableRootCall::abi_decode(input).ok()?;

        // CONDENSED
        let condensed_fields = vec![
            Self::create_number_field(
                "Root Index",
                &call.rootIndex.to_string(),
                None,
            ),
        ];

        // EXPANDED
        let expanded_fields = vec![
            Self::create_number_field(
                "Root Index",
                &call.rootIndex.to_string(),
                Some("Invalidating this merkle root prevents further reward claims from it"),
            ),
        ];

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_activation_delay(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setActivationDelayCall::abi_decode(input).ok()?;

        let delay_text = format!("{} seconds", call._activationDelay);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_text_field(
                "Delay",
                &delay_text,
                None,
            ),
        ];

        // EXPANDED
        let expanded_fields = vec![
            Self::create_text_field(
                "Activation Delay",
                &delay_text,
                Some("Time delay before rewards become active and claimable"),
            ),
        ];

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_default_operator_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setDefaultOperatorSplitCall::abi_decode(input).ok()?;

        let split_percent = format!("{}%", call.split as f64 / 100.0);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_text_field(
                "Default Split",
                &split_percent,
                None,
            ),
        ];

        // EXPANDED
        let expanded_fields = vec![
            Self::create_text_field(
                "Default Operator Split",
                &split_percent,
                Some("Percentage of rewards that operators receive by default"),
            ),
        ];

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_operator_avs_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setOperatorAVSSplitCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);
        let avs_addr = format!("{:?}", call.avs);

        let split_percent = format!("{}%", call.split as f64 / 100.0);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
            Self::create_text_field(
                "Split",
                &split_percent,
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_address_field(
            "AVS",
            &avs_addr,
            None,
            None,
            None,
            Some("AVS"),
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Rewards Split",
            &split_percent,
            Some("Percentage of rewards the operator receives from this AVS"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_operator_pi_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setOperatorPISplitCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        let split_percent = format!("{}%", call.split as f64 / 100.0);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
            Self::create_text_field(
                "PI Split",
                &split_percent,
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Programmatic Incentives Split",
            &split_percent,
            Some("Percentage of programmatic incentive rewards the operator receives"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_operator_set_split(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setOperatorSetSplitCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);
        let split_percent = format!("{}%", call.split as f64 / 100.0);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_text_field(
                "Split",
                &split_percent,
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Operator Set ID",
            &call.operatorSetId.to_string(),
            None,
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Rewards Split",
            &split_percent,
            Some("Percentage of rewards allocated to this operator set"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_rewards_for_all_submitter(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setRewardsForAllSubmitterCall::abi_decode(input).ok()?;

        let submitter_addr = format!("{:?}", call._submitter);

        let status = if call._newValue { "Enabled" } else { "Disabled" };

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "Submitter",
                &submitter_addr,
                None,
                None,
                None,
                Some("Config"),
            ),
            Self::create_text_field(
                "Status",
                status,
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Submitter Address",
                &submitter_addr,
                None,
                None,
                None,
                Some("Config"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "New Status",
            status,
            Some("Whether this address can submit rewards for all operators"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_rewards_updater(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IRewardsCoordinatorExtended::setRewardsUpdaterCall::abi_decode(input).ok()?;

        let updater_addr = format!("{:?}", call._rewardsUpdater);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "Updater",
                &updater_addr,
                None,
                None,
                None,
                Some("Admin"),
            ),
        ];

        // EXPANDED
        let expanded_fields = vec![
            Self::create_address_field(
                "New Rewards Updater",
                &updater_addr,
                None,
                None,
                Some("Address with permission to update reward parameters"),
                Some("Admin"),
            ),
        ];

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    // AllocationManager - Additional visualizers
    fn visualize_add_strategies_to_operator_set(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::addStrategiesToOperatorSetCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_text_field(
                "Strategies",
                &format!("{} strateg{}", call.strategies.len(), if call.strategies.len() == 1 { "y" } else { "ies" }),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Operator Set ID",
            &call.operatorSetId.to_string(),
            None,
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Strategies Being Added",
            &call.strategies.len().to_string(),
            Some("Number of strategies being enabled for this operator set"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_remove_strategies_from_operator_set(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::removeStrategiesFromOperatorSetCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_text_field(
                "Strategies",
                &format!("{} strateg{}", call.strategies.len(), if call.strategies.len() == 1 { "y" } else { "ies" }),
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_number_field(
            "Operator Set ID",
            &call.operatorSetId.to_string(),
            None,
        ));

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Strategies Being Removed",
            &call.strategies.len().to_string(),
            Some("Number of strategies being disabled from this operator set"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_avs_registrar(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::setAVSRegistrarCall::abi_decode(input).ok()?;

        let avs_addr = format!("{:?}", call.avs);
        let registrar_addr = format!("{:?}", call.registrar);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
            Self::create_address_field(
                "Registrar",
                &registrar_addr,
                None,
                None,
                None,
                Some("Admin"),
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "AVS",
                &avs_addr,
                None,
                None,
                None,
                Some("AVS"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_address_field(
            "New Registrar",
            &registrar_addr,
            None,
            None,
            Some("Address with permission to manage operator registrations for this AVS"),
            Some("Admin"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
            },
        })
    }

    fn visualize_set_allocation_delay(&self, input: &[u8]) -> Option<SignablePayloadField> {
        let call = IAllocationManager::setAllocationDelayCall::abi_decode(input).ok()?;

        let operator_addr = format!("{:?}", call.operator);

        let delay_text = format!("{} seconds", call.delay);

        // CONDENSED
        let condensed_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
            Self::create_text_field(
                "Delay",
                &delay_text,
                None,
            ),
        ];

        // EXPANDED
        let mut expanded_fields = vec![
            Self::create_address_field(
                "Operator",
                &operator_addr,
                None,
                None,
                None,
                Some("Operator"),
            ),
        ];

        expanded_fields.push(Self::create_divider());

        expanded_fields.push(Self::create_text_field(
            "Allocation Delay",
            &delay_text,
            Some("Time delay before allocations take effect for this operator"),
        ));

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
                condensed: Some(SignablePayloadFieldListLayout { fields: condensed_fields }),
                expanded: Some(SignablePayloadFieldListLayout { fields: expanded_fields }),
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
