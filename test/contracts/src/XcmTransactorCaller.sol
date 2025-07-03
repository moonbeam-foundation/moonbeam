// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Contracts ^5.0.0
pragma solidity >=0.8.25;

import "../../../precompiles/xcm-transactor/src/v1/XcmTransactorV1.sol";
import "../../../precompiles/xcm-transactor/src/v3/XcmTransactorV3.sol";

contract XcmTransactorCaller  {
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    function transactThroughDerivativeV1(uint8 _transactor, uint16 _index, address _feeToken, uint64 _weight, bytes calldata _innerCall) public  {
        require(msg.sender == owner, "Not owner");
        XCM_TRANSACTOR_V1_CONTRACT.transactThroughDerivative(
            _transactor,
            _index,
            _feeToken,
            _weight,
            _innerCall
        );
    }

    function transactThroughDerivativeMultilocationV1(uint8 _transactor, uint16 _index, XcmTransactorV1.Multilocation calldata _feeAssetML, uint64 _weight, bytes calldata _innerCall) public  {
        require(msg.sender == owner, "Not owner");
        XCM_TRANSACTOR_V1_CONTRACT.transactThroughDerivativeMultilocation(
            _transactor,
            _index,
            _feeAssetML,
            _weight,
            _innerCall
        );
    }

    function transactThroughDerivativeV3(
        uint8 _transactor,
        uint16 _index,
        address _feeToken,
        XcmTransactorV3.Weight calldata _weightTransact,
        bytes calldata _innerCall,
        uint256 _feeAmount,
        XcmTransactorV3.Weight calldata _weightOverall,
        bool refund
    ) public  {
        require(msg.sender == owner, "Not owner");
        XCM_TRANSACTOR_V3_CONTRACT.transactThroughDerivative(
            _transactor,
            _index,
            _feeToken,
            _weightTransact,
            _innerCall,
            _feeAmount,
            _weightOverall,
            refund
        );
    }

    function transactThroughDerivativeMultilocationV3(
        uint8 _transactor,
        uint16 _index,
        XcmTransactorV3.Multilocation calldata _feeAssetML,
        XcmTransactorV3.Weight calldata _weightTransact,
        bytes calldata _innerCall,
        uint256 _feeAmount,
        XcmTransactorV3.Weight calldata _weightOverall,
        bool refund
    ) public  {
        require(msg.sender == owner, "Not owner");
        XCM_TRANSACTOR_V3_CONTRACT.transactThroughDerivativeMultilocation(
            _transactor,
            _index,
            _feeAssetML,
            _weightTransact,
            _innerCall,
            _feeAmount,
            _weightOverall,
            refund
        );
    }

}