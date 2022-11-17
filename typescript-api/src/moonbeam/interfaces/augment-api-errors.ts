// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from "@polkadot/api-base/types";

declare module "@polkadot/api-base/types/errors" {
  export interface AugmentedErrors<ApiType extends ApiTypes> {
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
       */
      [key: string]: AugmentedError<ApiType>;
    };
    assets: {
      /**
       * The asset-account already exists.
       */
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * Invalid metadata given.
       */
      BadMetadata: AugmentedError<ApiType>;
      /**
       * Invalid witness data given.
       */
      BadWitness: AugmentedError<ApiType>;
      /**
       * Account balance must be greater than or equal to the transfer amount.
       */
      BalanceLow: AugmentedError<ApiType>;
      /**
       * The origin account is frozen.
       */
      Frozen: AugmentedError<ApiType>;
      /**
       * The asset ID is already taken.
       */
      InUse: AugmentedError<ApiType>;
      /**
       * Minimum balance should be non-zero.
       */
      MinBalanceZero: AugmentedError<ApiType>;
      /**
       * The account to alter does not exist.
       */
      NoAccount: AugmentedError<ApiType>;
      /**
       * The asset-account doesn't have an associated deposit.
       */
      NoDeposit: AugmentedError<ApiType>;
      /**
       * The signing account has no permission to do the operation.
       */
      NoPermission: AugmentedError<ApiType>;
      /**
       * Unable to increment the consumer reference counters on the account.
       * Either no provider reference exists to allow a non-zero balance of a
       * non-self-sufficient asset, or the maximum number of consumers has been reached.
       */
      NoProvider: AugmentedError<ApiType>;
      /**
       * No approval exists that would allow the transfer.
       */
      Unapproved: AugmentedError<ApiType>;
      /**
       * The given asset ID is unknown.
       */
      Unknown: AugmentedError<ApiType>;
      /**
       * The operation would result in funds being burned.
       */
      WouldBurn: AugmentedError<ApiType>;
      /**
       * The source account would not survive the transfer and it needs to stay alive.
       */
      WouldDie: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    authorInherent: {
      /**
       * Author already set in block.
       */
      AuthorAlreadySet: AugmentedError<ApiType>;
      /**
       * The author in the inherent is not an eligible author.
       */
      CannotBeAuthor: AugmentedError<ApiType>;
      /**
       * No AccountId was found to be associated with this author
       */
      NoAccountId: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    authorMapping: {
      /**
       * The NimbusId in question is already associated and cannot be overwritten
       */
      AlreadyAssociated: AugmentedError<ApiType>;
      /**
       * The association can't be cleared because it is not found.
       */
      AssociationNotFound: AugmentedError<ApiType>;
      /**
       * This account cannot set an author because it cannon afford the security deposit
       */
      CannotAffordSecurityDeposit: AugmentedError<ApiType>;
      /**
       * Failed to decode T::Keys for `set_keys`
       */
      DecodeKeysFailed: AugmentedError<ApiType>;
      /**
       * Failed to decode NimbusId for `set_keys`
       */
      DecodeNimbusFailed: AugmentedError<ApiType>;
      /**
       * The association can't be cleared because it belongs to another account.
       */
      NotYourAssociation: AugmentedError<ApiType>;
      /**
       * No existing NimbusId can be found for the account
       */
      OldAuthorIdNotFound: AugmentedError<ApiType>;
      /**
       * Keys have wrong size
       */
      WrongKeySize: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    balances: {
      /**
       * Beneficiary account must pre-exist
       */
      DeadAccount: AugmentedError<ApiType>;
      /**
       * Value too low to create account due to existential deposit
       */
      ExistentialDeposit: AugmentedError<ApiType>;
      /**
       * A vesting schedule already exists for this account
       */
      ExistingVestingSchedule: AugmentedError<ApiType>;
      /**
       * Balance too low to send value
       */
      InsufficientBalance: AugmentedError<ApiType>;
      /**
       * Transfer/payment would kill account
       */
      KeepAlive: AugmentedError<ApiType>;
      /**
       * Account liquidity restrictions prevent withdrawal
       */
      LiquidityRestrictions: AugmentedError<ApiType>;
      /**
       * Number of named reserves exceed MaxReserves
       */
      TooManyReserves: AugmentedError<ApiType>;
      /**
       * Vesting balance too high to send value
       */
      VestingBalance: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    councilCollective: {
      /**
       * Members are already initialized!
       */
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Duplicate proposals not allowed
       */
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate vote ignored
       */
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * Account is not a member
       */
      NotMember: AugmentedError<ApiType>;
      /**
       * Proposal must exist
       */
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * The close call was made too early, before the end of the voting.
       */
      TooEarly: AugmentedError<ApiType>;
      /**
       * There can only be a maximum of `MaxProposals` active proposals.
       */
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Mismatched index
       */
      WrongIndex: AugmentedError<ApiType>;
      /**
       * The given length bound for the proposal was too low.
       */
      WrongProposalLength: AugmentedError<ApiType>;
      /**
       * The given weight bound for the proposal was too low.
       */
      WrongProposalWeight: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    crowdloanRewards: {
      /**
       * User trying to associate a native identity with a relay chain identity
       * for posterior reward claiming provided an already associated relay
       * chain identity
       */
      AlreadyAssociated: AugmentedError<ApiType>;
      /**
       * Trying to introduce a batch that goes beyond the limits of the funds
       */
      BatchBeyondFundPot: AugmentedError<ApiType>;
      /**
       * First claim already done
       */
      FirstClaimAlreadyDone: AugmentedError<ApiType>;
      /**
       * User submitted an unsifficient number of proofs to change the reward address
       */
      InsufficientNumberOfValidProofs: AugmentedError<ApiType>;
      /**
       * User trying to associate a native identity with a relay chain identity
       * for posterior reward claiming provided a wrong signature
       */
      InvalidClaimSignature: AugmentedError<ApiType>;
      /**
       * User trying to claim the first free reward provided the wrong signature
       */
      InvalidFreeClaimSignature: AugmentedError<ApiType>;
      /**
       * User trying to claim an award did not have an claim associated with it.
       * This may mean they did not contribute to the crowdloan, or they have
       * not yet associated a native id with their contribution
       */
      NoAssociatedClaim: AugmentedError<ApiType>;
      /**
       * User provided a signature from a non-contributor relay account
       */
      NonContributedAddressProvided: AugmentedError<ApiType>;
      /**
       * The contribution is not high enough to be eligible for rewards
       */
      RewardNotHighEnough: AugmentedError<ApiType>;
      /**
       * User trying to claim rewards has already claimed all rewards associated
       * with its identity and contribution
       */
      RewardsAlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Rewards should match funds of the pallet
       */
      RewardsDoNotMatchFund: AugmentedError<ApiType>;
      /**
       * Reward vec has already been initialized
       */
      RewardVecAlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Reward vec has not yet been fully initialized
       */
      RewardVecNotFullyInitializedYet: AugmentedError<ApiType>;
      /**
       * Initialize_reward_vec received too many contributors
       */
      TooManyContributors: AugmentedError<ApiType>;
      /**
       * Provided vesting period is not valid
       */
      VestingPeriodNonValid: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    cumulusXcm: {
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    democracy: {
      /**
       * Cannot cancel the same proposal twice
       */
      AlreadyCanceled: AugmentedError<ApiType>;
      /**
       * The account is already delegating.
       */
      AlreadyDelegating: AugmentedError<ApiType>;
      /**
       * Identity may not veto a proposal twice
       */
      AlreadyVetoed: AugmentedError<ApiType>;
      /**
       * Preimage already noted
       */
      DuplicatePreimage: AugmentedError<ApiType>;
      /**
       * Proposal already made
       */
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Imminent
       */
      Imminent: AugmentedError<ApiType>;
      /**
       * The instant referendum origin is currently disallowed.
       */
      InstantNotAllowed: AugmentedError<ApiType>;
      /**
       * Too high a balance was provided that the account cannot afford.
       */
      InsufficientFunds: AugmentedError<ApiType>;
      /**
       * Invalid hash
       */
      InvalidHash: AugmentedError<ApiType>;
      /**
       * Maximum number of votes reached.
       */
      MaxVotesReached: AugmentedError<ApiType>;
      /**
       * No proposals waiting
       */
      NoneWaiting: AugmentedError<ApiType>;
      /**
       * Delegation to oneself makes no sense.
       */
      Nonsense: AugmentedError<ApiType>;
      /**
       * The actor has no permission to conduct the action.
       */
      NoPermission: AugmentedError<ApiType>;
      /**
       * No external proposal
       */
      NoProposal: AugmentedError<ApiType>;
      /**
       * The account is not currently delegating.
       */
      NotDelegating: AugmentedError<ApiType>;
      /**
       * Not imminent
       */
      NotImminent: AugmentedError<ApiType>;
      /**
       * Next external proposal not simple majority
       */
      NotSimpleMajority: AugmentedError<ApiType>;
      /**
       * The given account did not vote on the referendum.
       */
      NotVoter: AugmentedError<ApiType>;
      /**
       * Invalid preimage
       */
      PreimageInvalid: AugmentedError<ApiType>;
      /**
       * Preimage not found
       */
      PreimageMissing: AugmentedError<ApiType>;
      /**
       * Proposal still blacklisted
       */
      ProposalBlacklisted: AugmentedError<ApiType>;
      /**
       * Proposal does not exist
       */
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * Vote given for invalid referendum
       */
      ReferendumInvalid: AugmentedError<ApiType>;
      /**
       * Too early
       */
      TooEarly: AugmentedError<ApiType>;
      /**
       * Maximum number of proposals reached.
       */
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Value too low
       */
      ValueLow: AugmentedError<ApiType>;
      /**
       * The account currently has votes attached to it and the operation cannot
       * succeed until these are removed, either through `unvote` or `reap_vote`.
       */
      VotesExist: AugmentedError<ApiType>;
      /**
       * Voting period too low
       */
      VotingPeriodLow: AugmentedError<ApiType>;
      /**
       * Invalid upper bound.
       */
      WrongUpperBound: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    dmpQueue: {
      /**
       * The amount of weight given is possibly not enough for executing the message.
       */
      OverLimit: AugmentedError<ApiType>;
      /**
       * The message index given is unknown.
       */
      Unknown: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    ethereum: {
      /**
       * Signature is invalid.
       */
      InvalidSignature: AugmentedError<ApiType>;
      /**
       * Pre-log is present, therefore transact is not allowed.
       */
      PreLogExists: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    evm: {
      /**
       * Not enough balance to perform action
       */
      BalanceLow: AugmentedError<ApiType>;
      /**
       * Calculating total fee overflowed
       */
      FeeOverflow: AugmentedError<ApiType>;
      /**
       * Gas limit is too high.
       */
      GasLimitTooHigh: AugmentedError<ApiType>;
      /**
       * Gas limit is too low.
       */
      GasLimitTooLow: AugmentedError<ApiType>;
      /**
       * Gas price is too low.
       */
      GasPriceTooLow: AugmentedError<ApiType>;
      /**
       * Nonce is invalid
       */
      InvalidNonce: AugmentedError<ApiType>;
      /**
       * Calculating total payment overflowed
       */
      PaymentOverflow: AugmentedError<ApiType>;
      /**
       * Undefined error.
       */
      Undefined: AugmentedError<ApiType>;
      /**
       * Withdraw fee failed
       */
      WithdrawFailed: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    identity: {
      /**
       * Account ID is already named.
       */
      AlreadyClaimed: AugmentedError<ApiType>;
      /**
       * Empty index.
       */
      EmptyIndex: AugmentedError<ApiType>;
      /**
       * Fee is changed.
       */
      FeeChanged: AugmentedError<ApiType>;
      /**
       * The index is invalid.
       */
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * Invalid judgement.
       */
      InvalidJudgement: AugmentedError<ApiType>;
      /**
       * The target is invalid.
       */
      InvalidTarget: AugmentedError<ApiType>;
      /**
       * Judgement given.
       */
      JudgementGiven: AugmentedError<ApiType>;
      /**
       * No identity found.
       */
      NoIdentity: AugmentedError<ApiType>;
      /**
       * Account isn't found.
       */
      NotFound: AugmentedError<ApiType>;
      /**
       * Account isn't named.
       */
      NotNamed: AugmentedError<ApiType>;
      /**
       * Sub-account isn't owned by sender.
       */
      NotOwned: AugmentedError<ApiType>;
      /**
       * Sender is not a sub-account.
       */
      NotSub: AugmentedError<ApiType>;
      /**
       * Sticky judgement.
       */
      StickyJudgement: AugmentedError<ApiType>;
      /**
       * Too many additional fields.
       */
      TooManyFields: AugmentedError<ApiType>;
      /**
       * Maximum amount of registrars reached. Cannot add any more.
       */
      TooManyRegistrars: AugmentedError<ApiType>;
      /**
       * Too many subs-accounts.
       */
      TooManySubAccounts: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    localAssets: {
      /**
       * The asset-account already exists.
       */
      AlreadyExists: AugmentedError<ApiType>;
      /**
       * Invalid metadata given.
       */
      BadMetadata: AugmentedError<ApiType>;
      /**
       * Invalid witness data given.
       */
      BadWitness: AugmentedError<ApiType>;
      /**
       * Account balance must be greater than or equal to the transfer amount.
       */
      BalanceLow: AugmentedError<ApiType>;
      /**
       * The origin account is frozen.
       */
      Frozen: AugmentedError<ApiType>;
      /**
       * The asset ID is already taken.
       */
      InUse: AugmentedError<ApiType>;
      /**
       * Minimum balance should be non-zero.
       */
      MinBalanceZero: AugmentedError<ApiType>;
      /**
       * The account to alter does not exist.
       */
      NoAccount: AugmentedError<ApiType>;
      /**
       * The asset-account doesn't have an associated deposit.
       */
      NoDeposit: AugmentedError<ApiType>;
      /**
       * The signing account has no permission to do the operation.
       */
      NoPermission: AugmentedError<ApiType>;
      /**
       * Unable to increment the consumer reference counters on the account.
       * Either no provider reference exists to allow a non-zero balance of a
       * non-self-sufficient asset, or the maximum number of consumers has been reached.
       */
      NoProvider: AugmentedError<ApiType>;
      /**
       * No approval exists that would allow the transfer.
       */
      Unapproved: AugmentedError<ApiType>;
      /**
       * The given asset ID is unknown.
       */
      Unknown: AugmentedError<ApiType>;
      /**
       * The operation would result in funds being burned.
       */
      WouldBurn: AugmentedError<ApiType>;
      /**
       * The source account would not survive the transfer and it needs to stay alive.
       */
      WouldDie: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    maintenanceMode: {
      /**
       * The chain cannot enter maintenance mode because it is already in maintenance mode
       */
      AlreadyInMaintenanceMode: AugmentedError<ApiType>;
      /**
       * The chain cannot resume normal operation because it is not in maintenance mode
       */
      NotInMaintenanceMode: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    moonbeamOrbiters: {
      /**
       * The collator is already added in orbiters program.
       */
      CollatorAlreadyAdded: AugmentedError<ApiType>;
      /**
       * This collator is not in orbiters program.
       */
      CollatorNotFound: AugmentedError<ApiType>;
      /**
       * There are already too many orbiters associated with this collator.
       */
      CollatorPoolTooLarge: AugmentedError<ApiType>;
      /**
       * There are more collator pools than the number specified in the parameter.
       */
      CollatorsPoolCountTooLow: AugmentedError<ApiType>;
      /**
       * The minimum deposit required to register as an orbiter has not yet been
       * included in the onchain storage
       */
      MinOrbiterDepositNotSet: AugmentedError<ApiType>;
      /**
       * This orbiter is already associated with this collator.
       */
      OrbiterAlreadyInPool: AugmentedError<ApiType>;
      /**
       * This orbiter has not made a deposit
       */
      OrbiterDepositNotFound: AugmentedError<ApiType>;
      /**
       * This orbiter is not found
       */
      OrbiterNotFound: AugmentedError<ApiType>;
      /**
       * The orbiter is still at least in one pool
       */
      OrbiterStillInAPool: AugmentedError<ApiType>;
      /**
       * Generic error
       */
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
      CandidateNotLeaving: AugmentedError<ApiType>;
      CannotDelegateIfLeaving: AugmentedError<ApiType>;
      CannotDelegateLessThanOrEqualToLowestBottomWhenFull: AugmentedError<ApiType>;
      CannotGoOnlineIfLeaving: AugmentedError<ApiType>;
      CannotSetBelowMin: AugmentedError<ApiType>;
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
      NoWritingSameValue: AugmentedError<ApiType>;
      PendingCandidateRequestAlreadyExists: AugmentedError<ApiType>;
      PendingCandidateRequestNotDueYet: AugmentedError<ApiType>;
      PendingCandidateRequestsDNE: AugmentedError<ApiType>;
      PendingDelegationRequestAlreadyExists: AugmentedError<ApiType>;
      PendingDelegationRequestDNE: AugmentedError<ApiType>;
      PendingDelegationRequestNotDueYet: AugmentedError<ApiType>;
      PendingDelegationRevoke: AugmentedError<ApiType>;
      RoundLengthMustBeAtLeastTotalSelectedCollators: AugmentedError<ApiType>;
      TooLowCandidateCountToLeaveCandidates: AugmentedError<ApiType>;
      TooLowCandidateCountWeightHintCancelLeaveCandidates: AugmentedError<ApiType>;
      TooLowCandidateCountWeightHintJoinCandidates: AugmentedError<ApiType>;
      TooLowCandidateDelegationCountToDelegate: AugmentedError<ApiType>;
      TooLowCandidateDelegationCountToLeaveCandidates: AugmentedError<ApiType>;
      TooLowDelegationCountToDelegate: AugmentedError<ApiType>;
      TooLowDelegationCountToLeaveDelegators: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    parachainSystem: {
      /**
       * The inherent which supplies the host configuration did not run this block
       */
      HostConfigurationNotAvailable: AugmentedError<ApiType>;
      /**
       * No code upgrade has been authorized.
       */
      NothingAuthorized: AugmentedError<ApiType>;
      /**
       * No validation function upgrade is currently scheduled.
       */
      NotScheduled: AugmentedError<ApiType>;
      /**
       * Attempt to upgrade validation function while existing upgrade pending
       */
      OverlappingUpgrades: AugmentedError<ApiType>;
      /**
       * Polkadot currently prohibits this parachain from upgrading its
       * validation function
       */
      ProhibitedByPolkadot: AugmentedError<ApiType>;
      /**
       * The supplied validation function has compiled into a blob larger than
       * Polkadot is willing to run
       */
      TooBig: AugmentedError<ApiType>;
      /**
       * The given code upgrade has not been authorized.
       */
      Unauthorized: AugmentedError<ApiType>;
      /**
       * The inherent which supplies the validation data did not run this block
       */
      ValidationDataNotAvailable: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    polkadotXcm: {
      /**
       * The location is invalid since it already has a subscription from us.
       */
      AlreadySubscribed: AugmentedError<ApiType>;
      /**
       * The given location could not be used (e.g. because it cannot be
       * expressed in the desired version of XCM).
       */
      BadLocation: AugmentedError<ApiType>;
      /**
       * The version of the `Versioned` value used is not able to be interpreted.
       */
      BadVersion: AugmentedError<ApiType>;
      /**
       * Could not re-anchor the assets to declare the fees for the destination chain.
       */
      CannotReanchor: AugmentedError<ApiType>;
      /**
       * The destination `MultiLocation` provided cannot be inverted.
       */
      DestinationNotInvertible: AugmentedError<ApiType>;
      /**
       * The assets to be sent are empty.
       */
      Empty: AugmentedError<ApiType>;
      /**
       * The message execution fails the filter.
       */
      Filtered: AugmentedError<ApiType>;
      /**
       * Origin is invalid for sending.
       */
      InvalidOrigin: AugmentedError<ApiType>;
      /**
       * The referenced subscription could not be found.
       */
      NoSubscription: AugmentedError<ApiType>;
      /**
       * There was some other issue (i.e. not to do with routing) in sending the
       * message. Perhaps a lack of space for buffering the message.
       */
      SendFailure: AugmentedError<ApiType>;
      /**
       * Too many assets have been attempted for transfer.
       */
      TooManyAssets: AugmentedError<ApiType>;
      /**
       * The desired destination was unreachable, generally because there is a
       * no way of routing to it.
       */
      Unreachable: AugmentedError<ApiType>;
      /**
       * The message's weight could not be determined.
       */
      UnweighableMessage: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    proxy: {
      /**
       * Account is already a proxy.
       */
      Duplicate: AugmentedError<ApiType>;
      /**
       * Call may not be made by proxy because it may escalate its privileges.
       */
      NoPermission: AugmentedError<ApiType>;
      /**
       * Cannot add self as proxy.
       */
      NoSelfProxy: AugmentedError<ApiType>;
      /**
       * Proxy registration not found.
       */
      NotFound: AugmentedError<ApiType>;
      /**
       * Sender is not a proxy of the account to be proxied.
       */
      NotProxy: AugmentedError<ApiType>;
      /**
       * There are too many proxies registered or too many announcements pending.
       */
      TooMany: AugmentedError<ApiType>;
      /**
       * Announcement, if made at all, was made too recently.
       */
      Unannounced: AugmentedError<ApiType>;
      /**
       * A call which is incompatible with the proxy type's filter was attempted.
       */
      Unproxyable: AugmentedError<ApiType>;
      /**
       * Generic error
       */
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
       */
      [key: string]: AugmentedError<ApiType>;
    };
    scheduler: {
      /**
       * Failed to schedule a call
       */
      FailedToSchedule: AugmentedError<ApiType>;
      /**
       * Cannot find the scheduled call.
       */
      NotFound: AugmentedError<ApiType>;
      /**
       * Reschedule failed because it does not change scheduled time.
       */
      RescheduleNoChange: AugmentedError<ApiType>;
      /**
       * Given target block number is in the past.
       */
      TargetBlockNumberInPast: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    system: {
      /**
       * The origin filter prevent the call to be dispatched.
       */
      CallFiltered: AugmentedError<ApiType>;
      /**
       * Failed to extract the runtime version from the new runtime.
       *
       * Either calling `Core_version` or decoding `RuntimeVersion` failed.
       */
      FailedToExtractRuntimeVersion: AugmentedError<ApiType>;
      /**
       * The name of specification does not match between the current runtime
       * and the new runtime.
       */
      InvalidSpecName: AugmentedError<ApiType>;
      /**
       * Suicide called when the account has non-default composite data.
       */
      NonDefaultComposite: AugmentedError<ApiType>;
      /**
       * There is a non-zero reference count preventing the account from being purged.
       */
      NonZeroRefCount: AugmentedError<ApiType>;
      /**
       * The specification version is not allowed to decrease between the
       * current runtime and the new runtime.
       */
      SpecVersionNeedsToIncrease: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    techCommitteeCollective: {
      /**
       * Members are already initialized!
       */
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Duplicate proposals not allowed
       */
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate vote ignored
       */
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * Account is not a member
       */
      NotMember: AugmentedError<ApiType>;
      /**
       * Proposal must exist
       */
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * The close call was made too early, before the end of the voting.
       */
      TooEarly: AugmentedError<ApiType>;
      /**
       * There can only be a maximum of `MaxProposals` active proposals.
       */
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Mismatched index
       */
      WrongIndex: AugmentedError<ApiType>;
      /**
       * The given length bound for the proposal was too low.
       */
      WrongProposalLength: AugmentedError<ApiType>;
      /**
       * The given weight bound for the proposal was too low.
       */
      WrongProposalWeight: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    treasury: {
      /**
       * The spend origin is valid but the amount it is allowed to spend is
       * lower than the amount to be spent.
       */
      InsufficientPermission: AugmentedError<ApiType>;
      /**
       * Proposer's balance is too low.
       */
      InsufficientProposersBalance: AugmentedError<ApiType>;
      /**
       * No proposal or bounty at that index.
       */
      InvalidIndex: AugmentedError<ApiType>;
      /**
       * Proposal has not been approved.
       */
      ProposalNotApproved: AugmentedError<ApiType>;
      /**
       * Too many approvals in the queue.
       */
      TooManyApprovals: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    treasuryCouncilCollective: {
      /**
       * Members are already initialized!
       */
      AlreadyInitialized: AugmentedError<ApiType>;
      /**
       * Duplicate proposals not allowed
       */
      DuplicateProposal: AugmentedError<ApiType>;
      /**
       * Duplicate vote ignored
       */
      DuplicateVote: AugmentedError<ApiType>;
      /**
       * Account is not a member
       */
      NotMember: AugmentedError<ApiType>;
      /**
       * Proposal must exist
       */
      ProposalMissing: AugmentedError<ApiType>;
      /**
       * The close call was made too early, before the end of the voting.
       */
      TooEarly: AugmentedError<ApiType>;
      /**
       * There can only be a maximum of `MaxProposals` active proposals.
       */
      TooManyProposals: AugmentedError<ApiType>;
      /**
       * Mismatched index
       */
      WrongIndex: AugmentedError<ApiType>;
      /**
       * The given length bound for the proposal was too low.
       */
      WrongProposalLength: AugmentedError<ApiType>;
      /**
       * The given weight bound for the proposal was too low.
       */
      WrongProposalWeight: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    utility: {
      /**
       * Too many calls batched.
       */
      TooManyCalls: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    xcmpQueue: {
      /**
       * Bad overweight index.
       */
      BadOverweightIndex: AugmentedError<ApiType>;
      /**
       * Bad XCM data.
       */
      BadXcm: AugmentedError<ApiType>;
      /**
       * Bad XCM origin.
       */
      BadXcmOrigin: AugmentedError<ApiType>;
      /**
       * Failed to send XCM message.
       */
      FailedToSend: AugmentedError<ApiType>;
      /**
       * Provided weight is possibly not enough to execute the message.
       */
      WeightOverLimit: AugmentedError<ApiType>;
      /**
       * Generic error
       */
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
      ErrorSending: AugmentedError<ApiType>;
      FailedMultiLocationToJunction: AugmentedError<ApiType>;
      FeePerSecondNotSet: AugmentedError<ApiType>;
      IndexAlreadyClaimed: AugmentedError<ApiType>;
      InvalidDest: AugmentedError<ApiType>;
      MaxWeightTransactReached: AugmentedError<ApiType>;
      NotCrossChainTransfer: AugmentedError<ApiType>;
      NotCrossChainTransferableCurrency: AugmentedError<ApiType>;
      NotOwner: AugmentedError<ApiType>;
      SignedTransactNotAllowedForDestination: AugmentedError<ApiType>;
      TransactorInfoNotSet: AugmentedError<ApiType>;
      UnableToWithdrawAsset: AugmentedError<ApiType>;
      UnclaimedIndex: AugmentedError<ApiType>;
      UnweighableMessage: AugmentedError<ApiType>;
      WeightOverflow: AugmentedError<ApiType>;
      XcmExecuteError: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
    xTokens: {
      /**
       * Asset has no reserve location.
       */
      AssetHasNoReserve: AugmentedError<ApiType>;
      /**
       * The specified index does not exist in a MultiAssets struct.
       */
      AssetIndexNonExistent: AugmentedError<ApiType>;
      /**
       * The version of the `Versioned` value used is not able to be interpreted.
       */
      BadVersion: AugmentedError<ApiType>;
      /**
       * Could not re-anchor the assets to declare the fees for the destination chain.
       */
      CannotReanchor: AugmentedError<ApiType>;
      /**
       * The destination `MultiLocation` provided cannot be inverted.
       */
      DestinationNotInvertible: AugmentedError<ApiType>;
      /**
       * We tried sending distinct asset and fee but they have different reserve chains.
       */
      DistinctReserveForAssetAndFee: AugmentedError<ApiType>;
      /**
       * Fee is not enough.
       */
      FeeNotEnough: AugmentedError<ApiType>;
      /**
       * Could not get ancestry of asset reserve location.
       */
      InvalidAncestry: AugmentedError<ApiType>;
      /**
       * The MultiAsset is invalid.
       */
      InvalidAsset: AugmentedError<ApiType>;
      /**
       * Invalid transfer destination.
       */
      InvalidDest: AugmentedError<ApiType>;
      /**
       * MinXcmFee not registered for certain reserve location
       */
      MinXcmFeeNotDefined: AugmentedError<ApiType>;
      /**
       * Not cross-chain transfer.
       */
      NotCrossChainTransfer: AugmentedError<ApiType>;
      /**
       * Currency is not cross-chain transferable.
       */
      NotCrossChainTransferableCurrency: AugmentedError<ApiType>;
      /**
       * Not supported MultiLocation
       */
      NotSupportedMultiLocation: AugmentedError<ApiType>;
      /**
       * The number of assets to be sent is over the maximum.
       */
      TooManyAssetsBeingSent: AugmentedError<ApiType>;
      /**
       * The message's weight could not be determined.
       */
      UnweighableMessage: AugmentedError<ApiType>;
      /**
       * XCM execution failed.
       */
      XcmExecutionFailed: AugmentedError<ApiType>;
      /**
       * The transfering asset amount is zero.
       */
      ZeroAmount: AugmentedError<ApiType>;
      /**
       * The fee is zero.
       */
      ZeroFee: AugmentedError<ApiType>;
      /**
       * Generic error
       */
      [key: string]: AugmentedError<ApiType>;
    };
  } // AugmentedErrors
} // declare module
