// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @title Collective precompile
/// Allows to interact with Substrate pallet_collective from the EVM.
/// Address: TODO
interface Collective {
    function execute(bytes memory proposal, uint32 lengthBound) external;

    function propose(uint32 threshold, bytes memory proposal) external;

    function vote(
        bytes32 proposalHash,
        uint32 proposalIndex,
        bool approve
    ) external;

    function close(
        bytes32 proposalHash,
        uint32 proposalIndex,
        uint64 proposalWeightBound
    ) external;

    function proposalHash(bytes memory proposal)
        external
        view
        returns (bytes32 proposalHash);

    event Executed(bytes32 indexed proposalHash);
    event Proposed(
        address indexed who,
        uint32 indexed proposalIndex,
        bytes32 indexed proposalHash,
        uint32 threshold
    );
    event Voted(
        address indexed who,
        bytes32 indexed proposalHash,
        bool voted,
        uint32 yesVotes,
        uint32 noVotes
    );
}
