// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/referenda/Referenda.sol";
import "../../../precompiles/preimage/Preimage.sol";
import "../../../precompiles/conviction-voting/ConvictionVoting.sol";
import "./SubstrateTools.sol";

/// @notice Smart contract to verify some of the precompile methods
contract ReferendaChecker {
    /// @notice The id of the track used for root execution
    uint16 public rootTrackId;

    /// @notice The id of the referendum
    uint32 public referendumId;

    /// @notice construct the smart contract with the track id to send the proposal to
    constructor() {
        bytes memory remarkProposal = SubstrateTools.buildSystemRemarkProposal(
            "Referenda Test Contract"
        );
        referendumId = REFERENDA_CONTRACT.submitAfter(
            rootTrackId,
            PREIMAGE_CONTRACT.notePreimage(remarkProposal),
            uint32(remarkProposal.length),
            1
        );
    }

    function test_1_check_referendum() external view {
        // TODO: implement referenda precompile getters
    }
}
