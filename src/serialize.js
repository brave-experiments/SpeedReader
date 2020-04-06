const {gzip, ungzip} = require('node-gzip');
const fs = require('fs');
const util = require('util');

const readFile = util.promisify(fs.readFile);
const writeFile = util.promisify(fs.writeFile);

const input = "data/SpeedReaderConfig.json";
const output = "data/SpeedReaderConfig.dat";

readFile(input)
.then((config) => {
  return gzip(JSON.stringify(JSON.parse(config)))
})
.then((compressed) => {
  return writeFile(output, compressed);
})
.then(() => {
  console.log("Serialized")
});
