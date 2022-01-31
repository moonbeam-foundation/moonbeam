// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import type { ApiTypes } from '@polkadot/api-base/types';
import type { Bytes, Null, Option, Result, U256, U8aFixed, Vec, bool, u128, u16, u32, u64 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { AccountId20, H160, H256, Perbill, Percent } from '@polkadot/types/interfaces/runtime';
import type { EthereumLog, EvmCoreErrorExitReason, FrameSupportTokensMiscBalanceStatus, FrameSupportWeightsDispatchInfo, MoonbeamRuntimeProxyType, NimbusPrimitivesNimbusCryptoPublic, PalletDemocracyVoteAccountVote, PalletDemocracyVoteThreshold, ParachainStakingDelegationRequest, ParachainStakingDelegatorAdded, SpRuntimeDispatchError } from '@polkadot/types/lookup';

declare module '@polkadot/api-base/types/events' {
  export interface AugmentedEvents<ApiType extends ApiTypes> {
    authorFilter: {
      /**
       * The amount of eligible authors for the filter to select has been changed.
       **/
      EligibleUpdated: AugmentedEvent<ApiType, [Percent]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    authorMapping: {
      /**
       * An NimbusId has been de-registered, and its AccountId mapping removed.
       **/
      AuthorDeRegistered: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic]>;
      /**
       * A NimbusId has been registered and mapped to an AccountId.
       **/
      AuthorRegistered: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
      /**
       * An NimbusId has been registered, replacing a previous registration and its mapping.
       **/
      AuthorRotated: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
      /**
       * An NimbusId has been forcibly deregistered after not being rotated or cleaned up.
       * The reporteing account has been rewarded accordingly.
       **/
      DefunctAuthorBusted: AugmentedEvent<ApiType, [NimbusPrimitivesNimbusCryptoPublic, AccountId20]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    balances: {
      /**
       * A balance was set by root. \[who, free, reserved\]
       **/
      BalanceSet: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Some amount was deposited into the account (e.g. for transaction fees). \[who,
       * deposit\]
       **/
      Deposit: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * An account was removed whose balance was non-zero but below ExistentialDeposit,
       * resulting in an outright loss. \[account, balance\]
       **/
      DustLost: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * An account was created with some free balance. \[account, free_balance\]
       **/
      Endowed: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Some balance was reserved (moved from free to reserved). \[who, value\]
       **/
      Reserved: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Some balance was moved from the reserve of the first account to the second account.
       * Final argument indicates the destination balance type.
       * \[from, to, balance, destination_status\]
       **/
      ReserveRepatriated: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, FrameSupportTokensMiscBalanceStatus]>;
      /**
       * Some amount was removed from the account (e.g. for misbehavior). \[who,
       * amount_slashed\]
       **/
      Slashed: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Transfer succeeded. \[from, to, value\]
       **/
      Transfer: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Some balance was unreserved (moved from reserved to free). \[who, value\]
       **/
      Unreserved: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Some amount was withdrawn from the account (e.g. for transaction fees). \[who, value\]
       **/
      Withdraw: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    councilCollective: {
      /**
       * A motion was approved by the required threshold.
       * \[proposal_hash\]
       **/
      Approved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A proposal was closed because its threshold was reached or after its duration was up.
       * \[proposal_hash, yes, no\]
       **/
      Closed: AugmentedEvent<ApiType, [H256, u32, u32]>;
      /**
       * A motion was not approved by the required threshold.
       * \[proposal_hash\]
       **/
      Disapproved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A motion was executed; result will be `Ok` if it returned without error.
       * \[proposal_hash, result\]
       **/
      Executed: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A single member did some action; result will be `Ok` if it returned without error.
       * \[proposal_hash, result\]
       **/
      MemberExecuted: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a threshold (given
       * `MemberCount`).
       * \[account, proposal_index, proposal_hash, threshold\]
       **/
      Proposed: AugmentedEvent<ApiType, [AccountId20, u32, H256, u32]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving
       * a tally (yes votes and no votes given respectively as `MemberCount`).
       * \[account, proposal_hash, voted, yes, no\]
       **/
      Voted: AugmentedEvent<ApiType, [AccountId20, H256, bool, u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    crowdloanRewards: {
      /**
       * When initializing the reward vec an already initialized account was found
       **/
      InitializedAccountWithNotEnoughContribution: AugmentedEvent<ApiType, [U8aFixed, Option<AccountId20>, u128]>;
      /**
       * When initializing the reward vec an already initialized account was found
       **/
      InitializedAlreadyInitializedAccount: AugmentedEvent<ApiType, [U8aFixed, Option<AccountId20>, u128]>;
      /**
       * The initial payment of InitializationPayment % was paid
       **/
      InitialPaymentMade: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Someone has proven they made a contribution and associated a native identity with it.
       * Data is the relay account,  native account and the total amount of _rewards_ that will be paid
       **/
      NativeIdentityAssociated: AugmentedEvent<ApiType, [U8aFixed, AccountId20, u128]>;
      /**
       * A contributor has updated the reward address.
       **/
      RewardAddressUpdated: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * A contributor has claimed some rewards.
       * Data is the account getting paid and the amount of rewards paid.
       **/
      RewardsPaid: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    democracy: {
      /**
       * A proposal \[hash\] has been blacklisted permanently.
       **/
      Blacklisted: AugmentedEvent<ApiType, [H256]>;
      /**
       * A referendum has been cancelled. \[ref_index\]
       **/
      Cancelled: AugmentedEvent<ApiType, [u32]>;
      /**
       * An account has delegated their vote to another account. \[who, target\]
       **/
      Delegated: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * A proposal has been enacted. \[ref_index, result\]
       **/
      Executed: AugmentedEvent<ApiType, [u32, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * An external proposal has been tabled.
       **/
      ExternalTabled: AugmentedEvent<ApiType, []>;
      /**
       * A proposal has been rejected by referendum. \[ref_index\]
       **/
      NotPassed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A proposal has been approved by referendum. \[ref_index\]
       **/
      Passed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A proposal could not be executed because its preimage was invalid.
       * \[proposal_hash, ref_index\]
       **/
      PreimageInvalid: AugmentedEvent<ApiType, [H256, u32]>;
      /**
       * A proposal could not be executed because its preimage was missing.
       * \[proposal_hash, ref_index\]
       **/
      PreimageMissing: AugmentedEvent<ApiType, [H256, u32]>;
      /**
       * A proposal's preimage was noted, and the deposit taken. \[proposal_hash, who, deposit\]
       **/
      PreimageNoted: AugmentedEvent<ApiType, [H256, AccountId20, u128]>;
      /**
       * A registered preimage was removed and the deposit collected by the reaper.
       * \[proposal_hash, provider, deposit, reaper\]
       **/
      PreimageReaped: AugmentedEvent<ApiType, [H256, AccountId20, u128, AccountId20]>;
      /**
       * A proposal preimage was removed and used (the deposit was returned).
       * \[proposal_hash, provider, deposit\]
       **/
      PreimageUsed: AugmentedEvent<ApiType, [H256, AccountId20, u128]>;
      /**
       * A motion has been proposed by a public account. \[proposal_index, deposit\]
       **/
      Proposed: AugmentedEvent<ApiType, [u32, u128]>;
      /**
       * An account has secconded a proposal
       **/
      Seconded: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A referendum has begun. \[ref_index, threshold\]
       **/
      Started: AugmentedEvent<ApiType, [u32, PalletDemocracyVoteThreshold]>;
      /**
       * A public proposal has been tabled for referendum vote. \[proposal_index, deposit,
       * depositors\]
       **/
      Tabled: AugmentedEvent<ApiType, [u32, u128, Vec<AccountId20>]>;
      /**
       * An \[account\] has cancelled a previous delegation operation.
       **/
      Undelegated: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * An external proposal has been vetoed. \[who, proposal_hash, until\]
       **/
      Vetoed: AugmentedEvent<ApiType, [AccountId20, H256, u32]>;
      /**
       * An account has voted in a referendum
       **/
      Voted: AugmentedEvent<ApiType, [AccountId20, u32, PalletDemocracyVoteAccountVote]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    ethereum: {
      /**
       * An ethereum transaction was successfully executed. [from, to/contract_address, transaction_hash, exit_reason]
       **/
      Executed: AugmentedEvent<ApiType, [H160, H160, H256, EvmCoreErrorExitReason]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    evm: {
      /**
       * A deposit has been made at a given address. \[sender, address, value\]
       **/
      BalanceDeposit: AugmentedEvent<ApiType, [AccountId20, H160, U256]>;
      /**
       * A withdrawal has been made from a given address. \[sender, address, value\]
       **/
      BalanceWithdraw: AugmentedEvent<ApiType, [AccountId20, H160, U256]>;
      /**
       * A contract has been created at given \[address\].
       **/
      Created: AugmentedEvent<ApiType, [H160]>;
      /**
       * A \[contract\] was attempted to be created, but the execution failed.
       **/
      CreatedFailed: AugmentedEvent<ApiType, [H160]>;
      /**
       * A \[contract\] has been executed successfully with states applied.
       **/
      Executed: AugmentedEvent<ApiType, [H160]>;
      /**
       * A \[contract\] has been executed with errors. States are reverted with only gas fees applied.
       **/
      ExecutedFailed: AugmentedEvent<ApiType, [H160]>;
      /**
       * Ethereum events from contracts.
       **/
      Log: AugmentedEvent<ApiType, [EthereumLog]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    identity: {
      /**
       * A name was cleared, and the given balance returned. \[who, deposit\]
       **/
      IdentityCleared: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * A name was removed and the given balance slashed. \[who, deposit\]
       **/
      IdentityKilled: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * A name was set or reset (which will remove all judgements). \[who\]
       **/
      IdentitySet: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * A judgement was given by a registrar. \[target, registrar_index\]
       **/
      JudgementGiven: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A judgement was asked from a registrar. \[who, registrar_index\]
       **/
      JudgementRequested: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A judgement request was retracted. \[who, registrar_index\]
       **/
      JudgementUnrequested: AugmentedEvent<ApiType, [AccountId20, u32]>;
      /**
       * A registrar was added. \[registrar_index\]
       **/
      RegistrarAdded: AugmentedEvent<ApiType, [u32]>;
      /**
       * A sub-identity was added to an identity and the deposit paid. \[sub, main, deposit\]
       **/
      SubIdentityAdded: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * A sub-identity was removed from an identity and the deposit freed.
       * \[sub, main, deposit\]
       **/
      SubIdentityRemoved: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * A sub-identity was cleared, and the given deposit repatriated from the
       * main identity account to the sub-identity account. \[sub, main, deposit\]
       **/
      SubIdentityRevoked: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    maintenanceMode: {
      /**
       * The chain was put into Maintenance Mode
       **/
      EnteredMaintenanceMode: AugmentedEvent<ApiType, []>;
      /**
       * The chain returned to its normal operating state
       **/
      NormalOperationResumed: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    migrations: {
      MigrationCompleted: AugmentedEvent<ApiType, [Bytes, u64]>;
      MigrationStarted: AugmentedEvent<ApiType, [Bytes]>;
      RuntimeUpgradeCompleted: AugmentedEvent<ApiType, [u64]>;
      RuntimeUpgradeStarted: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    parachainStaking: {
      /**
       * Set blocks per round [current_round, first_block, old, new, new_per_round_inflation]
       **/
      BlocksPerRoundSet: AugmentedEvent<ApiType, [u32, u32, u32, u32, Perbill, Perbill, Perbill]>;
      /**
       * Candidate, Amount, Round at which could be executed
       **/
      CancelledCandidateBondLess: AugmentedEvent<ApiType, [AccountId20, u128, u32]>;
      /**
       * Candidate
       **/
      CancelledCandidateExit: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * Delegator, Cancelled Request
       **/
      CancelledDelegationRequest: AugmentedEvent<ApiType, [AccountId20, ParachainStakingDelegationRequest]>;
      /**
       * Round Online, Candidate
       **/
      CandidateBackOnline: AugmentedEvent<ApiType, [u32, AccountId20]>;
      /**
       * Candidate, Amount, New Bond
       **/
      CandidateBondedLess: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Candidate, Amount, New Bond Total
       **/
      CandidateBondedMore: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Candidate, Amount To Decrease, Round at which request can be executed by caller
       **/
      CandidateBondLessRequested: AugmentedEvent<ApiType, [AccountId20, u128, u32]>;
      /**
       * Ex-Candidate, Amount Unlocked, New Total Amt Locked
       **/
      CandidateLeft: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Round At Which Exit Is Allowed, Candidate, Scheduled Exit
       **/
      CandidateScheduledExit: AugmentedEvent<ApiType, [u32, AccountId20, u32]>;
      /**
       * Round Offline, Candidate
       **/
      CandidateWentOffline: AugmentedEvent<ApiType, [u32, AccountId20]>;
      /**
       * Round, Collator Account, Total Exposed Amount (includes all delegations)
       **/
      CollatorChosen: AugmentedEvent<ApiType, [u32, AccountId20, u128]>;
      /**
       * Set collator commission to this value [old, new]
       **/
      CollatorCommissionSet: AugmentedEvent<ApiType, [Perbill, Perbill]>;
      /**
       * Delegator, Amount Locked, Candidate, Delegator Position with New Total Counted if in Top
       **/
      Delegation: AugmentedEvent<ApiType, [AccountId20, u128, AccountId20, ParachainStakingDelegatorAdded]>;
      DelegationDecreased: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, bool]>;
      /**
       * Delegator, Candidate, Amount to be decreased, Round at which can be executed
       **/
      DelegationDecreaseScheduled: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, u32]>;
      DelegationIncreased: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, bool]>;
      /**
       * Round, Delegator, Candidate, Scheduled Exit
       **/
      DelegationRevocationScheduled: AugmentedEvent<ApiType, [u32, AccountId20, AccountId20, u32]>;
      /**
       * Delegator, Candidate, Amount Unstaked
       **/
      DelegationRevoked: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128]>;
      /**
       * Delegator
       **/
      DelegatorExitCancelled: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * Round, Delegator, Scheduled Exit
       **/
      DelegatorExitScheduled: AugmentedEvent<ApiType, [u32, AccountId20, u32]>;
      /**
       * Delegator, Amount Unstaked
       **/
      DelegatorLeft: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Delegator, Candidate, Amount Unstaked, New Total Amt Staked for Candidate
       **/
      DelegatorLeftCandidate: AugmentedEvent<ApiType, [AccountId20, AccountId20, u128, u128]>;
      /**
       * Annual inflation input (first 3) was used to derive new per-round inflation (last 3)
       **/
      InflationSet: AugmentedEvent<ApiType, [Perbill, Perbill, Perbill, Perbill, Perbill, Perbill]>;
      /**
       * Account, Amount Locked, New Total Amt Locked
       **/
      JoinedCollatorCandidates: AugmentedEvent<ApiType, [AccountId20, u128, u128]>;
      /**
       * Starting Block, Round, Number of Collators Selected, Total Balance
       **/
      NewRound: AugmentedEvent<ApiType, [u32, u32, u32, u128]>;
      /**
       * Account (re)set for parachain bond treasury [old, new]
       **/
      ParachainBondAccountSet: AugmentedEvent<ApiType, [AccountId20, AccountId20]>;
      /**
       * Percent of inflation reserved for parachain bond (re)set [old, new]
       **/
      ParachainBondReservePercentSet: AugmentedEvent<ApiType, [Percent, Percent]>;
      /**
       * Transferred to account which holds funds reserved for parachain bond
       **/
      ReservedForParachainBond: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Paid the account (delegator or collator) the balance as liquid rewards
       **/
      Rewarded: AugmentedEvent<ApiType, [AccountId20, u128]>;
      /**
       * Staking expectations set
       **/
      StakeExpectationsSet: AugmentedEvent<ApiType, [u128, u128, u128]>;
      /**
       * Set total selected candidates to this value [old, new]
       **/
      TotalSelectedSet: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    parachainSystem: {
      /**
       * Downward messages were processed using the given weight.
       * \[ weight_used, result_mqc_head \]
       **/
      DownwardMessagesProcessed: AugmentedEvent<ApiType, [u64, H256]>;
      /**
       * Some downward messages have been received and will be processed.
       * \[ count \]
       **/
      DownwardMessagesReceived: AugmentedEvent<ApiType, [u32]>;
      /**
       * An upgrade has been authorized.
       **/
      UpgradeAuthorized: AugmentedEvent<ApiType, [H256]>;
      /**
       * The validation function was applied as of the contained relay chain block number.
       **/
      ValidationFunctionApplied: AugmentedEvent<ApiType, [u32]>;
      /**
       * The relay-chain aborted the upgrade process.
       **/
      ValidationFunctionDiscarded: AugmentedEvent<ApiType, []>;
      /**
       * The validation function has been scheduled to apply.
       **/
      ValidationFunctionStored: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    proxy: {
      /**
       * An announcement was placed to make a call in the future. \[real, proxy, call_hash\]
       **/
      Announced: AugmentedEvent<ApiType, [AccountId20, AccountId20, H256]>;
      /**
       * Anonymous account has been created by new proxy with given
       * disambiguation index and proxy type. \[anonymous, who, proxy_type,
       * disambiguation_index\]
       **/
      AnonymousCreated: AugmentedEvent<ApiType, [AccountId20, AccountId20, MoonbeamRuntimeProxyType, u16]>;
      /**
       * A proxy was added. \[delegator, delegatee, proxy_type, delay\]
       **/
      ProxyAdded: AugmentedEvent<ApiType, [AccountId20, AccountId20, MoonbeamRuntimeProxyType, u32]>;
      /**
       * A proxy was executed correctly, with the given \[result\].
       **/
      ProxyExecuted: AugmentedEvent<ApiType, [Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    scheduler: {
      /**
       * Canceled some task. \[when, index\]
       **/
      Canceled: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Dispatched some task. \[task, id, result\]
       **/
      Dispatched: AugmentedEvent<ApiType, [ITuple<[u32, u32]>, Option<Bytes>, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * Scheduled some task. \[when, index\]
       **/
      Scheduled: AugmentedEvent<ApiType, [u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    system: {
      /**
       * `:code` was updated.
       **/
      CodeUpdated: AugmentedEvent<ApiType, []>;
      /**
       * An extrinsic failed. \[error, info\]
       **/
      ExtrinsicFailed: AugmentedEvent<ApiType, [SpRuntimeDispatchError, FrameSupportWeightsDispatchInfo]>;
      /**
       * An extrinsic completed successfully. \[info\]
       **/
      ExtrinsicSuccess: AugmentedEvent<ApiType, [FrameSupportWeightsDispatchInfo]>;
      /**
       * An \[account\] was reaped.
       **/
      KilledAccount: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * A new \[account\] was created.
       **/
      NewAccount: AugmentedEvent<ApiType, [AccountId20]>;
      /**
       * On on-chain remark happened. \[origin, remark_hash\]
       **/
      Remarked: AugmentedEvent<ApiType, [AccountId20, H256]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    techCommitteeCollective: {
      /**
       * A motion was approved by the required threshold.
       * \[proposal_hash\]
       **/
      Approved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A proposal was closed because its threshold was reached or after its duration was up.
       * \[proposal_hash, yes, no\]
       **/
      Closed: AugmentedEvent<ApiType, [H256, u32, u32]>;
      /**
       * A motion was not approved by the required threshold.
       * \[proposal_hash\]
       **/
      Disapproved: AugmentedEvent<ApiType, [H256]>;
      /**
       * A motion was executed; result will be `Ok` if it returned without error.
       * \[proposal_hash, result\]
       **/
      Executed: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A single member did some action; result will be `Ok` if it returned without error.
       * \[proposal_hash, result\]
       **/
      MemberExecuted: AugmentedEvent<ApiType, [H256, Result<Null, SpRuntimeDispatchError>]>;
      /**
       * A motion (given hash) has been proposed (by given account) with a threshold (given
       * `MemberCount`).
       * \[account, proposal_index, proposal_hash, threshold\]
       **/
      Proposed: AugmentedEvent<ApiType, [AccountId20, u32, H256, u32]>;
      /**
       * A motion (given hash) has been voted on by given account, leaving
       * a tally (yes votes and no votes given respectively as `MemberCount`).
       * \[account, proposal_hash, voted, yes, no\]
       **/
      Voted: AugmentedEvent<ApiType, [AccountId20, H256, bool, u32, u32]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    treasury: {
      /**
       * Some funds have been allocated. \[proposal_index, award, beneficiary\]
       **/
      Awarded: AugmentedEvent<ApiType, [u32, u128, AccountId20]>;
      /**
       * Some of our funds have been burnt. \[burn\]
       **/
      Burnt: AugmentedEvent<ApiType, [u128]>;
      /**
       * Some funds have been deposited. \[deposit\]
       **/
      Deposit: AugmentedEvent<ApiType, [u128]>;
      /**
       * New proposal. \[proposal_index\]
       **/
      Proposed: AugmentedEvent<ApiType, [u32]>;
      /**
       * A proposal was rejected; funds were slashed. \[proposal_index, slashed\]
       **/
      Rejected: AugmentedEvent<ApiType, [u32, u128]>;
      /**
       * Spending has finished; this is the amount that rolls over until next spend.
       * \[budget_remaining\]
       **/
      Rollover: AugmentedEvent<ApiType, [u128]>;
      /**
       * We have ended a spend period and will now allocate funds. \[budget_remaining\]
       **/
      Spending: AugmentedEvent<ApiType, [u128]>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
    utility: {
      /**
       * Batch of dispatches completed fully with no error.
       **/
      BatchCompleted: AugmentedEvent<ApiType, []>;
      /**
       * Batch of dispatches did not complete fully. Index of first failing dispatch given, as
       * well as the error. \[index, error\]
       **/
      BatchInterrupted: AugmentedEvent<ApiType, [u32, SpRuntimeDispatchError]>;
      /**
       * A single item within a Batch of dispatches has completed with no error.
       **/
      ItemCompleted: AugmentedEvent<ApiType, []>;
      /**
       * Generic event
       **/
      [key: string]: AugmentedEvent<ApiType>;
    };
  } // AugmentedEvents
} // declare module
