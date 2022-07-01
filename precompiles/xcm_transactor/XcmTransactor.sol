// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @title Xcm Transactor Interface
/// The interface through which solidity contracts will interact with xcm transactor pallet
/// Address :    0x0000000000000000000000000000000000000806
interface XcmTransactor {

    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes [] interior;
    }

    /// Selector: 3fdc4f36
    /// @param index The index of which we want to retrieve the account
    function indexToAccount(uint16 index) external view returns(address);

    /// Selector: c0282147
    /// @param index The index of which we want to retrieve the account
    function transactInfo(Multilocation memory multilocation) external view 
        returns(uint64, uint256, uint64, uint64, uint256);

   /// Selector: afb11701
   /// @dev The token transfer burns/transfers the corresponding amount before sending
   /// @param transactor The transactor to be used
   /// @param index The index to be used
   /// @param feeAsset The asset in which we want to pay fees. 
   /// It has to be a reserve of the destination chain
   /// @param weight The weight we want to buy in the destination chain
   /// @param innerCall The inner call to be executed in the destination chain
    function transactThroughDerivativeMultilocation(
        uint8 transactor,
        uint16 index,
        Multilocation memory feeAsset,
        uint64 weight,
        bytes memory innerCall
    ) external;
    
   /// Selector: 02ae072d
   /// @dev The token transfer burns/transfers the corresponding amount before sending
   /// @param transactor The transactor to be used
   /// @param index The index to be used
   /// @param currencyId Address of the currencyId of the asset to be used for fees
   /// It has to be a reserve of the destination chain
   /// @param weight The weight we want to buy in the destination chain
   /// @param innerCall The inner call to be executed in the destination chain
    function transactThroughDerivative(
        uint8 transactor,
        uint16 index,
        address currencyId,
        uint64 weight,
        bytes memory innerCall
    ) external;
    
}