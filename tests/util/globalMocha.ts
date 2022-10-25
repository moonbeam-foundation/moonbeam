import { closeApi } from "./apis";
import { MochaGlobals } from "mocha";

export function mochaGlobalTeardown() {
  closeApi("parachain");
  closeApi("relay");
  closeApi("ethers");
}
