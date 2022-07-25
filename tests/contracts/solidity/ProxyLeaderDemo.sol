// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

import "../../../precompiles/proxy/Proxy.sol";

/// @notice Smart contract to demonstrate how to use requestLocalVRFRandomWords
contract ProxyLeaderDemo {
    /// @notice The Proxy Precompile Interface
    Proxy public proxy = Proxy(0x000000000000000000000000000000000000080b);

    /// @notice Event sent when a voting session starts
    event VotingStarted(uint256 votingRound);

    /// @notice Event sent when a voting session ends
    event VotingEnded(
        uint256 votingRound,
        address winnerStaker,
        address winnerGovernor
    );

    /// @notice Event sent when a proxy is added
    /// @param delegate The account that is now a proxy account
    /// @param proxyType The proxy type that was added
    event ProxyAdded(address indexed delegate, Proxy.ProxyType proxyType);

    /// @notice Event sent when a proxy is removed
    /// @param delegate The account that is now no longer a proxy account
    /// @param proxyType The proxy type that was removed
    event ProxyRemoved(address indexed delegate, Proxy.ProxyType proxyType);

    /// @notice The voting has already been started
    error VotingAlreadyInProgress();

    /// @notice The voting has not been started
    error VotingNotInProgress();

    /// @notice Event sent when a voting session ends
    error AlreadyVoted(uint256 votingRound);

    /// @notice Event sent when an address joins the pool
    /// @param who The account that joined the pool
    /// @param pledgedAmount The amount pledged
    event JoinedPool(address indexed who, uint256 pledgedAmount);

    /// @notice Event sent when an address leaves the pool
    /// @param who The account that left the pool
    /// @param pledgedAmount The amount that was previously pledged
    event LeftPool(address indexed who, uint256 pledgedAmount);

    /// @notice The pool doesn't have enough participants to start
    error NotEnoughParticipants(uint256 value, uint256 required);

    /// @notice The pool doesn't accept additional participants
    error TooManyParticipants(uint256 value, uint256 required);

    /// @notice The participant is already in pool
    error AlreadyInPool(address who);

    /// @notice The participant is not in pool
    error NotInPool(address who);

    /// @notice There are not enough fee to join the pool
    error NotEnoughFee(uint256 value, uint256 required);

    /// @notice The minimum number of participants required in pool
    uint256 public MIN_PARTICIPANTS = 3;

    /// @notice The maximum number of participants allowed in thge pool
    /// @dev This is merely to limit the size of the participants array under a single uint8
    uint256 public MAX_PARTICIPANTS = 255;

    /// @notice The fee needed to participate in the pool
    uint256 public PARTICIPATION_FEE = 1 ether;

    /// @notice The current voting round
    /// @dev This is used to keep track of votes given per voting round
    uint256 public votingRound;

    /// @notice true, if voting is in progress
    bool isVoting = false;

    /// @notice The current staker
    address public staker;

    /// @notice The current governor
    address public governor;

    /// @notice The current amount of token at stake
    uint256 public staked;

    /// @notice The current amount of token used in council
    uint256 public governance;

    /// @notice The current amount of freely avilable tokens
    uint256 public totalPooled;

    /// @notice the owner of the contract
    address owner;

    /// @notice The type of proxy role
    /// @param Governor Allowed to perform both council voting and staking operations
    /// @param Staker Only allowed to perform staking operations
    enum RoleType {
        Governor,
        Staker
    }

    /// @notice Stores participant information
    /// @param isValid Stores true is the object is valid (differentiates from zero-value struct)
    /// @param keyIndex The index of the participant address in the keys array
    /// @param pledgedAmount The amount pledged to the pool
    struct Participant {
        bool isValid;
        uint8 keyIndex;
        uint256 pledgedAmount;
    }

    /// @notice The addresses of all participants
    /// @dev Used to iterate over all participants and compute the winner
    address[] public participantKeys;

    /// @notice The participants with their pledged amount and index in the keys array
    mapping(address => uint256) public participants;
    // mapping(address => Participant) public participants;

    mapping(uint256 => mapping(RoleType => mapping(address => uint256))) votesReceived;
    mapping(uint256 => mapping(address => bool)) votesGiven;

    constructor() {
        owner = msg.sender;
        staker = address(0);
        governor = address(0);
        votingRound = 0;
        totalPooled = 0;
    }

    /// @notice Join the pool of participants
    /// @dev Each participant stakes a minimum of PARTICIPATION_FEE
    /// @dev The pool can have a maximum of MAX_PARTICIPANTS
    function joinPool() external payable {
        address sender = msg.sender;
        uint256 amount = msg.value;
        Participant memory participant = participants[sender];
        if (participant.isValid) {
            revert AlreadyInPool(sender);
        }
        if (participantKeys.length >= MAX_PARTICIPANTS) {
            revert TooManyParticipants(
                participantKeys.length,
                MAX_PARTICIPANTS
            );
        }
        if (amount < PARTICIPATION_FEE) {
            revert NotEnoughFee(amount, PARTICIPATION_FEE);
        }

        participants[sender] = Participant(
            true,
            uint8(participantKeys.length),
            amount
        );
        participantKeys.push(sender);
        totalPooled += amount;

        emit JoinedPool(sender, amount);
    }

    /// @notice Leave the pool of participants
    /// @dev When a participant leaves the pool any associated proxis are removed
    function leavePool() external {
        address payable sender = payable(msg.sender);
        Participant memory participant = participants[sender];

        if (!participant.isValid) {
            revert NotInPool(sender);
        }

        address lastKey = participantKeys[participantKeys.length - 1];
        participantKeys[participant.keyIndex] = lastKey;
        participants[lastKey].keyIndex = participant.keyIndex;
        delete participantKeys[participantKeys.length - 1];
        delete participants[sender];

        if (sender == governor) {
            proxy.removeProxy(staker, Proxy.ProxyType.Governance, 0);
            emit ProxyRemoved(staker, Proxy.ProxyType.Governance);
            governor = address(0);
        }

        if (sender == staker) {
            proxy.removeProxy(staker, Proxy.ProxyType.Staking, 0);
            emit ProxyRemoved(staker, Proxy.ProxyType.Staking);
            staker = address(0);
        }

        totalPooled -= participant.pledgedAmount;
        sender.transfer(participant.pledgedAmount);
        emit LeftPool(sender, participant.pledgedAmount);
    }

    /// @notice Starts the next round of voting for the Staker and Governor roles
    /// @dev Requires MIN_PARTICIPANTS before voting can begin
    function startVoting() external onlyOwner {
        if (isVoting) {
            revert VotingAlreadyInProgress();
        }

        if (participantKeys.length < MIN_PARTICIPANTS) {
            revert NotEnoughParticipants(
                participantKeys.length,
                MIN_PARTICIPANTS
            );
        }

        votingRound += 1;
        isVoting = true;
        emit VotingStarted(votingRound);
    }

    /// @notice Ends a voting round
    /// @dev The participant to receive maximum votes in each role category is made the Staker and
    /// @dev the Governor, respectively. A single participant is allowed posses both roles
    function endVoting() external onlyOwner {
        if (!isVoting) {
            revert VotingNotInProgress();
        }

        isVoting = false;

        uint256 maxGovernorVotesSoFar = 0;
        address winnerGovernor;
        for (uint8 i = 0; i < participantKeys.length; i++) {
            address candidate = participantKeys[i];
            uint256 votes = votesReceived[votingRound][RoleType.Governor][
                candidate
            ];
            if (votes > maxGovernorVotesSoFar) {
                maxGovernorVotesSoFar = votes;
                winnerGovernor = candidate;
            }
        }

        uint256 maxStakerVotesSoFar = 0;
        address winnerStaker;
        for (uint8 i = 0; i < participantKeys.length; i++) {
            address candidate = participantKeys[i];
            uint256 votes = votesReceived[votingRound][RoleType.Staker][
                candidate
            ];
            if (votes > maxStakerVotesSoFar) {
                maxStakerVotesSoFar = votes;
                winnerStaker = candidate;
            }
        }

        // remove previous governor
        if (governor != address(0)) {
            proxy.removeProxy(governor, Proxy.ProxyType.Governance, 0);
            emit ProxyRemoved(governor, Proxy.ProxyType.Staking);
        }

        // remove previous staker
        if (staker != address(0)) {
            proxy.removeProxy(staker, Proxy.ProxyType.Staking, 0);
            emit ProxyRemoved(staker, Proxy.ProxyType.Staking);
        }

        proxy.addProxy(winnerGovernor, Proxy.ProxyType.Governance, 1);
        emit ProxyAdded(winnerGovernor, Proxy.ProxyType.Staking);

        proxy.addProxy(winnerStaker, Proxy.ProxyType.Staking, 1);
        emit ProxyAdded(winnerStaker, Proxy.ProxyType.Staking);

        governor = winnerGovernor;
        staker = winnerStaker;

        emit VotingEnded(votingRound, winnerStaker, winnerGovernor);
    }

    /// @notice Vote for the Governor and the Staker candidate for the current voting round
    /// @dev Each participant may vote only once and once cast the vote may not be changed
    function vote(address governorCandidate, address stakerCandidate) external {
        if (!isVoting) {
            revert VotingNotInProgress();
        }

        address sender = msg.sender;
        Participant memory participant = participants[sender];
        Participant memory governorParticipant = participants[
            governorCandidate
        ];
        Participant memory stakerParticipant = participants[stakerCandidate];

        if (votesGiven[votingRound][sender]) {
            revert AlreadyVoted(votingRound);
        }
        if (!participant.isValid) {
            revert NotInPool(sender);
        }
        if (!governorParticipant.isValid) {
            revert NotInPool(governorCandidate);
        }
        if (!stakerParticipant.isValid) {
            revert NotInPool(stakerCandidate);
        }

        votesGiven[votingRound][sender] = true;
        votesReceived[votingRound][RoleType.Governor][governorCandidate] += 1;
        votesReceived[votingRound][RoleType.Staker][stakerCandidate] += 1;
    }

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
}
