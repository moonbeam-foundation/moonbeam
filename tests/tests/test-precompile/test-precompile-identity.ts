import "@moonbeam-network/api-augment";
import Contract from "web3-eth-contract";
import { expect } from "chai";
import { ethers } from "ethers";
import {
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  alith,
  baltathar,
  charleth,
} from "../../util/accounts";
import { PRECOMPILE_IDENTITY_ADDRESS } from "../../util/constants";

import { getCompiled } from "../../util/contracts";

import { expectOk, expectSubstrateEvent } from "../../util/expect";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContractExecution } from "../../util/transactions";
import { expectEVMResult } from "../../util/eth-transactions";
import { web3EthCall } from "../../util/providers";
import { Option } from "@polkadot/types";
import { stringToHex, u8aToHex } from "@polkadot/util";

const IDENTITY_CONTRACT = getCompiled("precompiles/identity/Identity");
const IDENTITY_INTERFACE = new ethers.utils.Interface(IDENTITY_CONTRACT.contract.abi);

const identityContract = new Contract(IDENTITY_CONTRACT.contract.abi, PRECOMPILE_IDENTITY_ADDRESS);

describeDevMoonbeam("Precompiles - Identity precompile - identity", (context) => {
  before("set identity on baltathar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            additional: [[{ raw: "discord" }, { raw: "my-discord" }]],
            display: { raw: "display" },
            legal: { raw: "legal" },
            web: { raw: "web" },
            riot: { raw: "riot" },
            email: { raw: "email" },
            pgpFingerprint: new Option(
              context.polkadotApi.registry,
              "[u8;20]",
              new Array(20).fill(1)
            ),
            image: { raw: "image" },
            twitter: { raw: "twitter" },
          })
          .signAsync(baltathar)
      )
    );
  });

  it("should retrieve identity", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("identity", [baltathar.address]),
    });

    const identity = IDENTITY_INTERFACE.decodeFunctionResult("identity(address)", result)[0];
    expect(identity.isValid).to.be.true;
    expect(identity.judgements).to.be.empty;
    expect(identity.deposit.toBigInt()).to.equal(1032400000000000000n);
    expect(identity.info.additional.length).to.equal(1);
    expect(identity.info.additional[0].key.hasData).to.be.true;
    expect(identity.info.additional[0].key.value).to.equal(stringToHex("discord"));
    expect(identity.info.additional[0].value.hasData).to.be.true;
    expect(identity.info.additional[0].value.value).to.equal(stringToHex("my-discord"));
    expect(identity.info.display.hasData).to.be.true;
    expect(identity.info.display.value).to.equal(stringToHex("display"));
    expect(identity.info.legal.hasData).to.be.true;
    expect(identity.info.legal.value).to.equal(stringToHex("legal"));
    expect(identity.info.web.hasData).to.be.true;
    expect(identity.info.web.value).to.equal(stringToHex("web"));
    expect(identity.info.riot.hasData).to.be.true;
    expect(identity.info.riot.value).to.equal(stringToHex("riot"));
    expect(identity.info.email.hasData).to.be.true;
    expect(identity.info.email.value).to.equal(stringToHex("email"));
    expect(identity.info.hasPgpFingerprint).to.be.true;
    expect(identity.info.pgpFingerprint).to.equal(u8aToHex(Uint8Array.from(new Array(20).fill(1))));
    expect(identity.info.image.hasData).to.be.true;
    expect(identity.info.image.value).to.equal(stringToHex("image"));
    expect(identity.info.twitter.hasData).to.be.true;
    expect(identity.info.twitter.value).to.equal(stringToHex("twitter"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - registrars", (context) => {
  before("add alith as registrar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );

    await expectOk(context.createBlock(context.polkadotApi.tx.identity.setFee(0, 100)));
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity.setFields(0, new Set(["Display", "Web"]) as any)
      )
    );

    // verify substrate storage
    const registrars = await context.polkadotApi.query.identity.registrars();
    expect(registrars.length).to.equal(1);
    expect(registrars[0].unwrap().account.toString()).to.equal(alith.address);
  });

  it("should retrieve registrars", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("registrars"),
    });

    const registrars = IDENTITY_INTERFACE.decodeFunctionResult("registrars()", result)[0];
    expect(registrars.length).to.equal(1);
    expect(registrars[0].isValid).to.be.true;
    expect(registrars[0].index).to.equal(0);
    expect(registrars[0].account).to.equal(alith.address);
    expect(registrars[0].fee.toBigInt()).to.equal(100n);
    expect(registrars[0].fields.display).to.be.true;
    expect(registrars[0].fields.web).to.be.true;
    expect(registrars[0].fields.legal).to.be.false;
    expect(registrars[0].fields.riot).to.be.false;
    expect(registrars[0].fields.email).to.be.false;
    expect(registrars[0].fields.pgpFingerprint).to.be.false;
    expect(registrars[0].fields.image).to.be.false;
    expect(registrars[0].fields.twitter).to.be.false;
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - set identity", (context) => {
  before("set identity on baltathar", async function () {
    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.setIdentity({
            additional: [
              {
                key: { hasData: true, value: stringToHex("discord") },
                value: { hasData: true, value: stringToHex("my-discord") },
              },
            ],
            display: { hasData: true, value: stringToHex("display") },
            legal: { hasData: true, value: stringToHex("legal") },
            web: { hasData: true, value: stringToHex("web") },
            riot: { hasData: true, value: stringToHex("riot") },
            email: { hasData: true, value: stringToHex("email") },
            hasPgpFingerprint: true,
            pgpFingerprint: Uint8Array.from(new Array(20).fill(1)),
            image: { hasData: true, value: stringToHex("image") },
            twitter: { hasData: true, value: stringToHex("twitter") },
          }),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("IdentitySet");
    expect(evmLog.args.who).to.equal(baltathar.address);
  });

  it("should retrieve identity", async function () {
    const identitySubstrate = (
      await context.polkadotApi.query.identity.identityOf(baltathar.address)
    ).unwrap();
    expect(identitySubstrate.judgements).to.be.empty;
    expect(identitySubstrate.deposit.toBigInt()).to.equal(1032400000000000000n);
    expect(identitySubstrate.info.toJSON()).to.deep.equal({
      additional: [[{ raw: stringToHex("discord") }, { raw: stringToHex("my-discord") }]],
      display: { raw: stringToHex("display") },
      legal: { raw: stringToHex("legal") },
      web: { raw: stringToHex("web") },
      riot: { raw: stringToHex("riot") },
      email: { raw: stringToHex("email") },
      pgpFingerprint: u8aToHex(Uint8Array.from(new Array(20).fill(1))),
      image: { raw: stringToHex("image") },
      twitter: { raw: stringToHex("twitter") },
    });

    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("identity", [baltathar.address]),
    });
    const identity = IDENTITY_INTERFACE.decodeFunctionResult("identity(address)", result)[0];
    expect(identity.isValid).to.be.true;
    expect(identity.judgements).to.be.empty;
    expect(identity.deposit.toBigInt()).to.equal(1032400000000000000n);
    expect(identity.info.additional.length).to.equal(1);
    expect(identity.info.additional[0].key.hasData).to.be.true;
    expect(identity.info.additional[0].key.value).to.equal(stringToHex("discord"));
    expect(identity.info.additional[0].value.hasData).to.be.true;
    expect(identity.info.additional[0].value.value).to.equal(stringToHex("my-discord"));
    expect(identity.info.display.hasData).to.be.true;
    expect(identity.info.display.value).to.equal(stringToHex("display"));
    expect(identity.info.legal.hasData).to.be.true;
    expect(identity.info.legal.value).to.equal(stringToHex("legal"));
    expect(identity.info.web.hasData).to.be.true;
    expect(identity.info.web.value).to.equal(stringToHex("web"));
    expect(identity.info.riot.hasData).to.be.true;
    expect(identity.info.riot.value).to.equal(stringToHex("riot"));
    expect(identity.info.email.hasData).to.be.true;
    expect(identity.info.email.value).to.equal(stringToHex("email"));
    expect(identity.info.hasPgpFingerprint).to.be.true;
    expect(identity.info.pgpFingerprint).to.equal(u8aToHex(Uint8Array.from(new Array(20).fill(1))));
    expect(identity.info.image.hasData).to.be.true;
    expect(identity.info.image.value).to.equal(stringToHex("image"));
    expect(identity.info.twitter.hasData).to.be.true;
    expect(identity.info.twitter.value).to.equal(stringToHex("twitter"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - clear identity", (context) => {
  before("set identity for baltathar then clear", async function () {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar),
      ])
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.clearIdentity(),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("IdentityCleared");
    expect(evmLog.args.who).to.equal(baltathar.address);
  });

  it("should have no identity", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("identity", [baltathar.address]),
    });
    const identity = IDENTITY_INTERFACE.decodeFunctionResult("identity(address)", result)[0];
    expect(identity.isValid).to.be.false;
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - request judgement", (context) => {
  before("add alith as registrar and request judgement", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.identity.setFee(0, 100n),
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar),
      ])
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.requestJudgement(0, 1000),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("JudgementRequested");
    expect(evmLog.args.who).to.equal(baltathar.address);
    expect(evmLog.args.registrar_index).to.equal(0);
  });

  it("should retrieve requested judgement as part of identity", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("identity", [baltathar.address]),
    });
    const identity = IDENTITY_INTERFACE.decodeFunctionResult("identity(address)", result)[0];
    expect(identity.isValid).to.be.true;
    expect(identity.judgements).to.have.length(1);
    expect(identity.judgements[0].registrar_index).to.equal(0);
    expect(identity.judgements[0].judgement.isFeePaid).to.be.true;
    expect(identity.judgements[0].judgement.feePaidDeposit.toBigInt()).to.equal(100n);
    expect(identity.deposit.toBigInt()).to.equal(1025800000000000000n);
    expect(identity.info.display.hasData).to.be.true;
    expect(identity.info.display.value).to.equal(stringToHex("display"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - cancel requested judgement", (context) => {
  before("add alith as registrar and cancel requested judgement", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity.requestJudgement(0, 1000n).signAsync(baltathar)
      )
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.cancelRequest(0),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("JudgementUnrequested");
    expect(evmLog.args.who).to.equal(baltathar.address);
    expect(evmLog.args.registrar_index).to.equal(0);
  });

  it("should have no requested judgement as part of identity", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("identity", [baltathar.address]),
    });
    const identity = IDENTITY_INTERFACE.decodeFunctionResult("identity(address)", result)[0];
    expect(identity.isValid).to.be.true;
    expect(identity.judgements).to.be.empty;
    expect(identity.deposit.toBigInt()).to.equal(1025800000000000000n);
    expect(identity.info.display.hasData).to.be.true;
    expect(identity.info.display.value).to.equal(stringToHex("display"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - provide judgement", (context) => {
  before("add alith as registrar and provide judgement", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );
    const identityData = {
      display: { raw: "display" },
    };
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity.setIdentity(identityData).signAsync(baltathar)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity.requestJudgement(0, 1000n).signAsync(baltathar)
      )
    );

    const identityHash = context.polkadotApi.registry
      .createType("PalletIdentityIdentityInfo", identityData)
      .hash.toHex();
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: identityContract,
        contractCall: identityContract.methods.provideJudgement(
          0,
          baltathar.address,
          {
            isUnknown: false,
            isFeePaid: false,
            feePaidDeposit: 0,
            isReasonable: false,
            isKnownGood: true,
            isOutOfData: false,
            isLowQuality: false,
            isErroneous: false,
          },
          identityHash
        ),
      })
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("JudgementGiven");
    expect(evmLog.args.target).to.equal(baltathar.address);
    expect(evmLog.args.registrar_index).to.equal(0);
  });

  it("should have provided judgement as part of identity", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("identity", [baltathar.address]),
    });
    const identity = IDENTITY_INTERFACE.decodeFunctionResult("identity(address)", result)[0];
    expect(identity.isValid).to.be.true;
    expect(identity.judgements).to.have.length(1);
    expect(identity.judgements[0].judgement.isKnownGood).to.be.true;
    expect(identity.deposit.toBigInt()).to.equal(1025800000000000000n);
    expect(identity.info.display.hasData).to.be.true;
    expect(identity.info.display.value).to.equal(stringToHex("display"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - set subs", (context) => {
  before("set subs for baltathar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar)
      )
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.setSubs([
            {
              account: charleth.address,
              data: { hasData: true, value: stringToHex("test") },
            },
          ]),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
  });

  it("should retrieve subs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("subsOf", [baltathar.address]),
    });
    const subs = IDENTITY_INTERFACE.decodeFunctionResult("subsOf(address)", result)[0];
    expect(subs.deposit.toBigInt()).to.be.equal(1005300000000000000n);
    expect(subs.accounts).to.have.length(1);
    expect(subs.accounts[0]).to.be.equal(charleth.address);
  });

  it("should retrieve super", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("superOf", [charleth.address]),
    });
    const superOf = IDENTITY_INTERFACE.decodeFunctionResult("superOf(address)", result)[0];
    expect(superOf.isValid).to.be.true;
    expect(superOf.account).to.be.equal(baltathar.address);
    expect(superOf.data.hasData).to.be.true;
    expect(superOf.data.value).to.be.equal(stringToHex("test"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - set fee", (context) => {
  before("add alith as registrar and set fee", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );

    const block = await context.createBlock(
      createContractExecution(context, {
        contract: identityContract,
        contractCall: identityContract.methods.setFee(0, 100),
      })
    );

    expectEVMResult(block.result.events, "Succeed");
  });

  it("should retrieve the registrar", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("registrars"),
    });

    const registrars = IDENTITY_INTERFACE.decodeFunctionResult("registrars()", result)[0];
    expect(registrars.length).to.equal(1);
    expect(registrars[0].isValid).to.be.true;
    expect(registrars[0].index).to.equal(0);
    expect(registrars[0].account).to.equal(alith.address);
    expect(registrars[0].fee.toBigInt()).to.equal(100n);
    expect(registrars[0].fields.display).to.be.false;
    expect(registrars[0].fields.web).to.be.false;
    expect(registrars[0].fields.legal).to.be.false;
    expect(registrars[0].fields.riot).to.be.false;
    expect(registrars[0].fields.email).to.be.false;
    expect(registrars[0].fields.pgpFingerprint).to.be.false;
    expect(registrars[0].fields.image).to.be.false;
    expect(registrars[0].fields.twitter).to.be.false;
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - set fields", (context) => {
  before("add alith as registrar and set fee", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );

    const block = await context.createBlock(
      createContractExecution(context, {
        contract: identityContract,
        contractCall: identityContract.methods.setFields(0, {
          display: true,
          web: true,
          legal: true,
          riot: true,
          email: true,
          pgpFingerprint: true,
          image: true,
          twitter: true,
        }),
      })
    );

    expectEVMResult(block.result.events, "Succeed");
  });

  it("should retrieve the registrar", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("registrars"),
    });

    const registrars = IDENTITY_INTERFACE.decodeFunctionResult("registrars()", result)[0];
    expect(registrars.length).to.equal(1);
    expect(registrars[0].isValid).to.be.true;
    expect(registrars[0].index).to.equal(0);
    expect(registrars[0].account).to.equal(alith.address);
    expect(registrars[0].fee.toBigInt()).to.equal(0n);
    expect(registrars[0].fields.display).to.be.true;
    expect(registrars[0].fields.web).to.be.true;
    expect(registrars[0].fields.legal).to.be.true;
    expect(registrars[0].fields.riot).to.be.true;
    expect(registrars[0].fields.email).to.be.true;
    expect(registrars[0].fields.pgpFingerprint).to.be.true;
    expect(registrars[0].fields.image).to.be.true;
    expect(registrars[0].fields.twitter).to.be.true;
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - set account id", (context) => {
  before("add alith as registrar and set fee", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.identity.addRegistrar(alith.address)
        )
      )
    );

    const block = await context.createBlock(
      createContractExecution(context, {
        contract: identityContract,
        contractCall: identityContract.methods.setAccountId(0, charleth.address),
      })
    );

    expectEVMResult(block.result.events, "Succeed");
  });

  it("should retrieve the registrar", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("registrars"),
    });

    const registrars = IDENTITY_INTERFACE.decodeFunctionResult("registrars()", result)[0];
    expect(registrars.length).to.equal(1);
    expect(registrars[0].isValid).to.be.true;
    expect(registrars[0].index).to.equal(0);
    expect(registrars[0].account).to.equal(charleth.address);
    expect(registrars[0].fee.toBigInt()).to.equal(0n);
    expect(registrars[0].fields.display).to.be.false;
    expect(registrars[0].fields.web).to.be.false;
    expect(registrars[0].fields.legal).to.be.false;
    expect(registrars[0].fields.riot).to.be.false;
    expect(registrars[0].fields.email).to.be.false;
    expect(registrars[0].fields.pgpFingerprint).to.be.false;
    expect(registrars[0].fields.image).to.be.false;
    expect(registrars[0].fields.twitter).to.be.false;
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - add sub", (context) => {
  before("add sub for baltathar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar)
      )
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.addSub(charleth.address, {
            hasData: true,
            value: stringToHex("test"),
          }),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("SubIdentityAdded");
    expect(evmLog.args.sub).to.equal(charleth.address);
    expect(evmLog.args.main).to.equal(baltathar.address);
  });

  it("should retrieve subs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("subsOf", [baltathar.address]),
    });
    const subs = IDENTITY_INTERFACE.decodeFunctionResult("subsOf(address)", result)[0];
    expect(subs.deposit.toBigInt()).to.be.equal(1005300000000000000n);
    expect(subs.accounts).to.have.length(1);
    expect(subs.accounts[0]).to.be.equal(charleth.address);
  });

  it("should retrieve super", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("superOf", [charleth.address]),
    });
    const superOf = IDENTITY_INTERFACE.decodeFunctionResult("superOf(address)", result)[0];
    expect(superOf.isValid).to.be.true;
    expect(superOf.account).to.be.equal(baltathar.address);
    expect(superOf.data.hasData).to.be.true;
    expect(superOf.data.value).to.be.equal(stringToHex("test"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - rename sub", (context) => {
  before("add sub for baltathar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .addSub(charleth.address, { Raw: "test" })
          .signAsync(baltathar)
      )
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.renameSub(charleth.address, {
            hasData: true,
            value: stringToHex("foobar"),
          }),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
  });

  it("should retrieve subs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("subsOf", [baltathar.address]),
    });
    const subs = IDENTITY_INTERFACE.decodeFunctionResult("subsOf(address)", result)[0];
    expect(subs.deposit.toBigInt()).to.be.equal(1005300000000000000n);
    expect(subs.accounts).to.have.length(1);
    expect(subs.accounts[0]).to.be.equal(charleth.address);
  });

  it("should retrieve super", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("superOf", [charleth.address]),
    });
    const superOf = IDENTITY_INTERFACE.decodeFunctionResult("superOf(address)", result)[0];
    expect(superOf.isValid).to.be.true;
    expect(superOf.account).to.be.equal(baltathar.address);
    expect(superOf.data.hasData).to.be.true;
    expect(superOf.data.value).to.be.equal(stringToHex("foobar"));
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - remove sub", (context) => {
  before("add sub for baltathar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .addSub(charleth.address, { Raw: "test" })
          .signAsync(baltathar)
      )
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.removeSub(charleth.address),
        },
        {
          from: baltathar.address,
          privateKey: BALTATHAR_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("SubIdentityRemoved");
    expect(evmLog.args.sub).to.equal(charleth.address);
    expect(evmLog.args.main).to.equal(baltathar.address);
  });

  it("should have no subs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("subsOf", [baltathar.address]),
    });
    const subs = IDENTITY_INTERFACE.decodeFunctionResult("subsOf(address)", result)[0];
    expect(subs.deposit.toBigInt()).to.be.equal(0n);
    expect(subs.accounts).to.be.empty;
  });
});

describeDevMoonbeam("Precompiles - Identity precompile - quit sub", (context) => {
  before("add sub for baltathar", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .setIdentity({
            display: { raw: "display" },
          })
          .signAsync(baltathar)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.identity
          .addSub(charleth.address, { Raw: "test" })
          .signAsync(baltathar)
      )
    );

    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: identityContract,
          contractCall: identityContract.methods.quitSub(),
        },
        {
          from: charleth.address,
          privateKey: CHARLETH_PRIVATE_KEY,
        }
      )
    );

    expectEVMResult(block.result.events, "Succeed");
    const { data } = expectSubstrateEvent(block, "evm", "Log");
    const evmLog = IDENTITY_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name).to.equal("SubIdentityRevoked");
    expect(evmLog.args.sub).to.equal(charleth.address);
  });

  it("should have no subs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_IDENTITY_ADDRESS,
      data: IDENTITY_INTERFACE.encodeFunctionData("subsOf", [baltathar.address]),
    });
    const subs = IDENTITY_INTERFACE.decodeFunctionResult("subsOf(address)", result)[0];
    expect(subs.deposit.toBigInt()).to.be.equal(0n);
    expect(subs.accounts).to.be.empty;
  });
});
