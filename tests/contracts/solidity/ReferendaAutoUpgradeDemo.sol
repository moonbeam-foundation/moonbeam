// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/referenda/Referenda.sol";
import "../../../precompiles/preimage/Preimage.sol";
import "../../../precompiles/conviction-voting/ConvictionVoting.sol";
import "./SubstrateTools.sol";

/// @notice Smart contract to demonstrate how to use Referenda Precompile to self-upgrade
abstract contract ReferendaAutoUpgradeDemo {
    /// @notice The id of the track used for root execution
    uint16 public rootTrackId;

    /// @notice SetStorageCall index (pallet Index concatenated with call index, usually "\x00\x04")
    bytes2 public setStorageCallIndex;

    /// @notice construct the smart contract with the track id to send the proposal to
    constructor(string memory trackName, bytes2 pSetStorageCallIndex) {
        setStorageCallIndex = pSetStorageCallIndex;

        // This is obviously NOT THE RIGHT WAY to find/store the track id.
        // The track id should be passed as an argument of the constructor instead of the
        // track name.
        // In this demo, it is using the track name to show how to loop through the trackIds.
        rootTrackId = getTrackId(trackName);
    }

    /// @notice retrieves the track id matching the track name
    /// @notice this is ineficient and only used for a demo.
    function getTrackId(string memory trackName)
        internal
        view
        returns (uint16)
    {
        uint16[] memory trackIds = REFERENDA_CONTRACT.trackIds();
        for (uint256 i = 0; i < trackIds.length; i++) {
            Referenda.TrackInfo memory info = REFERENDA_CONTRACT.trackInfo(
                trackIds[i]
            );

            if (
                keccak256(abi.encodePacked((info.name))) ==
                keccak256(abi.encodePacked((trackName))) // Compare the 2 strings
            ) {
                return trackIds[i];
            }
        }
        revert("Couldn't find track");
    }

    /// @notice submits to upgrade contract for given storage key
    /// @param contractCode The code as deployed of the new contract
    /// @param contractStorageKey The storage key associated with the current smart contract
    function autoUpgrade(
        bytes memory contractCode,
        bytes memory contractStorageKey
    ) public {
        bytes memory setStorageCall = SubstrateTools.buildSetStorageProposal(
            contractStorageKey,
            contractCode
        );
        bytes32 preimageHash = PREIMAGE_CONTRACT.notePreimage(setStorageCall);

        // /// If the block count is lower than the minimum allowed, it will pick the minimum
        // /// allowed automatically.
        uint32 blockCount = 1;
        uint32 referendumId = REFERENDA_CONTRACT.submitAfter(
            rootTrackId,
            preimageHash,
            uint32(setStorageCall.length),
            blockCount
        );

        /// Directly place the deposit
        REFERENDA_CONTRACT.placeDecisionDeposit(referendumId);

        /// Vote for the referendum
        CONVICTION_VOTING_CONTRACT.voteYes(
            referendumId,
            address(this).balance, // Uses all the contract available balance
            ConvictionVoting.Conviction.Locked1x
        );
    }
}

/// @notice First version of the contract
contract ReferendaAutoUpgradeDemoV1 is ReferendaAutoUpgradeDemo {
    constructor(string memory trackName, bytes2 pSetStorageCallIndex)
        ReferendaAutoUpgradeDemo(trackName, pSetStorageCallIndex)
    {}

    function version() external pure returns (uint256) {
        return 1;
    }
}

/// @notice Second version of the contract
contract ReferendaAutoUpgradeDemoV2 is ReferendaAutoUpgradeDemo {
    constructor(string memory trackName, bytes2 pSetStorageCallIndex)
        ReferendaAutoUpgradeDemo(trackName, pSetStorageCallIndex)
    {}

    function version() external pure returns (uint256) {
        return 2;
    }
}
