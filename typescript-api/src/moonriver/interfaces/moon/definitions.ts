import { moonbeamDefinitions, types } from "@moonbeam-network/types-bundle";

export default {
  types: {},
  rpc: {
    ...moonbeamDefinitions.rpc?.moon
  }
};
