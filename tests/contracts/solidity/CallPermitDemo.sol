// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/call-permit/CallPermit.sol";

/// @notice Smart contract to demonstrate how to use Call Permit precompile
contract CallPermitDemo {
    /// @notice The bond amount is too low
    error BondAmountTooLow(uint256 value, uint256 required);

    /// @notice The bond does not exist
    error NoBond();

    /// @notice The bond already exists
    error AlreadyBonded();

    /// @notice A user bonded
    event Bonded(address indexed who, uint256 amount);

    /// @notice A user bonded on behalf of someone else
    event BondedFor(address via, address indexed who, uint256 amount);

    /// @notice A user unbonded
    event Unbonded(address indexed who, uint256 amount);

    /// @notice The fixed amound that needs to be bonded
    uint256 public BOND_AMOUNT = 100;

    /// @notice The total pooled amount
    uint256 public bondedAmount;

    /// @notice A mapping of bond per account
    mapping(address => uint256) bonds;

    /// @notice The owner of the contract
    address owner;

    constructor() {
        owner = msg.sender;
        bondedAmount = 0;
    }

    /// @notice Bonds BOND_AMOUNT on someone else's behalf using a signed EIP712 Message
    /// @param from The real signer of the permit
    /// @param gaslimit The maximum gas limit
    /// @param deadline The deadline for the permit
    /// @param v The v parameter of the permit signature
    /// @param r The r parameter of the permit signature
    /// @param s The s parameter of the permit signature
    /// @dev the request is fulfilled
    function bondFor(
        address from,
        uint64 gaslimit,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        uint256 bondAmount = bonds[from];
        if (bondAmount != 0) {
            revert AlreadyBonded();
        }

        CALL_PERMIT_CONTRACT.dispatch(
            from,
            address(this),
            BOND_AMOUNT,
            "", // transfer
            gaslimit,
            deadline,
            v,
            r,
            s
        );

        bonds[from] = BOND_AMOUNT;
        bondedAmount += BOND_AMOUNT;
    }

    /// @notice Bonds BOND_AMOUNT towards the pool
    function bond() external payable {
        address sender = msg.sender;
        uint256 amount = msg.value;
        uint256 bondAmount = bonds[sender];

        if (bondAmount != 0) {
            revert AlreadyBonded();
        }

        if (amount < BOND_AMOUNT) {
            revert BondAmountTooLow(amount, BOND_AMOUNT);
        }

        bonds[sender] += amount;
        bondedAmount += amount;
    }

    /// @notice Unbonds BOND_AMOUNT from the pool
    function unbond() external {
        address payable sender = payable(msg.sender);
        uint256 bondAmount = bonds[sender];
        if (bondAmount == 0) {
            revert NoBond();
        }

        bonds[sender] -= bondAmount;
        bondedAmount -= bondAmount;

        sender.transfer(bondAmount);
    }

    /// @notice Returns the total bonded acount
    function getBondAmount(address who) external view returns (uint256) {
        return bonds[who];
    }

    /// @notice Receive funds
    /// @dev This is needed to allow the contract to accept transfers via call permit
    receive() external payable {}

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
}
