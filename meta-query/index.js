#!/usr/bin/env node
const fs = require("fs");
const readline = require("readline");

const parseArgs = require("minimist");
const ndjson = require("ndjson");

const args = parseArgs(process.argv.slice(2));

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
    this.identifierId = "1000";
    this.valueTypeId = this.lookupByIdentifier("attribute/value-type")[0];
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
        match(pattern[2], x[2]) &&
        match(pattern[3], x[3])
    );
  };

  getAttributeValue = (id, attrId) => {
    return this.lookup([id, attrId]).map((x) => x[2]);
  };

  // Get all attributes of the element
  getAttributes = (id) => {
    return this.lookup([id]).map(([, attribute, value]) => [attribute, value]);
  };

  // Lookup identifier of the element
  getIdentifiers = (id) => {
    return this.getAttributeValue(id, this.identifierId);
  };

  lookupByIdentifier = (identifier) => {
    return this.lookup([null, this.identifierId, identifier]).map((x) => x[0]);
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
    return `blob(${JSON.stringify(value)}])`;
  }
};

const prettyName = (meta, id) => {
  const identifiers = meta.getIdentifiers(id);
  return `${identifiers[0] || ""}(${id})`;
};

const prettyRow = (meta, id, value, rid) => {
  return `${prettyName(meta, id)} = ${prettyValue(meta, id, value)} {rowid=${rid}}`;
};

// pretty-print element with given id
const pretty = (meta, id) => {
  const name = prettyName(meta, id);
  const attrs = meta.lookup([id]);

  console.log(name);
  attrs.forEach(([_id, attrId, value, rid]) => {
    console.log(`  ${prettyRow(meta, attrId, value, rid)}`);
  });
  console.log();
};

const annotateFile = async (meta, file) => {
  const stream = fs.createReadStream(file);
  const rl = readline.createInterface({ input: stream });

  for await (const line of rl) {
    try {
      const [element, attribute, value, rid] = JSON.parse(line);
      try {
        console.log(
          `${line}  //=> ${prettyName(meta, element)}.${prettyName(
            meta,
            attribute
          )} = ${prettyValue(meta, attribute, value)} {rowid=${rid}}`
        );
      } catch (e) {
        console.log(`${line}  //=> ${e}`);
      }
    } catch {
      if (!args["only-meta"]) {
        console.log(line);
      }
    }
  }
};

const annotate = async (meta, files) => {
  for (const file of files) {
    await annotateFile(meta, file);
  }
};

const main = async () => {
  const metaFiles = args._;
  const meta = new Meta(await readAllMeta(metaFiles));

  const annotateFile = args["annotate-file"];
  if (annotateFile) {
    if (annotateFile === true) {
      annotate(meta, metaFiles);
    } else {
      const files = Array.isArray(annotateFile) ? annotateFile : [annotateFile];
      annotate(meta, files);
    }
  } else {
    meta.getElements().forEach((id) => pretty(meta, id));
  }
};

main();
