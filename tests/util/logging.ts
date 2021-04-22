export function log(...msg: any[]) {
  if (process.argv && process.argv[2] && process.argv[2] === "--printlogs") {
    console.log(...msg);
  }
}
