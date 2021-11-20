import * as path from 'path';

import { safeLoad } from 'js-yaml';
import { readFile, writeFile, pathExists, mkdirp } from 'fs-extra';
import pluralize from './lib/pluralize';
import { toDashCase } from './lib/helpers';

type TransformType = 'json' | 'css' | 'scss' | 'ts';

interface Config {
  transforms: TransformType[];
}

interface TokenObject {
  [key: string]: string | string[] | TokenObject;
}
type Prefix = string | number;
type PrefixList = Prefix[];
type SourceFileType = 'yaml' | 'json';
type FlatToken = [PrefixList, string];
type FlatTokenList = FlatToken[];
type TransformPrefix = (str: Prefix) => string;
type TransformTokenObject = (json: TokenObject) => [string, string];

const config: Config = {
  transforms: ['json', 'css', 'scss', 'ts'],
};

export const run = async (filePath: string, outDir: string) => {
  const fileExists = await pathExists(filePath);

  if (!fileExists) {
    throw new Error(`File does not exist: ${filePath}`);
  }

  await mkdirp(outDir);

  let fileType = path.extname(filePath).substring(1);

  if (['json', 'yml', 'yaml'].indexOf(fileType) === -1) {
    throw new Error(`'${fileType}' is not a supported extension type.  Please us a yaml or json file.`);
  }
  if (fileType === 'yml') {
    fileType = 'yaml';
  }

  await transformFile(filePath, fileType as SourceFileType, outDir);
};

const transformFile = async (filePath: string, fileType: SourceFileType, outDir: string) => {
  const tokenContents = await readFile(filePath, {
    encoding: 'utf8',
  });

  let tmpval: any;
  let tokenJSON: TokenObject;

  if (fileType === 'yaml') {
    tmpval = safeLoad(tokenContents);

  } else if (fileType === 'json') {
    tmpval = JSON.parse(tokenContents);
  }

    if (isTokenObject(tmpval)) {
      tokenJSON = tmpval as TokenObject;
    } else {
      throw new Error(`File contents are not structured in a way that would allow for token conversion`);
    }


  const transformsToExecute = config.transforms.map((t) => {
    switch (t) {
      case 'json':
        return transformToJSON;
      case 'ts':
        return transformToTypescript;
      case 'css':
        return transformToCSSVariables;
      case 'scss':
        return transformToSCSSVariables;
    }
  });
  await Promise.all(
    transformsToExecute.map((tokenTransform) => {
      const [fileExtension, output] = tokenTransform(tokenJSON);
      const outputFilePath = path.join(outDir, `${path.basename(filePath, path.extname(filePath))}.${fileExtension}`);
      writeFile(outputFilePath, output);
    })
  );
};

/**
 * Take a TokenObject and return a string of formatted JSON for output and a filename
 */
const transformToJSON: TransformTokenObject = (tokenJSON: TokenObject) => {
  const output = JSON.stringify(tokenJSON, null, 2);

  return ['json', output];
};

/**
 * Take a TokenObject and return a string of TS for output to a file
 */
const transformToTypescript: TransformTokenObject = (tokenJSON: TokenObject) => {
  const json = JSON.stringify(tokenJSON, null, 2);
  const output = `
export const themeData = ${json} as const;
export type ThemeType = typeof themeData;
`;

  return ['ts', output];
};

/**
 * Take a TokenObject and return a string of formatted CSS for output and a filename
 */
const transformToCSSVariables: TransformTokenObject = (tokenJSON: TokenObject) => {
  const transformKeyName: TransformPrefix = (keyName: Prefix) => {
    if (typeof keyName === 'number') {
      return `${keyName}`;
    }
    return toDashCase(pluralize.singular(keyName));
  };

  const flatTokenList = convertToFlatList(tokenJSON, [] as FlatTokenList);
  let output = flatTokenList
    .map(([prefixList, value]) => {
      const cssVarName = prefixList.map(transformKeyName).join('-');
      return `  --${cssVarName}: ${value};`;
    })
    .join(`\n`);

  output = `
:root {
${output}
}
`;

  return ['css', output];
};

/**
 * Take a TokenObject and return a string of formatted SCSS for output and a filename
 */
const transformToSCSSVariables: TransformTokenObject = (tokenJSON: TokenObject) => {
  const transformKeyName: TransformPrefix = (keyName: Prefix) => {
    if (typeof keyName === 'number') {
      return `${keyName}`;
    }
    return toDashCase(pluralize.singular(keyName));
  };

  const flatTokenList = convertToFlatList(tokenJSON, [] as FlatTokenList);
  const output = flatTokenList
    .map(([prefixList, value]) => {
      const cssVarName = prefixList.map(transformKeyName).join('-');
      return `$${cssVarName}: ${value};`;
    })
    .join(`\n`);

  return ['scss', output];
};

function convertToFlatList(value: TokenObject, valueList: FlatTokenList = [], prefixes: PrefixList = []) {
  if (!value) {
    return valueList;
  }

  let newTokenValues: FlatTokenList = [];

  if (Array.isArray(value)) {
    newTokenValues = value.reduce((allNewTokens, value, index) => {
      return [...allNewTokens, [prefixes.concat(index + 1), value]];
    }, [] as FlatTokenList);
  } else if (typeof value === 'object') {
    newTokenValues = Object.keys(value).reduce((allNewTokens, key) => {
      return [
        ...allNewTokens,
        ...convertToFlatList(value[key] as TokenObject, valueList, prefixes.concat(key)),
      ] as FlatTokenList;
    }, [] as FlatTokenList);
  } else {
    newTokenValues = [[prefixes, value]];
  }

  return [...valueList, ...newTokenValues];
}

function isTokenObject(value: any): value is object {
  return value != null && typeof value === 'object';
}