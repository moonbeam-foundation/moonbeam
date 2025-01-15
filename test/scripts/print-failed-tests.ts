import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.resolve(path.dirname(__filename), "..");

["Moonbeam", "Moonriver", "Moonbase"].forEach((runtime) => {
  const outputPath = `${__dirname}/tmp/testResults${runtime}.json`;
  if (!fs.existsSync(outputPath)) {
    return;
  }

  const fileContent = fs.readFileSync(outputPath, { encoding: "utf-8" });
  const { testResults } = JSON.parse(fileContent);

  const failedSuites = testResults.filter((suite) => suite.status !== "passed");
  const failedTests = failedSuites
    .flatMap((suite) => {
      const file = suite.name.replaceAll(__dirname, "");
      const fails = suite.assertionResults.map((test) => `\n\t\t> ${test.title.trim()}`).join("");
      return `\t- ${file}${fails}`;
    })
    .join("\n");

  if (failedTests) {
    console.log(`Failed tests for ${runtime}:\n`, failedTests);
  }
});
