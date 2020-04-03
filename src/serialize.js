const {gzip, ungzip} = require('node-gzip');
const fs = require('fs');
const util = require('util');

const readFile = util.promisify(fs.readFile);
const writeFile = util.promisify(fs.writeFile);

readFile('dist/SpeedReaderConfig.json')
.then((config) => {
  return gzip(JSON.stringify(JSON.parse(config)))
})
.then((compressed) => {
  return writeFile("dist/SpeedReaderConfig.dat", compressed);
});
