const { writeFileSync, copyFileSync } = require('fs');
const pck = require('../package.json');

pck.scripts = {};
pck.private = false;
pck.type = 'module';
pck.files = ['**/*', '!**/tsconfig.tsbuildinfo', '!**/*.tgz'];

writeFileSync(`${process.env.PWD}/build/package.json`, JSON.stringify(pck, null, 2));
copyFileSync('README.md', `${process.env.PWD}/build/README.md`);
