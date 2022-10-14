import "@moonbeam-network/api-augment/moonbase";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck"
import {ethers} from "ethers"
import { EXTRINSIC_GAS_LIMIT, EXTRINSIC_BASE_WEIGHT, WEIGHT_PER_GAS } from "../util/constants";

const debug = require("debug")("smoke:weight-general");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;
const ethUrl = process.env.ETH_URL || null;

interface BlockWeights {
  hash: string;
  weights: BlockLimits;
}

interface BlockLimits {
  normal: BN;
  operational: BN;
}

describeSmokeSuite(`Verify weights of blocks being produced`, { wssUrl, relayWssUrl }, (context) => {
  let blockLimits: BlockLimits;
  let blockWeights: [BlockWeights?] = [];
  const blocks: any[] = []

  before("Retrieve past hour's worth of blocks", async function () {
    const limiter = new Bottleneck({
        minTime: 100,
        maxConcurrent: 5
    })

    const result = await limiter.schedule(()=> context.polkadotApi.rpc.chain.getBlock("0xc2fa80d3f80e13cac860182fb91107c819b1fc4b2032ba74991866f242102fd2"))
    blocks.push(result)
  });


  it("should have a block weight to its transactions weight", async function () {
    console.log("Hello timbo")
    console.log(blocks[0].block.header.number.toNumber())
    console.log(blocks[0].block.extrinsics.toHuman())
    const exts = blocks[0].block.extrinsics.filter((item)=>{
        console.log(item.toHuman())
        return item.method.section.toString()== "ethereum"})
    // console.log((exts[0].toJSON()))
    console.log(exts)

    const magicBlockNumber = 2070542
    const subBlockHash = await context.polkadotApi.rpc.chain.getBlockHash(magicBlockNumber)
    const apiAt = await context.polkadotApi.at(subBlockHash)
    
    const blockWeight = await apiAt.query.system.blockWeight()
    console.log("weight of block: " + JSON.stringify(blockWeight))
    const provider = new ethers.providers.JsonRpcProvider(ethUrl)
    
    // console.log(await provider.getBlock(magicBlockNumber))
    const blockGas = (await provider.getBlock(magicBlockNumber)).gasUsed.toBigInt()
    console.log( "gas used by block: "+ Number(blockGas))
    console.log("gas into weight calc: "+ Number(blockGas * WEIGHT_PER_GAS))
  });

});
