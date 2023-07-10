// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/proxy/Proxy.sol";
import "../../../precompiles/parachain-staking/StakingInterface.sol";

/// @notice Smart contract to demonstrate how to use Proxy::call
contract ProxyCallStakingDemo {
    /// @notice The participant did not have "Staking" proxy type for the contract address
    error ParticipantMustHaveStakingProxy();

    /// @notice The participant does not exist
    error InvalidParticipant(address who);

    /// @notice The candidate does not exist
    error InvalidCandidate(address who);

    /// @notice The participant already exists
    error ParticipantExists(address who);

    /// @notice The candidate already exists
    error CandidateExists(address who);

    /// @notice The pool doesn't accept additional participants
    /// @param value The value that was given
    /// @param required The value that was expected
    error TooManyParticipants(uint256 value, uint256 required);

    /// @notice The pool doesn't accept additional participants
    /// @param value The value that was given
    /// @param required The value that was expected
    error TooManyCandidates(uint256 value, uint256 required);

    error NotEnoughParticipationAmount(uint256 value, uint256 required);

    /// @notice Event sent when an participant joins the pool
    /// @param who The account that joined the pool
    event ParticipantAdded(address indexed who);

    /// @notice Event sent when an participant leaves the pool
    /// @param who The account that left the pool
    event ParticipantRemoved(address indexed who);

    /// @notice Event sent when a candidate joins the pool
    /// @param who The account that joined the pool
    event CandidateAdded(address indexed who);

    /// @notice Event sent when a candidate leaves the pool
    /// @param who The account that left the pool
    event CandidateRemoved(address indexed who);

    /// @notice Stores participant information
    /// @param isValid Stores true is the object is valid (differentiates from zero-value struct)
    /// @param keyIndex The index of the participant address in the keys array
    /// @param delegationCount The number of unique delegations
    struct Participant {
        bool isValid;
        uint8 keyIndex;
        uint16 delegationCount;
    }

    /// @notice The addresses of all participants
    /// @dev Used to iterate over all participants
    address[] participantKeys;

    /// @notice The participants with their delegation count  and index in the keys array
    /// @dev The key index is used to quickly remove the address from the `participantKeys` array
    mapping(address => Participant) participants;

    /// @notice Stores candidate information
    /// @param isValid Stores true is the object is valid (differentiates from zero-value struct)
    /// @param keyIndex The index of the participant address in the keys array
    /// @param delegationCount The number of unique delegations
    struct Candidate {
        bool isValid;
        uint8 keyIndex;
        uint16 delegationCount;
    }

    /// @notice The addresses of all candidates
    /// @dev Used to iterate over all candidates
    address[] candidateKeys;

    /// @notice The candidates with their delegation count and index in the keys array
    /// @dev The key index is used to quickly remove the address from the `candidateKeys` array
    mapping(address => Candidate) candidates;

    /// @notice The owner of the contract
    address owner;

    /// @notice The maximum number of participants allowed in the pool
    /// @dev This is merely to limit the size of the participants array under a single uint8
    uint256 public MAX_PARTICIPANTS = 255;

    /// @notice The maximum number of participants allowed in the pool
    /// @dev This is merely to limit the size of the participants array under a single uint8
    uint256 public MAX_CANDIDATES = 255;

    /// @notice The delegation amount to be delegated to each incoming candidate.
    uint256 public DELEGATE_AMOUNT = 1000000000000000000;

    /// @notice The autocompound percent for each delegation.
    uint256 public AUTOCOMPOUND_PERCENT = 100;

    constructor() payable {
        owner = msg.sender;
    }

    /// @notice Returns true if the address is an existing participant
    function isParticipant(address who) public view returns (bool) {
        return participants[who].isValid;
    }

    /// @notice Returns true if the address is an existing candidate
    function isCandidate(address who) public view returns (bool) {
        return candidates[who].isValid;
    }

    function hasStakingProxy(address who) private view returns (bool) {
        return
            PROXY_CONTRACT.isProxy(
                who,
                address(this),
                Proxy.ProxyType.Staking,
                0
            ) ||
            PROXY_CONTRACT.isProxy(
                who,
                address(this),
                Proxy.ProxyType.Governance,
                0
            ) ||
            PROXY_CONTRACT.isProxy(who, address(this), Proxy.ProxyType.Any, 0);
    }

    /// @notice Join the pool of participants
    /// @dev Each participant stakes the DELEGATE_AMOUNT and must have "Staking" proxy type for the contract address
    /// @dev The pool can have a maximum of MAX_PARTICIPANTS
    /// @dev Any existing candidates automatically get a delegation from the new participant
    function join(uint16 delegationCount) external {
        address sender = address(msg.sender);
        Participant memory participant = participants[sender];
        if (participant.isValid) {
            revert ParticipantExists(sender);
        }
        if (participantKeys.length >= MAX_PARTICIPANTS) {
            revert TooManyParticipants(
                participantKeys.length,
                MAX_PARTICIPANTS
            );
        }
        if (sender.balance < DELEGATE_AMOUNT) {
            revert NotEnoughParticipationAmount(
                sender.balance,
                DELEGATE_AMOUNT
            );
        }
        if (!hasStakingProxy(sender)) {
            revert ParticipantMustHaveStakingProxy();
        }

        participants[sender] = Participant(
            true,
            uint8(participantKeys.length),
            delegationCount
        );
        participantKeys.push(sender);

        // stake to all existing candidates
        for (uint i = 0; i < candidateKeys.length; i++) {
            address candidateAddress = candidateKeys[i];
            bytes memory delegateWithAutoCompoundCalldata = abi
                .encodeWithSignature(
                    "delegateWithAutoCompound(address,uint256,uint8,uint256,uint256,uint256)",
                    candidateAddress,
                    DELEGATE_AMOUNT,
                    AUTOCOMPOUND_PERCENT,
                    candidates[candidateAddress].delegationCount,
                    candidates[candidateAddress].delegationCount,
                    participants[sender].delegationCount
                );
            PROXY_CONTRACT.proxy(
                sender,
                PARACHAIN_STAKING_ADDRESS,
                delegateWithAutoCompoundCalldata
            );
            candidates[candidateAddress].delegationCount += 1;
            participants[sender].delegationCount += 1;
        }

        emit ParticipantAdded(sender);
    }

    /// @notice Leave the pool of participants
    /// @dev When a participant leaves the pool, all existing delegations schedule a revoke, if possible
    function leave() external {
        address sender = address(msg.sender);
        Participant memory participant = participants[sender];

        if (!participant.isValid) {
            revert InvalidParticipant(sender);
        }
        if (!hasStakingProxy(sender)) {
            revert ParticipantMustHaveStakingProxy();
        }

        address lastKey = participantKeys[participantKeys.length - 1];
        participantKeys[participant.keyIndex] = lastKey;
        participants[lastKey].keyIndex = participant.keyIndex;
        delete participantKeys[participantKeys.length - 1];
        delete participants[sender];

        // unstake from all candidates
        for (uint i = 0; i < candidateKeys.length; i++) {
            address candidateAddress = candidateKeys[i];
            bytes memory revokeDelegationCalldata = abi.encodeWithSignature(
                "scheduleRevokeDelegation(address)",
                candidateAddress
            );
            PROXY_CONTRACT.proxy(
                sender,
                PARACHAIN_STAKING_ADDRESS,
                revokeDelegationCalldata
            );
            candidates[candidateAddress].delegationCount -= 1;
        }

        emit ParticipantRemoved(sender);
    }

    /// @notice Register a candidate
    /// @dev The pool can have a maximum of MAX_CANDIDATES
    /// @dev When a candidate registers all existing participants delegate to them, if possible
    function registerCandidate(uint16 delegationCount) external {
        address sender = msg.sender;
        Candidate memory candidate = candidates[sender];
        if (candidate.isValid) {
            revert CandidateExists(sender);
        }
        if (candidateKeys.length >= MAX_CANDIDATES) {
            revert TooManyCandidates(candidateKeys.length, MAX_CANDIDATES);
        }

        candidates[sender] = Candidate(
            true,
            uint8(candidateKeys.length),
            delegationCount
        );
        candidateKeys.push(sender);

        // delegate from all existing participants
        for (uint i = 0; i < participantKeys.length; i++) {
            address participantAddress = participantKeys[i];
            if (!hasStakingProxy(participantAddress)) {
                continue;
            }

            bytes memory delegateWithAutoCompoundCalldata = abi
                .encodeWithSignature(
                    "delegateWithAutoCompound(address,uint256,uint8,uint256,uint256,uint256)",
                    sender,
                    DELEGATE_AMOUNT,
                    AUTOCOMPOUND_PERCENT,
                    candidates[sender].delegationCount,
                    candidates[sender].delegationCount,
                    participants[participantAddress].delegationCount
                );
            PROXY_CONTRACT.proxy(
                participantAddress,
                PARACHAIN_STAKING_ADDRESS,
                delegateWithAutoCompoundCalldata
            );
            candidates[sender].delegationCount += 1;
            participants[participantAddress].delegationCount += 1;
        }

        emit CandidateAdded(sender);
    }

    /// @notice Leave the pool of candidates
    /// @dev When a candidate leaves all participants schedule a revoke onto them
    function unregisterCandidate() external {
        address sender = address(msg.sender);
        Candidate memory candidate = candidates[sender];
        if (!candidate.isValid) {
            revert InvalidCandidate(sender);
        }

        address lastKey = candidateKeys[candidateKeys.length - 1];
        candidateKeys[candidate.keyIndex] = lastKey;
        candidates[lastKey].keyIndex = candidate.keyIndex;
        delete candidateKeys[candidateKeys.length - 1];
        delete candidates[sender];

        // unstake from all participants
        for (uint i = 0; i < participantKeys.length; i++) {
            address participantAddress = participantKeys[i];
            if (!hasStakingProxy(participantAddress)) {
                continue;
            }
            bytes memory revokeDelegationCalldata = abi.encodeWithSignature(
                "scheduleRevokeDelegation(address)",
                sender
            );
            PROXY_CONTRACT.proxy(
                participantAddress,
                PARACHAIN_STAKING_ADDRESS,
                revokeDelegationCalldata
            );
            participants[participantAddress].delegationCount -= 1;
        }

        emit CandidateRemoved(sender);
    }

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
}
