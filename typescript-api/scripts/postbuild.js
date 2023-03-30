import { writeFileSync, copyFileSync } from "fs";
import pck from "../package.json" assert { type: "json" };

const buildPath = `${process.env.PWD}/build`;

pck.scripts = {};
pck.private = false;
pck.type = "module";
pck.files = ["**/*", "!**/tsconfig.tsbuildinfo", "!**/*.tgz"];

writeFileSync(`${buildPath}/package.json`, JSON.stringify(pck, null, 2));
copyFileSync("README.md", `${buildPath}/README.md`);

// Copy empty files for CommonJS modules
copyFileSync("./src/index.cjs", `${buildPath}/index.cjs`);
copyFileSync("./src/index.cjs", `${buildPath}/moonriver/index.cjs`);
copyFileSync("./src/index.cjs", `${buildPath}/moonbase/index.cjs`);
