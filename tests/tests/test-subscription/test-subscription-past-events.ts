import { expect } from "chai";
import { createContract } from "../../util/transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { web3Subscribe } from "../../util/providers";

describeDevMoonbeam("Subscription - Past Events", (context) => {
  let web3Ws;

  before("Setup: Create 4 blocks with transfer", async () => {
    web3Ws = await context.createWeb3("ws");

    const { rawTx: rawTx1 } = await createContract(context.web3, "SingleEventContract", {
      nonce: 0,
    });
    const { rawTx: rawTx2 } = await createContract(context.web3, "SingleEventContract", {
      nonce: 1,
    });
    const { rawTx: rawTx3 } = await createContract(context.web3, "SingleEventContract", {
      nonce: 2,
    });
    const { rawTx: rawTx4 } = await createContract(context.web3, "SingleEventContract", {
      nonce: 3,
    });

    await context.createBlock({
      transactions: [rawTx1, rawTx2, rawTx3, rawTx4],
    });
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
      address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
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
      address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
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
        "0xe573BCA813c741229ffB2488F7856C6cAa841041",
        "0xF8cef78E923919054037a1D03662bBD884fF4edf",
        "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
        "0x5c4242beB94dE30b922f57241f1D02f36e906915",
        "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
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
