// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/assets-erc20/ERC20.sol";

contract ERC20Instance is IERC20 {
    /// The ierc20 at the known pre-compile address.
    IERC20 public erc20 = IERC20(0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080);
    address erc20address = 0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080;

    receive() external payable {
        // React to receiving ether
    }

    function name() external view override returns (string memory) {
        // We nominate our target collator with all the tokens provided
        return erc20.name();
    }

    function symbol() external view override returns (string memory) {
        // We nominate our target collator with all the tokens provided
        return erc20.symbol();
    }

    function decimals() external view override returns (uint8) {
        // We nominate our target collator with all the tokens provided
        return erc20.decimals();
    }

    function totalSupply() external view override returns (uint256) {
        // We nominate our target collator with all the tokens provided
        return erc20.totalSupply();
    }

    function totalSupply_static() public returns (uint256) {
        (bool result, bytes memory data) = erc20address.staticcall(
            abi.encodeWithSignature("totalSupply()")
        );
        require(result == true);
        return uint256(bytes32(data));
    }

    function balanceOf(address who) external view override returns (uint256) {
        // We nominate our target collator with all the tokens provided
        return erc20.balanceOf(who);
    }

    function allowance(
        address owner,
        address spender
    ) external view override returns (uint256) {
        return erc20.allowance(owner, spender);
    }

    function allowance_static(
        address _owner,
        address _spender
    ) external returns (bytes memory) {
        (bool result, bytes memory data) = erc20address.staticcall(
            abi.encodeWithSignature(
                "allowance(address, address)",
                _owner,
                _spender
            )
        );
        return data;
    }

    function transfer(
        address to,
        uint256 value
    ) external override returns (bool) {
        return erc20.transfer(to, value);
    }

    function transfer_delegate(
        address to,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = erc20address.delegatecall(
            abi.encodeWithSignature("transfer(address,uint256)", to, value)
        );
        return result;
    }

    function transfer_static(
        address to,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = erc20address.staticcall(
            abi.encodeWithSignature("transfer(address,uint256)", to, value)
        );
        return result;
    }

    function approve(
        address spender,
        uint256 value
    ) external override returns (bool) {
        return erc20.approve(spender, value);
    }

    function approve_max_supply(address spender) public returns (bool) {
        uint256 total = totalSupply_static();
        return erc20.approve(spender, total);
    }

    function approve_delegate(
        address spender,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = erc20address.delegatecall(
            abi.encodeWithSignature("approve(address,uint256)", spender, value)
        );
        return result;
    }

    function approve_ext_delegate(address spender, uint256 value) external {
        (bool result, bytes memory data) = address(this).delegatecall(
            abi.encodeWithSignature("approve(address,uint256)", spender, value)
        );
        require(result, string(data));
    }

    function approve_static(
        address spender,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = erc20address.staticcall(
            abi.encodeWithSignature("approve(address,uint256)", spender, value)
        );
        return result;
    }

    function approve_ext_static(address spender, uint256 value) external {
        (bool result, bytes memory data) = address(this).staticcall(
            abi.encodeWithSignature("approve(address,uint256)", spender, value)
        );
        require(result, string(data));
    }

    function approve_delegate_to_static(
        address spender,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = address(this).delegatecall(
            abi.encodeWithSignature(
                "approve_ext_static(address,uint256)",
                spender,
                value
            )
        );
        return result;
    }

    function approve_static_to_delegate(
        address spender,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = address(this).staticcall(
            abi.encodeWithSignature(
                "approve_ext_delegate(address,uint256)",
                spender,
                value
            )
        );
        return result;
    }

    function transferFrom(
        address from,
        address to,
        uint256 value
    ) external override returns (bool) {
        return erc20.transferFrom(from, to, value);
    }

    function transferFrom_delegate(
        address from,
        address to,
        uint256 value
    ) external returns (bool) {
        (bool result, bytes memory data) = erc20address.delegatecall(
            abi.encodeWithSignature(
                "transferFrom(address,address,uint256)",
                from,
                to,
                value
            )
        );
        return result;
    }
}
