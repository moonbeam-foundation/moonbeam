import { moonbeamDefinitions, types } from "moonbeam-types-bundle";

export default {
  types: {},
  rpc: {
    ...moonbeamDefinitions.rpc?.moon,
  },
};
