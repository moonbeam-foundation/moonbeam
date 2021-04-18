export default {
  files: ["./ava-tests/**/*.ts"],
  require: ["ts-node/register", "./util/setup.ts", "./util/node.ts"],
  extensions: ["ts"],
};
