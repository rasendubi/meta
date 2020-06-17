#!/usr/bin/env node
const fs = require("fs");

const ndjson = require("ndjson");

const readMeta = (filename, result = []) => {
  return new Promise((resolve, reject) => {
    fs.createReadStream(filename)
      .pipe(ndjson.parse({ strict: false }))
      .on("data", (d) => result.push(d))
      .on("end", () => resolve(result))
      .on("error", (err) => reject(err));
  });
};

const readAllMeta = (filenames, result = []) => {
  return Promise.all(filenames.map((arg) => readMeta(arg, result))).then(
    () => result
  );
};

class Meta {
  constructor(meta) {
    this._meta = meta;

    // define/find well-known elements
    this.identifierId = "0";
    this.valueTypeId = this.lookupByIdentifier("value-type")[0];
    this.StringId = this.lookupByIdentifier("String")[0];
    this.ReferenceId = this.lookupByIdentifier("Reference")[0];
  }

  lookup = (pattern) => {
    const match = (expected, actual) => {
      return expected === undefined || expected === null || expected === actual;
    };

    return this._meta.filter(
      (x) =>
        match(pattern[0], x[0]) &&
        match(pattern[1], x[1]) &&
        match(pattern[2], x[2])
    );
  };

  getAttributeValue = (id, attrId) => {
    return this.lookup([id, attrId]).map((x) => x[2]);
  };

  // Get all attributes of the element
  getAttributes = (id) => {
    return this._meta
      .filter((x) => x[0] == id)
      .map(([, attribute, value]) => [attribute, value]);
  };

  // Lookup identifier of the element
  getIdentifiers = (id) => {
    // "0" is a well-known "identifier" attribute
    return this.getAttributeValue(id, "0");
  };

  lookupByIdentifier = (identifier) => {
    return this.lookup([null, "0", identifier]).map((x) => x[0]);
  };

  // Get a set of all known elements
  getElements = () => {
    const result = new Set();
    this._meta.forEach(([id, ,]) => result.add(id));
    return result;
  };
}

const prettyValue = (meta, attrId, value) => {
  const attrType = meta.getAttributeValue(attrId, meta.valueTypeId)[0];
  if (attrType === meta.ReferenceId) {
    return prettyName(meta, value);
  } else if (attrType === meta.StringId) {
    return JSON.stringify(value);
  } else {
    // unknown blob
    return `[${JSON.stringify(value)}]`;
  }
};

const prettyName = (meta, id) => {
  const identifiers = meta.getIdentifiers(id);
  return `${identifiers[0]}(${id})`;
};

const prettyAttribute = (meta, id, value) => {
  return `${prettyName(meta, id)} = ${prettyValue(meta, id, value)}`;
};

// pretty-print element with given id
const pretty = (meta, id) => {
  const name = prettyName(meta, id);
  const attrs = meta.lookup([id]);

  console.log(name);
  attrs.forEach(([_id, attrId, value]) => {
    console.log(`  ${prettyAttribute(meta, attrId, value)}`);
  });
  console.log();
};

const main = async () => {
  const files = process.argv.slice(2);
  const meta = new Meta(await readAllMeta(files));

  meta.getElements().forEach((id) => pretty(meta, id));
};

main();
