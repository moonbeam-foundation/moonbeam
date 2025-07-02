// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import "@polkadot/api-base/types/errors";

import type { ApiTypes, AugmentedError } from "@polkadot/api-base/types";

export type __AugmentedError<ApiType extends ApiTypes> = AugmentedError<ApiType>;

declare module "@polkadot/api-base/types/errors" {
  interface AugmentedErrors<ApiType extends ApiTypes> {
    assetManager: {
      AssetAlreadyExists: AugmentedError<ApiType>;
      AssetDoesNotExist: AugmentedError<ApiType>;
      ErrorCreatingAsset: AugmentedError<ApiType>;
      ErrorDestroyingAsset: AugmentedError<ApiType>;
      LocalAssetLimitReached: AugmentedError<ApiType>;
      NonExistentLocalAsset: AugmentedError<ApiType>;
      NotSufficientDeposit: AugmentedError<ApiType>;
      TooLowNumAssetsWeightHint: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    assets: {
      /**
       * The asset-account already exists.
       **/
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * The asset is not live, and likely being destroyed.
       **/
      AssetNotLive: AugmentedError<ApiType>;
      /**
       * The asset ID must be equal to the [`NextAssetId`].
       **/
      BadAssetId: AugmentedError<ApiType>;
      /**
       * Invalid metadata given.
       **/
      BadMetadata: AugmentedError<ApiType>;
      /**
       * Invalid witness data given.
       **/
      BadWitness: AugmentedError<ApiType>;
      /**
       * Account balance must be greater than or equal to the transfer amount.
       **/
      BalanceLow: AugmentedError<ApiType>;
      /**
       * Callback action resulted in error
       **/
      CallbackFailed: AugmentedError<ApiType>;
      /**
       * The origin account is frozen.
       **/
      Frozen: AugmentedError<ApiType>;
      /**
       * The asset status is not the expected status.
       **/
      IncorrectStatus: AugmentedError<ApiType>;
      /**
       * The asset ID is already taken.
       **/
      InUse: AugmentedError<ApiType>;
      /**
       * The asset is a live asset and is actively being used. Usually emit for operations such
       * as `start_destroy` which require the asset to be in a destroying state.
       **/
      LiveAsset: AugmentedError<ApiType>;
      /**
       * Minimum balance should be non-zero.
       **/
      MinBalanceZero: AugmentedError<ApiType>;
      /**
       * The account to alter does not exist.
       **/
      NoAccount: AugmentedError<ApiType>;
      /**
       * The asset-account doesn't have an associated deposit.
       **/
      NoDeposit: AugmentedError<ApiType>;
      /**
       * The signing account has no permission to do the operation.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * The asset should be frozen before the given operation.
       **/
      NotFrozen: AugmentedError<ApiType>;
      /**
       * No approval exists that would allow the transfer.
       **/
      Unapproved: AugmentedError<ApiType>;
      /**
       * Unable to increment the consumer reference counters on the account. Either no provider
       * reference exists to allow a non-zero balance of a non-self-sufficient asset, or one
       * fewer then the maximum number of consumers has been reached.
       **/
      UnavailableConsumer: AugmentedError<ApiType>;
      /**
       * The given asset ID is unknown.
       **/
      Unknown: AugmentedError<ApiType>;
      /**
       * The operation would result in funds being burned.
       **/
      WouldBurn: AugmentedError<ApiType>;
      /**
       * The source account would not survive the transfer and it needs to stay alive.
       **/
      WouldDie: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    authorInherent: {
      /**
       * Author already set in block.
       **/
      AuthorAlreadySet: AugmentedError<ApiType>;
      /**
       * The author in the inherent is not an eligible author.
       **/
      CannotBeAuthor: AugmentedError<ApiType>;
      /**
       * No AccountId was found to be associated with this author
       **/
      NoAccountId: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    authorMapping: {
      /**
       * The NimbusId in question is already associated and cannot be overwritten
       **/
      AlreadyAssociated: AugmentedError<ApiType>;
      /**
       * The association can't be cleared because it is not found.
       **/
      AssociationNotFound: AugmentedError<ApiType>;
      /**
       * This account cannot set an author because it cannon afford the security deposit
       **/
      CannotAffordSecurityDeposit: AugmentedError<ApiType>;
      /**
       * Failed to decode T::Keys for `set_keys`
       **/
      DecodeKeysFailed: AugmentedError<ApiType>;
      /**
       * Failed to decode NimbusId for `set_keys`
       **/
      DecodeNimbusFailed: AugmentedError<ApiType>;
      /**
       * The association can't be cleared because it belongs to another account.
       **/
      NotYourAssociation: AugmentedError<ApiType>;
      /**
       * No existing NimbusId can be found for the account
       **/
      OldAuthorIdNotFound: AugmentedError<ApiType>;
      /**
       * Keys have wrong size
       **/
      WrongKeySize: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    balances: {
      /**
       * Beneficiary account must pre-exist.
       **/
      DeadAccount: AugmentedError<ApiType>;
      /**
       * The delta cannot be zero.
       **/
      DeltaZero: AugmentedError<ApiType>;
      /**
       * Value too low to create account due to existential deposit.
       **/
      ExistentialDeposit: AugmentedError<ApiType>;
      /**
       * A vesting schedule already exists for this account.
       **/
      ExistingVestingSchedule: AugmentedError<ApiType>;
      /**
       * Transfer/payment would kill account.
       **/
      Expendability: AugmentedError<ApiType>;
      /**
       * Balance too low to send value.
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * The issuance cannot be modified since it is already deactivated.
       **/
      IssuanceDeactivated: AugmentedError<ApiType>;
      /**
       * Account liquidity restrictions prevent withdrawal.
       **/
      LiquidityRestrictions: AugmentedError<ApiType>;
      /**
       * Number of freezes exceed `MaxFreezes`.
       **/
      TooManyFreezes: AugmentedError<ApiType>;
      /**
       * Number of holds exceed `VariantCountOf<T::RuntimeHoldReason>`.
       **/
      TooManyHolds: AugmentedError<ApiType>;
      /**
       * Number of named reserves exceed `MaxReserves`.
       **/
      TooManyReserves: AugmentedError<ApiType>;
      /**
       * Vesting balance too high to send value.
       **/
      VestingBalance: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bridgeKusamaGrandpa: {
      /**
       * The pallet has already been initialized.
       **/
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * The submitter wanted free execution, but the difference between best known and
       * bundled header numbers is below the `FreeHeadersInterval`.
       **/
      BelowFreeHeaderInterval: AugmentedError<ApiType>;
      /**
       * Error generated by the `OwnedBridgeModule` trait.
       **/
      BridgeModule: AugmentedError<ApiType>;
      /**
       * The submitter wanted free execution, but we can't fit more free transactions
       * to the block.
       **/
      FreeHeadersLimitExceded: AugmentedError<ApiType>;
      /**
       * The header (and its finality) submission overflows hardcoded chain limits: size
       * and/or weight are larger than expected.
       **/
      HeaderOverflowLimits: AugmentedError<ApiType>;
      /**
       * The authority set from the underlying header chain is invalid.
       **/
      InvalidAuthoritySet: AugmentedError<ApiType>;
      /**
       * The `current_set_id` argument of the `submit_finality_proof_ex` doesn't match
       * the id of the current set, known to the pallet.
       **/
      InvalidAuthoritySetId: AugmentedError<ApiType>;
      /**
       * The given justification is invalid for the given header.
       **/
      InvalidJustification: AugmentedError<ApiType>;
      /**
       * The pallet is not yet initialized.
       **/
      NotInitialized: AugmentedError<ApiType>;
      /**
       * The header being imported is older than the best finalized header known to the pallet.
       **/
      OldHeader: AugmentedError<ApiType>;
      /**
       * Too many authorities in the set.
       **/
      TooManyAuthoritiesInSet: AugmentedError<ApiType>;
      /**
       * The scheduled authority set change found in the header is unsupported by the pallet.
       *
       * This is the case for non-standard (e.g forced) authority set changes.
       **/
      UnsupportedScheduledChange: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bridgeKusamaMessages: {
      /**
       * Error generated by the `OwnedBridgeModule` trait.
       **/
      BridgeModule: AugmentedError<ApiType>;
      /**
       * The cumulative dispatch weight, passed by relayer is not enough to cover dispatch
       * of all bundled messages.
       **/
      InsufficientDispatchWeight: AugmentedError<ApiType>;
      /**
       * Invalid messages delivery proof has been submitted.
       **/
      InvalidMessagesDeliveryProof: AugmentedError<ApiType>;
      /**
       * Invalid messages has been submitted.
       **/
      InvalidMessagesProof: AugmentedError<ApiType>;
      /**
       * The relayer has declared invalid unrewarded relayers state in the
       * `receive_messages_delivery_proof` call.
       **/
      InvalidUnrewardedRelayersState: AugmentedError<ApiType>;
      /**
       * Error that is reported by the lanes manager.
       **/
      LanesManager: AugmentedError<ApiType>;
      /**
       * Message has been treated as invalid by the pallet logic.
       **/
      MessageRejectedByPallet: AugmentedError<ApiType>;
      /**
       * Pallet is not in Normal operating mode.
       **/
      NotOperatingNormally: AugmentedError<ApiType>;
      /**
       * Error confirming messages receival.
       **/
      ReceptionConfirmation: AugmentedError<ApiType>;
      /**
       * The transaction brings too many messages.
       **/
      TooManyMessagesInTheProof: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bridgeKusamaParachains: {
      /**
       * Error generated by the `OwnedBridgeModule` trait.
       **/
      BridgeModule: AugmentedError<ApiType>;
      /**
       * Parachain heads storage proof is invalid.
       **/
      HeaderChainStorageProof: AugmentedError<ApiType>;
      /**
       * The number of stored relay block is different from what the relayer has provided.
       **/
      InvalidRelayChainBlockNumber: AugmentedError<ApiType>;
      /**
       * Relay chain block hash is unknown to us.
       **/
      UnknownRelayChainBlock: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    bridgeXcmOverMoonriver: {
      /**
       * Trying to close already closed bridge.
       **/
      BridgeAlreadyClosed: AugmentedError<ApiType>;
      /**
       * The bridge is already registered in this pallet.
       **/
      BridgeAlreadyExists: AugmentedError<ApiType>;
      /**
       * Bridge locations error.
       **/
      BridgeLocations: AugmentedError<ApiType>;
      /**
       * The bridge origin can't pay the required amount for opening the bridge.
       **/
      FailedToReserveBridgeDeposit: AugmentedError<ApiType>;
      /**
       * Invalid local bridge origin account.
       **/
      InvalidBridgeOriginAccount: AugmentedError<ApiType>;
      /**
       * Lanes manager error.
       **/
      LanesManager: AugmentedError<ApiType>;
      /**
       * The local origin already owns a maximal number of bridges.
       **/
      TooManyBridgesForLocalOrigin: AugmentedError<ApiType>;
      /**
       * Trying to access unknown bridge.
       **/
      UnknownBridge: AugmentedError<ApiType>;
      /**
       * The version of XCM location argument is unsupported.
       **/
      UnsupportedXcmVersion: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    convictionVoting: {
      /**
       * The account is already delegating.
       **/
      AlreadyDelegating: AugmentedError<ApiType>;
      /**
       * The account currently has votes attached to it and the operation cannot succeed until
       * these are removed through `remove_vote`.
       **/
      AlreadyVoting: AugmentedError<ApiType>;
      /**
       * The class ID supplied is invalid.
       **/
      BadClass: AugmentedError<ApiType>;
      /**
       * The class must be supplied since it is not easily determinable from the state.
       **/
      ClassNeeded: AugmentedError<ApiType>;
      /**
       * Too high a balance was provided that the account cannot afford.
       **/
      InsufficientFunds: AugmentedError<ApiType>;
      /**
       * Maximum number of votes reached.
       **/
      MaxVotesReached: AugmentedError<ApiType>;
      /**
       * Delegation to oneself makes no sense.
       **/
      Nonsense: AugmentedError<ApiType>;
      /**
       * The actor has no permission to conduct the action.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * The actor has no permission to conduct the action right now but will do in the future.
       **/
      NoPermissionYet: AugmentedError<ApiType>;
      /**
       * The account is not currently delegating.
       **/
      NotDelegating: AugmentedError<ApiType>;
      /**
       * Poll is not ongoing.
       **/
      NotOngoing: AugmentedError<ApiType>;
      /**
       * The given account did not vote on the poll.
       **/
      NotVoter: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    crowdloanRewards: {
      /**
       * User trying to associate a native identity with a relay chain identity for posterior
       * reward claiming provided an already associated relay chain identity
       **/
      AlreadyAssociated: AugmentedError<ApiType>;
      /**
       * Trying to introduce a batch that goes beyond the limits of the funds
       **/
      BatchBeyondFundPot: AugmentedError<ApiType>;
      /**
       * First claim already done
       **/
      FirstClaimAlreadyDone: AugmentedError<ApiType>;
      /**
       * User submitted an unsifficient number of proofs to change the reward address
       **/
      InsufficientNumberOfValidProofs: AugmentedError<ApiType>;
      /**
       * User trying to associate a native identity with a relay chain identity for posterior
       * reward claiming provided a wrong signature
       **/
      InvalidClaimSignature: AugmentedError<ApiType>;
      /**
       * User trying to claim the first free reward provided the wrong signature
       **/
      InvalidFreeClaimSignature: AugmentedError<ApiType>;
      /**
       * User trying to claim an award did not have an claim associated with it. This may mean
       * they did not contribute to the crowdloan, or they have not yet associated a native id
       * with their contribution
       **/
      NoAssociatedClaim: AugmentedError<ApiType>;
      /**
       * User provided a signature from a non-contributor relay account
       **/
      NonContributedAddressProvided: AugmentedError<ApiType>;
      /**
       * The contribution is not high enough to be eligible for rewards
       **/
      RewardNotHighEnough: AugmentedError<ApiType>;
      /**
       * User trying to claim rewards has already claimed all rewards associated with its
       * identity and contribution
       **/
      RewardsAlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Rewards should match funds of the pallet
       **/
      RewardsDoNotMatchFund: AugmentedError<ApiType>;
      /**
       * Reward vec has already been initialized
       **/
      RewardVecAlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Reward vec has not yet been fully initialized
       **/
      RewardVecNotFullyInitializedYet: AugmentedError<ApiType>;
      /**
       * Initialize_reward_vec received too many contributors
       **/
      TooManyContributors: AugmentedError<ApiType>;
      /**
       * Provided vesting period is not valid
       **/
      VestingPeriodNonValid: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    emergencyParaXcm: {
      /**
       * The current XCM Mode is not Paused
       **/
      NotInPausedMode: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    ethereum: {
      /**
       * Signature is invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Pre-log is present, therefore transact is not allowed.
       **/
      PreLogExists: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    ethereumXcm: {
      /**
       * Xcm to Ethereum execution is suspended
       **/
      EthereumXcmExecutionSuspended: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    evm: {
      /**
       * Not enough balance to perform action
       **/
      BalanceLow: AugmentedError<ApiType>;
      /**
       * Calculating total fee overflowed
       **/
      FeeOverflow: AugmentedError<ApiType>;
      /**
       * Gas limit is too high.
       **/
      GasLimitTooHigh: AugmentedError<ApiType>;
      /**
       * Gas limit is too low.
       **/
      GasLimitTooLow: AugmentedError<ApiType>;
      /**
       * Gas price is too low.
       **/
      GasPriceTooLow: AugmentedError<ApiType>;
      /**
       * The chain id is invalid.
       **/
      InvalidChainId: AugmentedError<ApiType>;
      /**
       * Nonce is invalid
       **/
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * the signature is invalid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Calculating total payment overflowed
       **/
      PaymentOverflow: AugmentedError<ApiType>;
      /**
       * EVM reentrancy
       **/
      Reentrancy: AugmentedError<ApiType>;
      /**
       * EIP-3607,
       **/
      TransactionMustComeFromEOA: AugmentedError<ApiType>;
      /**
       * Undefined error.
       **/
      Undefined: AugmentedError<ApiType>;
      /**
       * Withdraw fee failed
       **/
      WithdrawFailed: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    evmForeignAssets: {
      AssetAlreadyExists: AugmentedError<ApiType>;
      AssetAlreadyFrozen: AugmentedError<ApiType>;
      AssetDoesNotExist: AugmentedError<ApiType>;
      AssetIdFiltered: AugmentedError<ApiType>;
      AssetNotFrozen: AugmentedError<ApiType>;
      AssetNotInSiblingPara: AugmentedError<ApiType>;
      CannotConvertLocationToAccount: AugmentedError<ApiType>;
      CorruptedStorageOrphanLocation: AugmentedError<ApiType>;
      Erc20ContractCreationFail: AugmentedError<ApiType>;
      EvmCallMintIntoFail: AugmentedError<ApiType>;
      EvmCallPauseFail: AugmentedError<ApiType>;
      EvmCallTransferFail: AugmentedError<ApiType>;
      EvmCallUnpauseFail: AugmentedError<ApiType>;
      EvmInternalError: AugmentedError<ApiType>;
      /**
       * Account has insufficient balance for locking
       **/
      InsufficientBalance: AugmentedError<ApiType>;
      InvalidSymbol: AugmentedError<ApiType>;
      InvalidTokenName: AugmentedError<ApiType>;
      LocationAlreadyExists: AugmentedError<ApiType>;
      LocationOutsideOfOrigin: AugmentedError<ApiType>;
      TooManyForeignAssets: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    identity: {
      /**
       * Account ID is already named.
       **/
      AlreadyClaimed: AugmentedError<ApiType>;
      /**
       * The username cannot be unbound because it is already unbinding.
       **/
      AlreadyUnbinding: AugmentedError<ApiType>;
      /**
       * Empty index.
       **/
      EmptyIndex: AugmentedError<ApiType>;
      /**
       * Fee is changed.
       **/
      FeeChanged: AugmentedError<ApiType>;
      /**
       * The action cannot be performed because of insufficient privileges (e.g. authority
       * trying to unbind a username provided by the system).
       **/
      InsufficientPrivileges: AugmentedError<ApiType>;
      /**
       * The index is invalid.
       **/
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * Invalid judgement.
       **/
      InvalidJudgement: AugmentedError<ApiType>;
      /**
       * The signature on a username was not valid.
       **/
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * The provided suffix is too long.
       **/
      InvalidSuffix: AugmentedError<ApiType>;
      /**
       * The target is invalid.
       **/
      InvalidTarget: AugmentedError<ApiType>;
      /**
       * The username does not meet the requirements.
       **/
      InvalidUsername: AugmentedError<ApiType>;
      /**
       * The provided judgement was for a different identity.
       **/
      JudgementForDifferentIdentity: AugmentedError<ApiType>;
      /**
       * Judgement given.
       **/
      JudgementGiven: AugmentedError<ApiType>;
      /**
       * Error that occurs when there is an issue paying for judgement.
       **/
      JudgementPaymentFailed: AugmentedError<ApiType>;
      /**
       * The authority cannot allocate any more usernames.
       **/
      NoAllocation: AugmentedError<ApiType>;
      /**
       * No identity found.
       **/
      NoIdentity: AugmentedError<ApiType>;
      /**
       * The username cannot be forcefully removed because it can still be accepted.
       **/
      NotExpired: AugmentedError<ApiType>;
      /**
       * Account isn't found.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Account isn't named.
       **/
      NotNamed: AugmentedError<ApiType>;
      /**
       * Sub-account isn't owned by sender.
       **/
      NotOwned: AugmentedError<ApiType>;
      /**
       * Sender is not a sub-account.
       **/
      NotSub: AugmentedError<ApiType>;
      /**
       * The username cannot be removed because it is not unbinding.
       **/
      NotUnbinding: AugmentedError<ApiType>;
      /**
       * The sender does not have permission to issue a username.
       **/
      NotUsernameAuthority: AugmentedError<ApiType>;
      /**
       * The requested username does not exist.
       **/
      NoUsername: AugmentedError<ApiType>;
      /**
       * Setting this username requires a signature, but none was provided.
       **/
      RequiresSignature: AugmentedError<ApiType>;
      /**
       * Sticky judgement.
       **/
      StickyJudgement: AugmentedError<ApiType>;
      /**
       * The username cannot be removed because it's still in the grace period.
       **/
      TooEarly: AugmentedError<ApiType>;
      /**
       * Maximum amount of registrars reached. Cannot add any more.
       **/
      TooManyRegistrars: AugmentedError<ApiType>;
      /**
       * Too many subs-accounts.
       **/
      TooManySubAccounts: AugmentedError<ApiType>;
      /**
       * The username is already taken.
       **/
      UsernameTaken: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    maintenanceMode: {
      /**
       * The chain cannot enter maintenance mode because it is already in maintenance mode
       **/
      AlreadyInMaintenanceMode: AugmentedError<ApiType>;
      /**
       * The chain cannot resume normal operation because it is not in maintenance mode
       **/
      NotInMaintenanceMode: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    messageQueue: {
      /**
       * The message was already processed and cannot be processed again.
       **/
      AlreadyProcessed: AugmentedError<ApiType>;
      /**
       * There is temporarily not enough weight to continue servicing messages.
       **/
      InsufficientWeight: AugmentedError<ApiType>;
      /**
       * The referenced message could not be found.
       **/
      NoMessage: AugmentedError<ApiType>;
      /**
       * Page to be reaped does not exist.
       **/
      NoPage: AugmentedError<ApiType>;
      /**
       * Page is not reapable because it has items remaining to be processed and is not old
       * enough.
       **/
      NotReapable: AugmentedError<ApiType>;
      /**
       * The message is queued for future execution.
       **/
      Queued: AugmentedError<ApiType>;
      /**
       * The queue is paused and no message can be executed from it.
       *
       * This can change at any time and may resolve in the future by re-trying.
       **/
      QueuePaused: AugmentedError<ApiType>;
      /**
       * Another call is in progress and needs to finish before this call can happen.
       **/
      RecursiveDisallowed: AugmentedError<ApiType>;
      /**
       * This message is temporarily unprocessable.
       *
       * Such errors are expected, but not guaranteed, to resolve themselves eventually through
       * retrying.
       **/
      TemporarilyUnprocessable: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    migrations: {
      /**
       * Preimage already exists in the new storage.
       **/
      PreimageAlreadyExists: AugmentedError<ApiType>;
      /**
       * Preimage is larger than the new max size.
       **/
      PreimageIsTooBig: AugmentedError<ApiType>;
      /**
       * Missing preimage in original democracy storage
       **/
      PreimageMissing: AugmentedError<ApiType>;
      /**
       * Provided upper bound is too low.
       **/
      WrongUpperBound: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    moonbeamLazyMigrations: {
      /**
       * The contract already have metadata
       **/
      ContractMetadataAlreadySet: AugmentedError<ApiType>;
      /**
       * Contract not exist
       **/
      ContractNotExist: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    moonbeamOrbiters: {
      /**
       * The collator is already added in orbiters program.
       **/
      CollatorAlreadyAdded: AugmentedError<ApiType>;
      /**
       * This collator is not in orbiters program.
       **/
      CollatorNotFound: AugmentedError<ApiType>;
      /**
       * There are already too many orbiters associated with this collator.
       **/
      CollatorPoolTooLarge: AugmentedError<ApiType>;
      /**
       * There are more collator pools than the number specified in the parameter.
       **/
      CollatorsPoolCountTooLow: AugmentedError<ApiType>;
      /**
       * The minimum deposit required to register as an orbiter has not yet been included in the
       * onchain storage
       **/
      MinOrbiterDepositNotSet: AugmentedError<ApiType>;
      /**
       * This orbiter is already associated with this collator.
       **/
      OrbiterAlreadyInPool: AugmentedError<ApiType>;
      /**
       * This orbiter has not made a deposit
       **/
      OrbiterDepositNotFound: AugmentedError<ApiType>;
      /**
       * This orbiter is not found
       **/
      OrbiterNotFound: AugmentedError<ApiType>;
      /**
       * The orbiter is still at least in one pool
       **/
      OrbiterStillInAPool: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    multiBlockMigrations: {
      /**
       * The operation cannot complete since some MBMs are ongoing.
       **/
      Ongoing: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    multisig: {
      /**
       * Call is already approved by this signatory.
       **/
      AlreadyApproved: AugmentedError<ApiType>;
      /**
       * The data to be stored is already stored.
       **/
      AlreadyStored: AugmentedError<ApiType>;
      /**
       * The maximum weight information provided was too low.
       **/
      MaxWeightTooLow: AugmentedError<ApiType>;
      /**
       * Threshold must be 2 or greater.
       **/
      MinimumThreshold: AugmentedError<ApiType>;
      /**
       * Call doesn't need any (more) approvals.
       **/
      NoApprovalsNeeded: AugmentedError<ApiType>;
      /**
       * Multisig operation not found when attempting to cancel.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * No timepoint was given, yet the multisig operation is already underway.
       **/
      NoTimepoint: AugmentedError<ApiType>;
      /**
       * Only the account that originally created the multisig is able to cancel it.
       **/
      NotOwner: AugmentedError<ApiType>;
      /**
       * The sender was contained in the other signatories; it shouldn't be.
       **/
      SenderInSignatories: AugmentedError<ApiType>;
      /**
       * The signatories were provided out of order; they should be ordered.
       **/
      SignatoriesOutOfOrder: AugmentedError<ApiType>;
      /**
       * There are too few signatories in the list.
       **/
      TooFewSignatories: AugmentedError<ApiType>;
      /**
       * There are too many signatories in the list.
       **/
      TooManySignatories: AugmentedError<ApiType>;
      /**
       * A timepoint was given, yet no multisig operation is underway.
       **/
      UnexpectedTimepoint: AugmentedError<ApiType>;
      /**
       * A different timepoint was given to the multisig operation that is underway.
       **/
      WrongTimepoint: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    openTechCommitteeCollective: {
      /**
       * Members are already initialized!
       **/
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Duplicate proposals not allowed
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate vote ignored
       **/
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * Account is not a member
       **/
      NotMember: AugmentedError<ApiType>;
      /**
       * Prime account is not a member
       **/
      PrimeAccountNotMember: AugmentedError<ApiType>;
      /**
       * Proposal is still active.
       **/
      ProposalActive: AugmentedError<ApiType>;
      /**
       * Proposal must exist
       **/
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * The close call was made too early, before the end of the voting.
       **/
      TooEarly: AugmentedError<ApiType>;
      /**
       * There can only be a maximum of `MaxProposals` active proposals.
       **/
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Mismatched index
       **/
      WrongIndex: AugmentedError<ApiType>;
      /**
       * The given length bound for the proposal was too low.
       **/
      WrongProposalLength: AugmentedError<ApiType>;
      /**
       * The given weight bound for the proposal was too low.
       **/
      WrongProposalWeight: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    parachainStaking: {
      AlreadyActive: AugmentedError<ApiType>;
      AlreadyDelegatedCandidate: AugmentedError<ApiType>;
      AlreadyOffline: AugmentedError<ApiType>;
      CandidateAlreadyLeaving: AugmentedError<ApiType>;
      CandidateBondBelowMin: AugmentedError<ApiType>;
      CandidateCannotLeaveYet: AugmentedError<ApiType>;
      CandidateDNE: AugmentedError<ApiType>;
      CandidateExists: AugmentedError<ApiType>;
      CandidateLimitReached: AugmentedError<ApiType>;
      CandidateNotLeaving: AugmentedError<ApiType>;
      CannotBeNotifiedAsInactive: AugmentedError<ApiType>;
      CannotDelegateIfLeaving: AugmentedError<ApiType>;
      CannotDelegateLessThanOrEqualToLowestBottomWhenFull: AugmentedError<ApiType>;
      CannotGoOnlineIfLeaving: AugmentedError<ApiType>;
      CannotSetAboveMaxCandidates: AugmentedError<ApiType>;
      CannotSetBelowMin: AugmentedError<ApiType>;
      CurrentRoundTooLow: AugmentedError<ApiType>;
      DelegationBelowMin: AugmentedError<ApiType>;
      DelegationDNE: AugmentedError<ApiType>;
      DelegatorAlreadyLeaving: AugmentedError<ApiType>;
      DelegatorBondBelowMin: AugmentedError<ApiType>;
      DelegatorCannotLeaveYet: AugmentedError<ApiType>;
      DelegatorDNE: AugmentedError<ApiType>;
      DelegatorDNEInDelegatorSet: AugmentedError<ApiType>;
      DelegatorDNEinTopNorBottom: AugmentedError<ApiType>;
      DelegatorExists: AugmentedError<ApiType>;
      DelegatorNotLeaving: AugmentedError<ApiType>;
      ExceedMaxDelegationsPerDelegator: AugmentedError<ApiType>;
      InsufficientBalance: AugmentedError<ApiType>;
      InvalidSchedule: AugmentedError<ApiType>;
      MarkingOfflineNotEnabled: AugmentedError<ApiType>;
      NoWritingSameValue: AugmentedError<ApiType>;
      PendingCandidateRequestAlreadyExists: AugmentedError<ApiType>;
      PendingCandidateRequestNotDueYet: AugmentedError<ApiType>;
      PendingCandidateRequestsDNE: AugmentedError<ApiType>;
      PendingDelegationRequestAlreadyExists: AugmentedError<ApiType>;
      PendingDelegationRequestDNE: AugmentedError<ApiType>;
      PendingDelegationRequestNotDueYet: AugmentedError<ApiType>;
      PendingDelegationRevoke: AugmentedError<ApiType>;
      RoundLengthMustBeGreaterThanTotalSelectedCollators: AugmentedError<ApiType>;
      TooLowCandidateAutoCompoundingDelegationCountToAutoCompound: AugmentedError<ApiType>;
      TooLowCandidateAutoCompoundingDelegationCountToDelegate: AugmentedError<ApiType>;
      TooLowCandidateAutoCompoundingDelegationCountToLeaveCandidates: AugmentedError<ApiType>;
      TooLowCandidateCountToLeaveCandidates: AugmentedError<ApiType>;
      TooLowCandidateCountWeightHint: AugmentedError<ApiType>;
      TooLowCandidateCountWeightHintCancelLeaveCandidates: AugmentedError<ApiType>;
      TooLowCandidateCountWeightHintGoOffline: AugmentedError<ApiType>;
      TooLowCandidateCountWeightHintJoinCandidates: AugmentedError<ApiType>;
      TooLowCandidateDelegationCountToDelegate: AugmentedError<ApiType>;
      TooLowCandidateDelegationCountToLeaveCandidates: AugmentedError<ApiType>;
      TooLowCollatorCountToNotifyAsInactive: AugmentedError<ApiType>;
      TooLowDelegationCountToAutoCompound: AugmentedError<ApiType>;
      TooLowDelegationCountToDelegate: AugmentedError<ApiType>;
      TooLowDelegationCountToLeaveDelegators: AugmentedError<ApiType>;
      TotalInflationDistributionPercentExceeds100: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    parachainSystem: {
      /**
       * The inherent which supplies the host configuration did not run this block.
       **/
      HostConfigurationNotAvailable: AugmentedError<ApiType>;
      /**
       * No code upgrade has been authorized.
       **/
      NothingAuthorized: AugmentedError<ApiType>;
      /**
       * No validation function upgrade is currently scheduled.
       **/
      NotScheduled: AugmentedError<ApiType>;
      /**
       * Attempt to upgrade validation function while existing upgrade pending.
       **/
      OverlappingUpgrades: AugmentedError<ApiType>;
      /**
       * Polkadot currently prohibits this parachain from upgrading its validation function.
       **/
      ProhibitedByPolkadot: AugmentedError<ApiType>;
      /**
       * The supplied validation function has compiled into a blob larger than Polkadot is
       * willing to run.
       **/
      TooBig: AugmentedError<ApiType>;
      /**
       * The given code upgrade has not been authorized.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * The inherent which supplies the validation data did not run this block.
       **/
      ValidationDataNotAvailable: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    polkadotXcm: {
      /**
       * The given account is not an identifiable sovereign account for any location.
       **/
      AccountNotSovereign: AugmentedError<ApiType>;
      /**
       * The location is invalid since it already has a subscription from us.
       **/
      AlreadySubscribed: AugmentedError<ApiType>;
      /**
       * The given location could not be used (e.g. because it cannot be expressed in the
       * desired version of XCM).
       **/
      BadLocation: AugmentedError<ApiType>;
      /**
       * The version of the `Versioned` value used is not able to be interpreted.
       **/
      BadVersion: AugmentedError<ApiType>;
      /**
       * Could not check-out the assets for teleportation to the destination chain.
       **/
      CannotCheckOutTeleport: AugmentedError<ApiType>;
      /**
       * Could not re-anchor the assets to declare the fees for the destination chain.
       **/
      CannotReanchor: AugmentedError<ApiType>;
      /**
       * The destination `Location` provided cannot be inverted.
       **/
      DestinationNotInvertible: AugmentedError<ApiType>;
      /**
       * The assets to be sent are empty.
       **/
      Empty: AugmentedError<ApiType>;
      /**
       * The operation required fees to be paid which the initiator could not meet.
       **/
      FeesNotMet: AugmentedError<ApiType>;
      /**
       * The message execution fails the filter.
       **/
      Filtered: AugmentedError<ApiType>;
      /**
       * The unlock operation cannot succeed because there are still consumers of the lock.
       **/
      InUse: AugmentedError<ApiType>;
      /**
       * Invalid asset, reserve chain could not be determined for it.
       **/
      InvalidAssetUnknownReserve: AugmentedError<ApiType>;
      /**
       * Invalid asset, do not support remote asset reserves with different fees reserves.
       **/
      InvalidAssetUnsupportedReserve: AugmentedError<ApiType>;
      /**
       * Origin is invalid for sending.
       **/
      InvalidOrigin: AugmentedError<ApiType>;
      /**
       * Local XCM execution incomplete.
       **/
      LocalExecutionIncomplete: AugmentedError<ApiType>;
      /**
       * A remote lock with the corresponding data could not be found.
       **/
      LockNotFound: AugmentedError<ApiType>;
      /**
       * The owner does not own (all) of the asset that they wish to do the operation on.
       **/
      LowBalance: AugmentedError<ApiType>;
      /**
       * The referenced subscription could not be found.
       **/
      NoSubscription: AugmentedError<ApiType>;
      /**
       * There was some other issue (i.e. not to do with routing) in sending the message.
       * Perhaps a lack of space for buffering the message.
       **/
      SendFailure: AugmentedError<ApiType>;
      /**
       * Too many assets have been attempted for transfer.
       **/
      TooManyAssets: AugmentedError<ApiType>;
      /**
       * The asset owner has too many locks on the asset.
       **/
      TooManyLocks: AugmentedError<ApiType>;
      /**
       * Too many assets with different reserve locations have been attempted for transfer.
       **/
      TooManyReserves: AugmentedError<ApiType>;
      /**
       * The desired destination was unreachable, generally because there is a no way of routing
       * to it.
       **/
      Unreachable: AugmentedError<ApiType>;
      /**
       * The message's weight could not be determined.
       **/
      UnweighableMessage: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    precompileBenchmarks: {
      BenchmarkError: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    preimage: {
      /**
       * Preimage has already been noted on-chain.
       **/
      AlreadyNoted: AugmentedError<ApiType>;
      /**
       * The user is not authorized to perform this action.
       **/
      NotAuthorized: AugmentedError<ApiType>;
      /**
       * The preimage cannot be removed since it has not yet been noted.
       **/
      NotNoted: AugmentedError<ApiType>;
      /**
       * The preimage request cannot be removed since no outstanding requests exist.
       **/
      NotRequested: AugmentedError<ApiType>;
      /**
       * A preimage may not be removed when there are outstanding requests.
       **/
      Requested: AugmentedError<ApiType>;
      /**
       * Preimage is too large to store on-chain.
       **/
      TooBig: AugmentedError<ApiType>;
      /**
       * Too few hashes were requested to be upgraded (i.e. zero).
       **/
      TooFew: AugmentedError<ApiType>;
      /**
       * More than `MAX_HASH_UPGRADE_BULK_COUNT` hashes were requested to be upgraded at once.
       **/
      TooMany: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    proxy: {
      /**
       * Account is already a proxy.
       **/
      Duplicate: AugmentedError<ApiType>;
      /**
       * Call may not be made by proxy because it may escalate its privileges.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * Cannot add self as proxy.
       **/
      NoSelfProxy: AugmentedError<ApiType>;
      /**
       * Proxy registration not found.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Sender is not a proxy of the account to be proxied.
       **/
      NotProxy: AugmentedError<ApiType>;
      /**
       * There are too many proxies registered or too many announcements pending.
       **/
      TooMany: AugmentedError<ApiType>;
      /**
       * Announcement, if made at all, was made too recently.
       **/
      Unannounced: AugmentedError<ApiType>;
      /**
       * A call which is incompatible with the proxy type's filter was attempted.
       **/
      Unproxyable: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    randomness: {
      CannotRequestMoreWordsThanMax: AugmentedError<ApiType>;
      CannotRequestRandomnessAfterMaxDelay: AugmentedError<ApiType>;
      CannotRequestRandomnessBeforeMinDelay: AugmentedError<ApiType>;
      MustRequestAtLeastOneWord: AugmentedError<ApiType>;
      OnlyRequesterCanIncreaseFee: AugmentedError<ApiType>;
      RandomnessResultDNE: AugmentedError<ApiType>;
      RandomnessResultNotFilled: AugmentedError<ApiType>;
      RequestCannotYetBeFulfilled: AugmentedError<ApiType>;
      RequestCounterOverflowed: AugmentedError<ApiType>;
      RequestDNE: AugmentedError<ApiType>;
      RequestFeeOverflowed: AugmentedError<ApiType>;
      RequestHasNotExpired: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    referenda: {
      /**
       * The referendum index provided is invalid in this context.
       **/
      BadReferendum: AugmentedError<ApiType>;
      /**
       * The referendum status is invalid for this operation.
       **/
      BadStatus: AugmentedError<ApiType>;
      /**
       * The track identifier given was invalid.
       **/
      BadTrack: AugmentedError<ApiType>;
      /**
       * There are already a full complement of referenda in progress for this track.
       **/
      Full: AugmentedError<ApiType>;
      /**
       * Referendum's decision deposit is already paid.
       **/
      HasDeposit: AugmentedError<ApiType>;
      /**
       * The deposit cannot be refunded since none was made.
       **/
      NoDeposit: AugmentedError<ApiType>;
      /**
       * The deposit refunder is not the depositor.
       **/
      NoPermission: AugmentedError<ApiType>;
      /**
       * There was nothing to do in the advancement.
       **/
      NothingToDo: AugmentedError<ApiType>;
      /**
       * Referendum is not ongoing.
       **/
      NotOngoing: AugmentedError<ApiType>;
      /**
       * No track exists for the proposal origin.
       **/
      NoTrack: AugmentedError<ApiType>;
      /**
       * The preimage does not exist.
       **/
      PreimageNotExist: AugmentedError<ApiType>;
      /**
       * The preimage is stored with a different length than the one provided.
       **/
      PreimageStoredWithDifferentLength: AugmentedError<ApiType>;
      /**
       * The queue of the track is empty.
       **/
      QueueEmpty: AugmentedError<ApiType>;
      /**
       * Any deposit cannot be refunded until after the decision is over.
       **/
      Unfinished: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    scheduler: {
      /**
       * Failed to schedule a call
       **/
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Attempt to use a non-named function on a named task.
       **/
      Named: AugmentedError<ApiType>;
      /**
       * Cannot find the scheduled call.
       **/
      NotFound: AugmentedError<ApiType>;
      /**
       * Reschedule failed because it does not change scheduled time.
       **/
      RescheduleNoChange: AugmentedError<ApiType>;
      /**
       * Given target block number is in the past.
       **/
      TargetBlockNumberInPast: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    system: {
      /**
       * The origin filter prevent the call to be dispatched.
       **/
      CallFiltered: AugmentedError<ApiType>;
      /**
       * Failed to extract the runtime version from the new runtime.
       *
       * Either calling `Core_version` or decoding `RuntimeVersion` failed.
       **/
      FailedToExtractRuntimeVersion: AugmentedError<ApiType>;
      /**
       * The name of specification does not match between the current runtime
       * and the new runtime.
       **/
      InvalidSpecName: AugmentedError<ApiType>;
      /**
       * A multi-block migration is ongoing and prevents the current code from being replaced.
       **/
      MultiBlockMigrationsOngoing: AugmentedError<ApiType>;
      /**
       * Suicide called when the account has non-default composite data.
       **/
      NonDefaultComposite: AugmentedError<ApiType>;
      /**
       * There is a non-zero reference count preventing the account from being purged.
       **/
      NonZeroRefCount: AugmentedError<ApiType>;
      /**
       * No upgrade authorized.
       **/
      NothingAuthorized: AugmentedError<ApiType>;
      /**
       * The specification version is not allowed to decrease between the current runtime
       * and the new runtime.
       **/
      SpecVersionNeedsToIncrease: AugmentedError<ApiType>;
      /**
       * The submitted code is not authorized.
       **/
      Unauthorized: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    treasury: {
      /**
       * The payment has already been attempted.
       **/
      AlreadyAttempted: AugmentedError<ApiType>;
      /**
       * The spend is not yet eligible for payout.
       **/
      EarlyPayout: AugmentedError<ApiType>;
      /**
       * The balance of the asset kind is not convertible to the balance of the native asset.
       **/
      FailedToConvertBalance: AugmentedError<ApiType>;
      /**
       * The payment has neither failed nor succeeded yet.
       **/
      Inconclusive: AugmentedError<ApiType>;
      /**
       * The spend origin is valid but the amount it is allowed to spend is lower than the
       * amount to be spent.
       **/
      InsufficientPermission: AugmentedError<ApiType>;
      /**
       * No proposal, bounty or spend at that index.
       **/
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * The payout was not yet attempted/claimed.
       **/
      NotAttempted: AugmentedError<ApiType>;
      /**
       * There was some issue with the mechanism of payment.
       **/
      PayoutError: AugmentedError<ApiType>;
      /**
       * Proposal has not been approved.
       **/
      ProposalNotApproved: AugmentedError<ApiType>;
      /**
       * The spend has expired and cannot be claimed.
       **/
      SpendExpired: AugmentedError<ApiType>;
      /**
       * Too many approvals in the queue.
       **/
      TooManyApprovals: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    treasuryCouncilCollective: {
      /**
       * Members are already initialized!
       **/
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Duplicate proposals not allowed
       **/
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate vote ignored
       **/
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * Account is not a member
       **/
      NotMember: AugmentedError<ApiType>;
      /**
       * Prime account is not a member
       **/
      PrimeAccountNotMember: AugmentedError<ApiType>;
      /**
       * Proposal is still active.
       **/
      ProposalActive: AugmentedError<ApiType>;
      /**
       * Proposal must exist
       **/
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * The close call was made too early, before the end of the voting.
       **/
      TooEarly: AugmentedError<ApiType>;
      /**
       * There can only be a maximum of `MaxProposals` active proposals.
       **/
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Mismatched index
       **/
      WrongIndex: AugmentedError<ApiType>;
      /**
       * The given length bound for the proposal was too low.
       **/
      WrongProposalLength: AugmentedError<ApiType>;
      /**
       * The given weight bound for the proposal was too low.
       **/
      WrongProposalWeight: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    utility: {
      /**
       * Too many calls batched.
       **/
      TooManyCalls: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    whitelist: {
      /**
       * The call was already whitelisted; No-Op.
       **/
      CallAlreadyWhitelisted: AugmentedError<ApiType>;
      /**
       * The call was not whitelisted.
       **/
      CallIsNotWhitelisted: AugmentedError<ApiType>;
      /**
       * The weight of the decoded call was higher than the witness.
       **/
      InvalidCallWeightWitness: AugmentedError<ApiType>;
      /**
       * The preimage of the call hash could not be loaded.
       **/
      UnavailablePreImage: AugmentedError<ApiType>;
      /**
       * The call could not be decoded.
       **/
      UndecodableCall: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    xcmpQueue: {
      /**
       * The execution is already resumed.
       **/
      AlreadyResumed: AugmentedError<ApiType>;
      /**
       * The execution is already suspended.
       **/
      AlreadySuspended: AugmentedError<ApiType>;
      /**
       * Setting the queue config failed since one of its values was invalid.
       **/
      BadQueueConfig: AugmentedError<ApiType>;
      /**
       * The message is too big.
       **/
      TooBig: AugmentedError<ApiType>;
      /**
       * There are too many active outbound channels.
       **/
      TooManyActiveOutboundChannels: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    xcmTransactor: {
      AmountOverflow: AugmentedError<ApiType>;
      AssetHasNoReserve: AugmentedError<ApiType>;
      AssetIsNotReserveInDestination: AugmentedError<ApiType>;
      BadVersion: AugmentedError<ApiType>;
      CannotReanchor: AugmentedError<ApiType>;
      DestinationNotInvertible: AugmentedError<ApiType>;
      DispatchWeightBiggerThanTotalWeight: AugmentedError<ApiType>;
      ErrorDelivering: AugmentedError<ApiType>;
      ErrorValidating: AugmentedError<ApiType>;
      FailedMultiLocationToJunction: AugmentedError<ApiType>;
      FeePerSecondNotSet: AugmentedError<ApiType>;
      HrmpHandlerNotImplemented: AugmentedError<ApiType>;
      IndexAlreadyClaimed: AugmentedError<ApiType>;
      InvalidDest: AugmentedError<ApiType>;
      MaxWeightTransactReached: AugmentedError<ApiType>;
      NotCrossChainTransfer: AugmentedError<ApiType>;
      NotCrossChainTransferableCurrency: AugmentedError<ApiType>;
      NotOwner: AugmentedError<ApiType>;
      RefundNotSupportedWithTransactInfo: AugmentedError<ApiType>;
      SignedTransactNotAllowedForDestination: AugmentedError<ApiType>;
      TooMuchFeeUsed: AugmentedError<ApiType>;
      TransactorInfoNotSet: AugmentedError<ApiType>;
      UnableToWithdrawAsset: AugmentedError<ApiType>;
      UnclaimedIndex: AugmentedError<ApiType>;
      UnweighableMessage: AugmentedError<ApiType>;
      WeightOverflow: AugmentedError<ApiType>;
      XcmExecuteError: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
    xcmWeightTrader: {
      /**
       * The given asset was already added
       **/
      AssetAlreadyAdded: AugmentedError<ApiType>;
      /**
       * The given asset was already paused
       **/
      AssetAlreadyPaused: AugmentedError<ApiType>;
      /**
       * The given asset was not found
       **/
      AssetNotFound: AugmentedError<ApiType>;
      /**
       * The given asset is not paused
       **/
      AssetNotPaused: AugmentedError<ApiType>;
      /**
       * The relative price cannot be zero
       **/
      PriceCannotBeZero: AugmentedError<ApiType>;
      /**
       * The relative price calculation overflowed
       **/
      PriceOverflow: AugmentedError<ApiType>;
      /**
       * XCM location filtered
       **/
      XcmLocationFiltered: AugmentedError<ApiType>;
      /**
       * Generic error
       **/
      [key: string]: AugmentedError<ApiType>;
    };
  } // AugmentedErrors
} // declare module
