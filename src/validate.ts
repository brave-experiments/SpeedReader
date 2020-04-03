import AJV from "ajv";
import fs from "fs";



export const validate = (schema: object, data: object) => {
  const ajv = new AJV({
    allErrors: true,
    coerceTypes: 'array',
    removeAdditional: true,
    useDefaults: 'empty',
  });
  const valid = ajv.validate(schema, data);
  const errorText =
    ajv.errorsText() && ajv.errorsText().toLowerCase() !== "no errors"
      ? ajv.errorsText()
      : "";

  return {
    errorText,
    valid: !!valid
  };
};

const configRaw = fs.readFileSync("dist/SpeedReaderConfig.json", 'utf8');
const configJson = JSON.parse(configRaw);

const schemaRaw = fs.readFileSync("dist/SpeedReaderConfig.schema.json", 'utf8');
const schema = JSON.parse(schemaRaw);

const validRes = validate(schema, configJson[0]);

if (validRes.valid) {
  console.log("Schema is alright.");
  process.exit(0);
} else {
  console.log(validRes.errorText);
  process.exit(1);
}
