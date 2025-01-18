// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/randomness/Randomness.sol";
import {RandomnessConsumer} from "precompiles/randomness/RandomnessConsumer.sol";

/// @notice Smart contract to demonstrate how to use requestLocalVRFRandomWords
contract RandomnessLotteryDemo is RandomnessConsumer {
    /// @notice The lottery has requested random words and is waiting for fulfillment
    error WaitingFulfillment();

    /// @notice The lottery doesn't have enough participants to start
    error NotEnoughParticipants(uint256 value, uint256 required);

    /// @notice The lottery doesn't accept additional participants
    error TooManyParticipants(uint256 value, uint256 required);

    /// @notice There are not enough fee to start the lottery
    error NotEnoughFee(uint256 value, uint256 required);

    /// @notice The deposit given is too low
    error DepositTooLow(uint256 value, uint256 required);

    /// @notice The provided fee to participate doesn't match the required amount
    error InvalidParticipationFee(uint256 value, uint256 required);

    /// @notice Event sent when a winner is awarded
    /// @param winner The participant getting awarded
    /// @param randomWord The randomWord being used (for informative purposes)
    /// @param amount The amount being awarded
    event Awarded(address indexed winner, uint256 randomWord, uint256 amount);

    /// @notice Event sent when the lottery started
    /// @param participantCount The number of participants
    /// @param jackpot The total jackpot
    /// @param requestId The pseudo-random request id
    event Started(uint256 participantCount, uint256 jackpot, uint256 requestId);

    /// @notice Event sent when the lottery ends
    /// @param participantCount The number of participants
    /// @param jackpot The total jackpot
    /// @param winnerCount The number of winners
    event Ended(uint256 participantCount, uint256 jackpot, uint256 winnerCount);

    /// @notice The status of lottery
    /// @param OpenForRegistration Participants can register to get a chance to win
    /// @param RollingNumbers The lottery has requested the random words and is waiting for them
    /// @param Expired The lottery has been rolling numbers for too long. The randomness has expired
    enum LotteryStatus {
        OpenForRegistration,
        RollingNumbers,
        Expired
    }

    /// @notice The gas limit allowed to be used for the fulfillment
    /// @dev Depends on the code that is executed and the number of words requested
    /// @dev so XXX is a safe default for this example contract. Test and adjust
    /// @dev this limit based on the size of the request and the processing of the
    /// @dev callback request in the fulfillRandomWords() function.
    /// @dev The fee paid to start the lottery needs to be sufficient to pay for the gas limit
    uint64 public FULFILLMENT_GAS_LIMIT = 200000; // Safe gas limit for processing NUM_WINNERS random words and transferring rewards

    /// @notice The minimum fee needed to start the lottery
    /// @dev This does not guarantee that there will be enough fee to pay for the
    /// @dev gas used by the fulfillment. Ideally it should be over-estimated
    /// @dev considering possible fluctuation of the gas price.
    /// @dev Additional fee will be refunded to the caller
    uint256 public MIN_FEE = FULFILLMENT_GAS_LIMIT * 1 gwei;

    /// @notice The number of winners
    /// @dev This number corresponds to how many random words will requested
    /// @dev Cannot exceed MAX_RANDOM_WORDS
    uint8 public NUM_WINNERS = 2;

    /// @notice The number of block before the request can be fulfilled (for Local VRF randomness)
    /// @dev The MIN_VRF_BLOCKS_DELAY provides a minimum number that is safe enough for
    /// @dev games with low economical value at stake.
    /// @dev Increasing the delay reduces slightly the probability (already very low)
    /// @dev of a collator being able to predict the pseudo-random number
    uint32 public VRF_BLOCKS_DELAY = MIN_VRF_BLOCKS_DELAY;

    /// @notice The minimum number of participants to start the lottery
    uint256 public MIN_PARTICIPANTS = 3;

    /// @notice The maximum number of participants allowed to participate
    /// @dev It is important to limit the total jackpot (by limiting the number of
    /// @dev participants) to guarantee the economic incentive of a collator
    /// @dev to avoid trying to influence the pseudo-random
    /// @dev (See Randomness.sol for more details)
    uint256 public MAX_PARTICIPANTS = 20;

    /// @notice The fee needed to participate in the lottery. Will go into the jackpot
    uint256 public PARTICIPATION_FEE = 1 ether;

    /// @notice A string used to allow having different salt that other contracts
    bytes32 public SALT_PREFIX = "my_demo_salt_change_me";

    /// @notice global number of request done
    /// @dev This number is used as a salt to make it unique for each request
    uint256 public globalRequestCount;

    /// @notice The current request id
    uint256 public requestId;

    /// @notice The list of current participants
    address[] public participants;

    /// @notice The current amount of token at stake in the lottery
    uint256 public jackpot;

    /// @notice the owner of the contract
    address owner;

    /// @notice Which randomness source to use
    Randomness.RandomnessSource randomnessSource;

    constructor(
        Randomness.RandomnessSource source
    ) payable RandomnessConsumer() {
        /// Because this contract can only perform 1 random request at a time,
        /// We only need to have 1 required deposit.
        uint256 requiredDeposit = RANDOMNESS_CONTRACT.requiredDeposit();
        if (msg.value < requiredDeposit) {
            revert DepositTooLow(msg.value, requiredDeposit);
        }

        randomnessSource = source;
        owner = msg.sender;
        globalRequestCount = 0;
        jackpot = 0;
        /// Set the requestId to the maximum allowed value by the precompile (64 bits)
        requestId = 2 ** 64 - 1;
    }

    function status() external view returns (LotteryStatus) {
        Randomness.RequestStatus requestStatus = RANDOMNESS_CONTRACT
            .getRequestStatus(requestId);
        if (requestStatus == Randomness.RequestStatus.DoesNotExist) {
            return LotteryStatus.OpenForRegistration;
        }
        if (
            requestStatus == Randomness.RequestStatus.Pending ||
            requestStatus == Randomness.RequestStatus.Ready
        ) {
            return LotteryStatus.RollingNumbers;
        }
        return LotteryStatus.Expired;
    }

    function participate() external payable {
        /// We check we haven't started the randomness request yet
        if (
            RANDOMNESS_CONTRACT.getRequestStatus(requestId) !=
            Randomness.RequestStatus.DoesNotExist
        ) {
            revert WaitingFulfillment();
        }

        //each player is compelled to add a certain ETH to join
        if (msg.value != PARTICIPATION_FEE) {
            revert InvalidParticipationFee(msg.value, PARTICIPATION_FEE);
        }
        participants.push(msg.sender);
        jackpot += msg.value;
    }

    function startLottery() external payable onlyOwner {
        /// We check we haven't started the randomness request yet
        if (
            RANDOMNESS_CONTRACT.getRequestStatus(requestId) !=
            Randomness.RequestStatus.DoesNotExist
        ) {
            revert WaitingFulfillment();
        }

        if (participants.length < MIN_PARTICIPANTS) {
            revert NotEnoughParticipants(participants.length, MIN_PARTICIPANTS);
        }
        if (participants.length >= MAX_PARTICIPANTS) {
            revert TooManyParticipants(participants.length, MAX_PARTICIPANTS);
        }

        uint256 fee = msg.value;
        if (fee < MIN_FEE) {
            revert NotEnoughFee(fee, MIN_FEE);
        }

        /// We verify there is enough balance on the contract to pay for the deposit.
        /// This would fail only if the deposit amount required is changed in the
        /// Randomness Precompile.
        uint256 requiredDeposit = RANDOMNESS_CONTRACT.requiredDeposit();
        if (address(this).balance < jackpot + requiredDeposit) {
            revert DepositTooLow(
                address(this).balance - jackpot,
                requiredDeposit
            );
        }

        if (randomnessSource == Randomness.RandomnessSource.LocalVRF) {
            /// Requesting NUM_WINNERS random words using Local VRF Randomness
            /// with a delay of VRF_BLOCKS_DELAY blocks
            /// Refund after fulfillment will go back to the caller of this function
            /// globalRequestCount is used as salt to be unique for each request
            requestId = RANDOMNESS_CONTRACT.requestLocalVRFRandomWords(
                msg.sender,
                fee,
                FULFILLMENT_GAS_LIMIT,
                SALT_PREFIX ^ bytes32(globalRequestCount++),
                NUM_WINNERS,
                VRF_BLOCKS_DELAY
            );
        } else {
            /// Requesting NUM_WINNERS random words using Babe Epoch Randomness
            /// Babe Epoch Randomness requires a longer delay (depending on the relaychain)
            /// Refund after fulfillment will go back to the caller of this function
            /// globalRequestCount is used as salt to be unique for each request
            requestId = RANDOMNESS_CONTRACT.requestRelayBabeEpochRandomWords(
                msg.sender,
                fee,
                FULFILLMENT_GAS_LIMIT,
                SALT_PREFIX ^ bytes32(globalRequestCount++),
                NUM_WINNERS
            );
        }
    }

    /// @notice Allows to increase the fee associated with the request
    /// @dev This is needed if the gas price increase significantly before
    /// @dev the request is fulfilled
    function increaseRequestFee() external payable {
        RANDOMNESS_CONTRACT.increaseRequestFee(requestId, msg.value);
    }

    /// @dev This function is called only by the fulfillment callback
    function pickWinners(uint256[] memory randomWords) internal {
        /// Get the total number of winners to select
        uint256 totalWinners = NUM_WINNERS < participants.length
            ? NUM_WINNERS
            : participants.length;

        /// The amount distributed to each winner
        /// The left-over is kept for the next lottery
        uint256 amountAwarded = jackpot / totalWinners;
        emit Ended(participants.length, jackpot, totalWinners);
        for (uint32 i = 0; i < totalWinners; i++) {
            /// This is safe to index randomWords with i because we requested
            /// NUM_WINNERS random words
            uint256 randomWord = randomWords[i];

            /// Using modulo is not totally fair, but fair enough for this demo.
            uint256 index = randomWord % participants.length;
            address payable winner = payable(participants[index]);
            delete participants[index];
            emit Awarded(winner, randomWord, amountAwarded);
            jackpot -= amountAwarded;
            winner.transfer(amountAwarded);
        }
        delete participants;
        requestId = 0;
    }

    function fulfillRandomWords(
        uint256 /* requestId */,
        uint256[] memory randomWords
    ) internal override {
        pickWinners(randomWords);
    }

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
}
