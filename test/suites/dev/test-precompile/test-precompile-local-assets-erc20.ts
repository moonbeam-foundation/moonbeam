import "@moonbeam-network/api-augment";
import { beforeEach, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  DOROTHY_ADDRESS,
  alith,
  baltathar,
} from "@moonwall/util";
import { u8aToString } from "@polkadot/util";
import { getAddress, keccak256, toBytes } from "viem";
import { registerLocalAssetWithMeta } from "../../../helpers/assets.js";

describeSuite({
  id: "D2538",
  title: "Precompiles - Assets-ERC20 Wasm",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetAddress: string;
    let assetId: string;
    let localAssetContractAddress: `0x${string}`;

    beforeEach(async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: baltathar, amount: 100000000000000n }],
      }));

      // Set team
      await context.createBlock(
        context
          .polkadotJs()
          .tx.localAssets // Issuer, admin, freezer
          .setTeam(assetId, BALTATHAR_ADDRESS, CHARLETH_ADDRESS, DOROTHY_ADDRESS)
          .signAsync(baltathar)
      );

      // Set owner
      await context.createBlock(
        context
          .polkadotJs()
          .tx.localAssets // owner
          .transferOwnership(assetId, ALITH_ADDRESS)
          .signAsync(baltathar)
      );

      const { contractAddress } = await deployCreateCompiledContract(
        context,
        "LocalAssetExtendedErc20Instance"
      );
      localAssetContractAddress = contractAddress;

      await context.writeContract!({
        contractName: "LocalAssetExtendedErc20Instance",
        functionName: "set_address_interface",
        contractAddress: localAssetContractAddress,
        args: [getAddress(assetAddress)],
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "allows to call name",
      test: async function () {
        const name = await context.readContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "name",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(name).equals("Local");
      },
    });

    it({
      id: "T02",
      title: "allows to call symbol",
      test: async function () {
        const symbol = await context.readContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "symbol",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(symbol).equals("Local");
      },
    });

    it({
      id: "T03",
      title: "allows to call decimals",
      test: async function () {
        const bal = await context.readContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "decimals",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(bal).equals(12);
      },
    });

    it({
      id: "T04",
      title: "allows to call getBalance",
      test: async function () {
        const balanceOf = await context.readContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(balanceOf).equals(100000000000000n);
      },
    });

    it({
      id: "T05",
      title: "allows to call totalSupply",
      test: async function () {
        const totalSupply = await context.readContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "totalSupply",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(totalSupply).equals(100000000000000n);
      },
    });

    it({
      id: "T06",
      title: "allows to call owner",
      test: async function () {
        const owner = await context.readContract!({
          contractName: "Roles",
          functionName: "owner",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(owner).equals(ALITH_ADDRESS);
      },
    });

    it({
      id: "T07",
      title: "allows to call freezer",
      test: async function () {
        const freezer = await context.readContract!({
          contractName: "Roles",
          functionName: "freezer",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(freezer).equals(DOROTHY_ADDRESS);
      },
    });

    it({
      id: "T08",
      title: "allows to call admin",
      test: async function () {
        const admin = await context.readContract!({
          contractName: "Roles",
          functionName: "admin",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(admin).equals(CHARLETH_ADDRESS);
      },
    });

    it({
      id: "T09",
      title: "allows to call issuer",
      test: async function () {
        const issuer = await context.readContract!({
          contractName: "Roles",
          functionName: "issuer",
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(issuer).equals(BALTATHAR_ADDRESS);
      },
    });

    it({
      id: "T10",
      title: "allows to approve transfers, and allowance matches",
      test: async function () {
        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "approve",
          args: [BALTATHAR_ADDRESS, 1000],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).toBe(
          keccak256(toBytes("Approval(address,address,uint256)"))
        );

        const approvals = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, ALITH_ADDRESS, BALTATHAR_ADDRESS);
        expect(approvals.unwrap().amount.toBigInt()).toBe(1000n);
      },
    });

    it({
      id: "T11",
      title: "should have allowances",
      test: async function () {
        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "approve",
          args: [BALTATHAR_ADDRESS, 1337],
          rawTxOnly: true,
        });

        await context.createBlock(rawTx);

        const allowance = await context.readContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "allowance",
          args: [ALITH_ADDRESS, BALTATHAR_ADDRESS],
          contractAddress: assetAddress as `0x${string}`,
        });

        expect(allowance).equals(1337n);
      },
    });

    it({
      id: "T12",
      title: "allows to approve and use transferFrom",
      test: async function () {
        await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "approve",
          args: [ALITH_ADDRESS, 1000],
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        await context.createBlock();

        const rawTx2 = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "transferFrom",
          args: [BALTATHAR_ADDRESS, CHARLETH_ADDRESS, 1000],
          privateKey: ALITH_PRIVATE_KEY,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx2);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLocaleLowerCase()).to.eq(assetAddress);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).toBe(
          keccak256(toBytes("Transfer(address,address,uint256)"))
        );
        expect(receipt.status).to.equal("success");

        // Approve amount is null now
        const approvals = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, BALTATHAR_ADDRESS, ALITH_ADDRESS);
        expect(approvals.isNone).to.eq(true);

        // Charleth balance is 1000
        const charlethBalance = await context
          .polkadotJs()
          .query.localAssets.account(assetId, CHARLETH_ADDRESS);
        expect(charlethBalance.unwrap().balance.toBigInt()).to.equal(1000n);
      },
    });

    it({
      id: "T13",
      title: "allows to transfer",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "transfer",
          args: [CHARLETH_ADDRESS, 1000],
          privateKey: BALTATHAR_PRIVATE_KEY,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).toBe("success");

        const charlethBalance = await context
          .polkadotJs()
          .query.localAssets.account(assetId, CHARLETH_ADDRESS);
        expect(charlethBalance.unwrap().balance.toBigInt()).toBe(1000n);
      },
    });

    it({
      id: "T14",
      title: "allows to approve transfer and use transferFrom from contract calls",
      test: async function () {
        // register, setMeta & mint local Asset
        const { assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        });

        const { contractAddress: contractInstanceAddress } = await context.deployContract!(
          "LocalAssetExtendedErc20Instance"
        );

        // before we mint asset, since these are non-sufficient, we need to transfer native balance
        await context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transfer(contractInstanceAddress, 1000)
            .signAsync(baltathar)
        );

        // mint asset
        await context.createBlock(
          context
            .polkadotJs()
            .tx.localAssets.mint(assetId, contractInstanceAddress, 100000000000000)
            .signAsync(baltathar)
        );
        // set asset address

        await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: contractInstanceAddress,
          functionName: "set_address_interface",
          args: [getAddress(assetAddress)],
        });
        await context.createBlock();

        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: contractInstanceAddress,
          functionName: "approve",
          args: [BALTATHAR_ADDRESS, 1000],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).toBe(
          keccak256(toBytes("Approval(address,address,uint256)"))
        );

        const approvals = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, contractInstanceAddress, baltathar.address);
        expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);

        // We are gonna spend 1000 from contractInstanceAddress to send it to charleth
        // Since this is a regular call, it will take contractInstanceAddress as msg.sender
        // thus from & to will be the same, and approval wont be touched

        const rawTxn2 = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: contractInstanceAddress,
          functionName: "transferFrom",
          args: [contractInstanceAddress, CHARLETH_ADDRESS, 1000],
          rawTxOnly: true,
        });

        const { result: result2 } = await context.createBlock(rawTxn2);
        const receipt2 = await context
          .viem()
          .getTransactionReceipt({ hash: result2!.hash as `0x${string}` });

        expect(receipt2.logs.length).to.eq(1);
        expect(receipt2.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt2.logs[0].topics.length).to.eq(3);
        expect(receipt2.logs[0].topics[0]).toBe(
          keccak256(toBytes("Transfer(address,address,uint256)"))
        );
        expect(receipt2.status).to.equal("success");

        // approvals are untouched
        const approvals2 = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, contractInstanceAddress, baltathar.address);
        expect(approvals2.unwrap().amount.toBigInt()).toBe(1000n);

        // this time we call directly from Baltathar the ERC20 contract

        const { result: baltharResult } = await context.createBlock(
          context.writeContract!({
            privateKey: BALTATHAR_PRIVATE_KEY,
            contractName: "LocalAssetExtendedErc20",
            contractAddress: assetAddress as `0x${string}`,
            functionName: "transferFrom",
            rawTxOnly: true,
            args: [contractInstanceAddress, CHARLETH_ADDRESS, 1000],
          })
        );

        const receipt3 = await context
          .viem()
          .getTransactionReceipt({ hash: baltharResult!.hash as `0x${string}` });
        expect(receipt3.logs.length).to.eq(1);
        expect(receipt3.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt3.logs[0].topics.length).to.eq(3);
        expect(receipt3.logs[0].topics[0]).toBe(
          keccak256(toBytes("Transfer(address,address,uint256)"))
        );
        expect(receipt3.status).to.equal("success");

        // Approve amount is null now
        const approvals3 = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, contractInstanceAddress, baltathar.address);
        expect(approvals3.isNone).to.eq(true);

        // Charleth balance is 2000
        const charletBalance = await context
          .polkadotJs()
          .query.localAssets.account(assetId, CHARLETH_ADDRESS);
        expect(charletBalance.unwrap().balance.toBigInt()).toBe(2000n);
      },
    });

    it({
      id: "T15",
      title: "Baltathar approves contract and use transferFrom from contract calls",
      test: async function () {
        log(assetAddress);
        // Create approval
        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "approve",
          args: [localAssetContractAddress, 1000],
          privateKey: BALTATHAR_PRIVATE_KEY,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).toBe(
          keccak256(toBytes("Approval(address,address,uint256)"))
        );

        const approvals = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, BALTATHAR_ADDRESS, localAssetContractAddress);
        expect(approvals.unwrap().amount.toBigInt()).toBe(1000n);

        // We are gonna spend 1000 from Baltathar to send it to charleth from contract address
        // even if Bob calls, msg.sender will become the contract with regular calls
        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: localAssetContractAddress,
          functionName: "transferFrom",
          args: [BALTATHAR_ADDRESS, CHARLETH_ADDRESS, 1000],
          rawTxOnly: true,
        });
        const { result: baltatharResult } = await context.createBlock(rawTx);

        const receipt2 = await context
          .viem()
          .getTransactionReceipt({ hash: baltatharResult!.hash as `0x${string}` });
        expect(receipt2.logs.length).to.eq(1);
        expect(receipt2.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt2.logs[0].topics.length).to.eq(3);
        expect(receipt2.logs[0].topics[0]).toBe(
          keccak256(toBytes("Transfer(address,address,uint256)"))
        );
        expect(receipt2.status).to.equal("success");

        const approvals2 = await context
          .polkadotJs()
          .query.localAssets.approvals(assetId, BALTATHAR_ADDRESS, localAssetContractAddress);
        expect(approvals2.isNone).to.eq(true);

        const charletBalance = await context
          .polkadotJs()
          .query.localAssets.account(assetId, CHARLETH_ADDRESS);
        expect(charletBalance.unwrap().balance.toBigInt()).toBe(1000n);
      },
    });

    it({
      id: "T16",
      title: "allows to transfer through call from SC",
      test: async function () {
        // before we mint asset, since these are non-sufficient, we need to transfer native balance
        await context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transfer(localAssetContractAddress, 1000)
            .signAsync(baltathar)
        );

        // register, setMeta & mint local Asset
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
          mints: [{ account: localAssetContractAddress, amount: 100000000000000n }],
        }));

        await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          functionName: "set_address_interface",
          contractAddress: localAssetContractAddress,
          args: [getAddress(assetAddress)],
        });
        await context.createBlock();

        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: localAssetContractAddress as `0x${string}`,
          functionName: "transfer",
          args: [ALITH_ADDRESS, 1000],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const bal = await context.polkadotJs().query.localAssets.account(assetId, ALITH_ADDRESS);
        expect(bal.unwrap().balance.toBigInt()).toBe(1000n);
      },
    });

    it({
      id: "T17",
      title: "allows to mint",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "mint",
          args: [CHARLETH_ADDRESS, 1000],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTxn);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).toBe(
          keccak256(toBytes("Transfer(address,address,uint256)"))
        );

        const bal = await context.polkadotJs().query.localAssets.account(assetId, CHARLETH_ADDRESS);

        expect(bal.unwrap().balance.toBigInt()).to.equal(1000n);
      },
    });

    it({
      id: "T18",
      title: "allows to burn",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
          mints: [{ account: alith, amount: 100000000000000n }],
        }));

        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "burn",
          args: [ALITH_ADDRESS, 1000000],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).toBe(
          keccak256(toBytes("Transfer(address,address,uint256)"))
        );

        const bal = await context.polkadotJs().query.localAssets.account(assetId, ALITH_ADDRESS);

        expect(bal.unwrap().balance.toBigInt()).toBe(99999999000000n);
      },
    });

    it({
      id: "T19",
      title: "allows to freeze account",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
          mints: [{ account: alith, amount: 100000000000000n }],
        }));

        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "freeze",
          args: [ALITH_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");

        const frozen = await context.polkadotJs().query.localAssets.account(assetId, ALITH_ADDRESS);

        expect(frozen.unwrap().status.isFrozen).to.be.true;
      },
    });

    it({
      id: "T20",
      title: "allows to thaw account",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
          mints: [{ account: alith, amount: 100000000000000n }],
        }));

        await context.createBlock(
          await context
            .polkadotJs()
            .tx.localAssets.freeze(assetId, ALITH_ADDRESS)
            .signAsync(baltathar)
        );

        const rawTxn = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "thaw",
          args: [ALITH_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");

        const frozen = await context.polkadotJs().query.localAssets.account(assetId, ALITH_ADDRESS);
        expect(frozen.unwrap().status.isFrozen).toBe(false);
      },
    });

    it({
      id: "T21",
      title: "allows to freeze an asset",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "freeze_asset",
          args: [],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");

        const registeredAsset = (
          await context.polkadotJs().query.localAssets.asset(assetId)
        ).unwrap();

        expect(registeredAsset.status.isFrozen).to.be.true;
      },
    });

    it({
      id: "T22",
      title: "allows to thaw an asset",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        }));

        await context.createBlock(
          context.polkadotJs().tx.localAssets.freezeAsset(assetId).signAsync(baltathar)
        );

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "thaw_asset",
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const registeredAsset = (
          await context.polkadotJs().query.localAssets.asset(assetId)
        ).unwrap();

        expect(registeredAsset.status.isFrozen).to.be.false;
      },
    });

    it({
      id: "T23",
      title: "allows to transfer ownership",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "transfer_ownership",
          args: [ALITH_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const registeredAsset = (
          await context.polkadotJs().query.localAssets.asset(assetId)
        ).unwrap();
        expect(registeredAsset.owner.toHex()).to.eq(ALITH_ADDRESS.toLowerCase());
      },
    });

    it({
      id: "T24",
      title: "allows to set team",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "set_team",
          args: [ALITH_ADDRESS, ALITH_ADDRESS, ALITH_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const registeredAsset = (
          await context.polkadotJs().query.localAssets.asset(assetId)
        ).unwrap();

        expect(registeredAsset.admin.toHex()).to.eq(ALITH_ADDRESS.toLowerCase());
        expect(registeredAsset.freezer.toHex()).to.eq(ALITH_ADDRESS.toLowerCase());
        expect(registeredAsset.issuer.toHex()).to.eq(ALITH_ADDRESS.toLowerCase());
      },
    });

    it({
      id: "T25",
      title: "allows to set metadata",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "set_metadata",
          args: ["Local", "LOC", 12],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock(rawTx);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const metadata = await context.polkadotJs().query.localAssets.metadata(assetId);
        expect(u8aToString(metadata.name)).to.eq("Local");
        expect(u8aToString(metadata.symbol)).to.eq("LOC");
        expect(metadata.decimals.toString()).to.eq("12");
      },
    });

    it({
      id: "T26",
      title: "allows to clear metadata",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "clear_metadata",
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock(rawTx);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const metadata = await context.polkadotJs().query.localAssets.metadata(assetId);
        expect(metadata.name.isEmpty).toBe(true);
        expect(metadata.symbol.isEmpty).toBe(true);
        expect(metadata.decimals.toBigInt()).to.eq(0n);
      },
    });

    it({
      id: "T27",
      title: "succeeds to mint to 2^128 - 1",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
          mints: [{ account: alith, amount: 2n ** 128n - 3n }],
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "mint",
          args: [baltathar.address, 2],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
      },
    });

    it({
      id: "T28",
      title: "fails to mint over 2^128 total supply",
      test: async function () {
        ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
          registrerAccount: baltathar,
          mints: [{ account: alith, amount: 2n ** 128n - 3n }],
        }));

        const rawTx = await context.writeContract!({
          contractName: "LocalAssetExtendedErc20Instance",
          contractAddress: assetAddress as `0x${string}`,
          functionName: "mint",
          args: [baltathar.address, 3],
          rawTxOnly: true,
          gas: 100_000n,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.equal("reverted");

        expect(
          async () =>
            await context.writeContract!({
              contractName: "LocalAssetExtendedErc20Instance",
              contractAddress: assetAddress as `0x${string}`,
              functionName: "mint",
              args: [baltathar.address, 3],
              rawTxOnly: true,
              privateKey: BALTATHAR_PRIVATE_KEY,
            })
        ).rejects.toThrowError("Dispatched call failed with error: Arithmetic(Overflow)");
      },
    });
  },
});
