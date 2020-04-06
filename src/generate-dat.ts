import AJV from "ajv"
import fs from "fs"
import {resolve} from "path"
import * as TJS from "typescript-json-schema"
import util from 'util'
import {gzip} from 'node-gzip'
import { sys } from "typescript"

const configFile = "data/SpeedReaderConfig.json"
const outputFile = "data/speedreader-updater.dat"

const readFile = util.promisify(fs.readFile)
const writeFile = util.promisify(fs.writeFile)

const validate = (schema: TJS.Definition, data: object) => {
  const ajv = new AJV({
    allErrors: true,
    coerceTypes: 'array',
    removeAdditional: true,
    useDefaults: 'empty',
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
  // optionally pass argument to schema generator
  const settings: TJS.PartialArgs = {
      required: true,
      topRef: true,
  }

  // optionally pass ts compiler options
  const compilerOptions: TJS.CompilerOptions = {
      strictNullChecks: true
  }

  const program = TJS.getProgramFromFiles([resolve("src/types/SpeedReaderConfig.d.ts")], compilerOptions)
  const generator = TJS.buildGenerator(program, settings)
  const schema = generator.getSchemaForSymbol("SpeedReader.Configuration")
  return schema
}

readFile(configFile)
.then((config) => {
  return JSON.parse(config.toString('utf-8'))
})
.then((config) => {
  const validated = validate(getSchema(), config);
  if (!validated.valid) {
    throw TypeError("The configuration does not match expected format");
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

