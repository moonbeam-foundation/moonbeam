import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { createContract } from "../../util/transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { web3Subscribe } from "../../util/providers";
import { ALITH_CONTRACT_ADDRESSES } from "../../util/accounts";

describeDevMoonbeam("Subscription - Past Events", (context) => {
  let web3Ws;

  before("Setup: Create 4 blocks with transfer", async () => {
    web3Ws = await context.createWeb3("ws");

    const { rawTx: rawTx1 } = await createContract(context, "SingleEventContract", {
      nonce: 0,
    });
    const { rawTx: rawTx2 } = await createContract(context, "SingleEventContract", {
      nonce: 1,
    });
    const { rawTx: rawTx3 } = await createContract(context, "SingleEventContract", {
      nonce: 2,
    });
    const { rawTx: rawTx4 } = await createContract(context, "SingleEventContract", {
      nonce: 3,
    });

    await context.createBlock([rawTx1, rawTx2, rawTx3, rawTx4]);
  });

  it("should be retrieved by topic", async function () {
    const subscription = web3Subscribe(web3Ws, "logs", {
      fromBlock: "0x0",
      topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
    });

    const data = await new Promise((resolve) => {
      const data = [];
      subscription.on("data", function (d: any) {
        data.push(d);
        if (data.length == 4) resolve(data);
      });
    });
    subscription.unsubscribe();

    expect(data).to.not.be.empty;
  });

  it("should be retrieved by address", async function () {
    const subscription = web3Subscribe(web3Ws, "logs", {
      fromBlock: "0x0",
      address: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
    });

    const data = await new Promise((resolve) => {
      const data = [];
      subscription.on("data", function (d: any) {
        data.push(d);
        if (data.length == 1) resolve(data);
      });
    });
    subscription.unsubscribe();

    expect(data).to.not.be.empty;
  });

  it("should be retrieved by address + topic", async function () {
    const subscription = web3Subscribe(web3Ws, "logs", {
      fromBlock: "0x0",
      topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
      address: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
    });

    const data = await new Promise((resolve) => {
      const data = [];
      subscription.on("data", function (d: any) {
        data.push(d);
        if (data.length == 1) resolve(data);
      });
    });
    subscription.unsubscribe();

    expect(data).to.not.be.empty;
  });

  it("should be retrieved by multiple addresses", async function () {
    const subscription = web3Subscribe(web3Ws, "logs", {
      fromBlock: "0x0",
      topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
      address: [
        ALITH_CONTRACT_ADDRESSES[4],
        ALITH_CONTRACT_ADDRESSES[3],
        ALITH_CONTRACT_ADDRESSES[2],
        ALITH_CONTRACT_ADDRESSES[1],
        ALITH_CONTRACT_ADDRESSES[0],
      ],
    });

    const data = await new Promise((resolve) => {
      const data = [];
      subscription.on("data", function (d: any) {
        data.push(d);
        if (data.length == 4) resolve(data);
      });
    });
    subscription.unsubscribe();

    expect(data).to.not.be.empty;
  });
});
