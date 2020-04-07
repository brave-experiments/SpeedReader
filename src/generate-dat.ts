import AJV from "ajv"
import fs from "fs"
import {resolve} from "path"
import * as TJS from "typescript-json-schema"
import util from 'util'
import {gzip} from 'node-gzip'
import request from 'request'

const configURL = "https://raw.githubusercontent.com/brave-experiments/SpeedReader/master/"
const configFile = "data/SpeedReaderConfig.json"
const outputFile = "data/speedreader-updater.dat"

const readFile = util.promisify(fs.readFile)
const writeFile = util.promisify(fs.writeFile)

const validate = (schema: TJS.Definition, data: object) => {
  const ajv = new AJV({
    allErrors: true,
    nullable: true,
    coerceTypes: true,
    removeAdditional: true,
  })
  const valid = ajv.validate(schema, data)
  const errorText =
    ajv.errorsText() && ajv.errorsText().toLowerCase() !== "no errors"
      ? ajv.errorsText()
      : ""

  return {
    errorText,
    valid: !!valid
  }
}

const getSchema = () => {
  const settings: TJS.PartialArgs = {
      required: true,   // Include required fields for non-optional properties
      topRef: true,     // Create a top-level ref definition
  }

  const compilerOptions: TJS.CompilerOptions = {
      strictNullChecks: true    // Make values non-nullable by default
  }

  const program = TJS.getProgramFromFiles([resolve("src/types/SpeedReaderConfig.d.ts")], compilerOptions)
  const generator = TJS.buildGenerator(program, settings)
  const schema = generator.getSchemaForSymbol("SpeedReader.Configuration")
  return schema
}

const downloadConfig = (url: string, file: string) => {
  return new Promise<Array<Object>>(function(resolve, reject){
      request(url + file, function (err, response, body) {
          // in addition to parsing the value, deal with possible errors
          if (err) return reject(err);
          try {
              resolve(JSON.parse(body));
          } catch(e) {
              reject(e);
          }
      });
  });
}

downloadConfig(configURL, configFile)
.then((config) => {
  const validated = validate(getSchema(), config);
  if (!validated.valid) {
    throw TypeError("The configuration does not match expected format: " + validated.errorText);
  }
  return config
})
.then((config) => {
  return gzip(JSON.stringify(config))
})
.then((compressed: Buffer) => {
  return writeFile(outputFile, compressed)
})
.then(() => {
  console.log("Serialized")
})
.catch((error) => {
  console.log("Error: ", error);
  process.exit(1);
})

