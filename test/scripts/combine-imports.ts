// Description: Combines imports from the same file into a single import statement,
//             useful for old test cases that had multiple imports from helpers in the same file.
// Usage: pnpm tsx ./scripts/combine-imports.ts <directory>

import fs from "node:fs/promises";
import { join } from "node:path";

const processFile = async (filePath: string): Promise<void> => {
  const content = await fs.readFile(filePath, "utf-8");

  const importRegex = /import \{ ([^\}]+) \} from "\.\.\/\.\.\/\.\.\/helpers";/g;

  const allImports: string[] = [];
  let firstMatchIndex = -1;

  let match = importRegex.exec(content);
  while (match !== null) {
    if (firstMatchIndex === -1) firstMatchIndex = match.index;
    allImports.push(match[1].trim());
    match = importRegex.exec(content);
  }

  if (allImports.length > 0) {
    const combinedImport = `import { ${allImports.join(", ")} } from "../../../../helpers";\n`;

    let updatedContent =
      content.slice(0, firstMatchIndex) +
      combinedImport +
      content.slice(firstMatchIndex).replace(importRegex, "");

    updatedContent = updatedContent.replace(/\n{3,}/g, "\n\n");

    await fs.writeFile(filePath, updatedContent);
    console.log("Imports combined for:", filePath);
  }
};

const processDirectory = async (dirPath: string): Promise<void> => {
  const entries = await fs.readdir(dirPath, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(dirPath, entry.name);

    if (entry.isDirectory()) {
      await processDirectory(fullPath);
    } else if (entry.name.endsWith(".ts")) {
      await processFile(fullPath);
    }
  }
};

const targetDirectory = process.argv[2];
if (!targetDirectory) {
  console.error("Please provide a directory path as an argument.");
  process.exit(1);
}

processDirectory(targetDirectory).catch((error) => {
  console.error("Error processing files:", error);
});
