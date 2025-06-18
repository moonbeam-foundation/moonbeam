import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.resolve(path.dirname(__filename), "..");

// Function to convert test results to CSV format
function convertTestsToCSV(runtime: string, failedSuites: any[]): string {
  let csvContent = "Runtime,Test Suite,Test Title\n";

  failedSuites.forEach((suite) => {
    const file = suite.name.replaceAll(__dirname, "");
    suite.assertionResults.forEach((test: { title: string }) => {
      // Clean and escape the test title if it contains commas
      const cleanTitle = test.title.trim();
      const escapedTitle = cleanTitle.includes(",") ? `"${cleanTitle}"` : cleanTitle;

      csvContent += `${runtime},${file},${escapedTitle}\n`;
    });
  });

  return csvContent;
}

["Moonbeam", "Moonriver", "Moonbase"].forEach((runtime) => {
  const outputPath = `${__dirname}/tmp/testResults${runtime}.json`;
  if (!fs.existsSync(outputPath)) {
    return;
  }

  const fileContent = fs.readFileSync(outputPath, { encoding: "utf-8" });
  const { testResults } = JSON.parse(fileContent);

  const failedSuites = testResults.filter((suite: { status: string }) => suite.status !== "passed");

  // Original console output
  const failedTests = failedSuites
    .flatMap((suite) => {
      const file = suite.name.replaceAll(__dirname, "");
      const fails = suite.assertionResults
        .map((test: { title: string }) => `\n\t\t> ${test.title.trim()}`)
        .join("");
      return `\t- ${file}${fails}`;
    })
    .join("\n");

  if (failedTests) {
    console.log(`Failed tests for ${runtime}:\n`, failedTests);

    // Generate and save CSV
    const csvContent = convertTestsToCSV(runtime, failedSuites);
    const csvPath = `${__dirname}/tmp/failedTests${runtime}.csv`;
    fs.writeFileSync(csvPath, csvContent);
    console.log(`\nCSV report generated at: ${csvPath}`);
  }
});
