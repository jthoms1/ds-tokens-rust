"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = void 0;
const path = __importStar(require("path"));
const js_yaml_1 = require("js-yaml");
const fs_extra_1 = require("fs-extra");
const pluralize_1 = __importDefault(require("./lib/pluralize"));
const helpers_1 = require("./lib/helpers");
const config = {
    transforms: ['json', 'css', 'scss', 'ts'],
};
exports.run = async (filePath, outDir) => {
    const fileExists = await fs_extra_1.pathExists(filePath);
    if (!fileExists) {
        throw new Error(`File does not exist: ${filePath}`);
    }
    await fs_extra_1.mkdirp(outDir);
    let fileType = path.extname(filePath).substring(1);
    if (['json', 'yml', 'yaml'].indexOf(fileType) === -1) {
        throw new Error(`'${fileType}' is not a supported extension type.  Please us a yaml or json file.`);
    }
    if (fileType === 'yml') {
        fileType = 'yaml';
    }
    await transformFile(filePath, fileType, outDir);
};
const transformFile = async (filePath, fileType, outDir) => {
    const tokenContents = await fs_extra_1.readFile(filePath, {
        encoding: 'utf8',
    });
    let tmpval;
    let tokenJSON;
    if (fileType === 'yaml') {
        tmpval = js_yaml_1.safeLoad(tokenContents);
    }
    else if (fileType === 'json') {
        tmpval = JSON.parse(tokenContents);
    }
    if (isTokenObject(tmpval)) {
        tokenJSON = tmpval;
    }
    else {
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
    await Promise.all(transformsToExecute.map((tokenTransform) => {
        const [fileExtension, output] = tokenTransform(tokenJSON);
        const outputFilePath = path.join(outDir, `${path.basename(filePath, path.extname(filePath))}.${fileExtension}`);
        fs_extra_1.writeFile(outputFilePath, output);
    }));
};
/**
 * Take a TokenObject and return a string of formatted JSON for output and a filename
 */
const transformToJSON = (tokenJSON) => {
    const output = JSON.stringify(tokenJSON, null, 2);
    return ['json', output];
};
/**
 * Take a TokenObject and return a string of TS for output to a file
 */
const transformToTypescript = (tokenJSON) => {
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
const transformToCSSVariables = (tokenJSON) => {
    const transformKeyName = (keyName) => {
        if (typeof keyName === 'number') {
            return `${keyName}`;
        }
        return helpers_1.toDashCase(pluralize_1.default.singular(keyName));
    };
    const flatTokenList = convertToFlatList(tokenJSON, []);
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
const transformToSCSSVariables = (tokenJSON) => {
    const transformKeyName = (keyName) => {
        if (typeof keyName === 'number') {
            return `${keyName}`;
        }
        return helpers_1.toDashCase(pluralize_1.default.singular(keyName));
    };
    const flatTokenList = convertToFlatList(tokenJSON, []);
    const output = flatTokenList
        .map(([prefixList, value]) => {
        const cssVarName = prefixList.map(transformKeyName).join('-');
        return `$${cssVarName}: ${value};`;
    })
        .join(`\n`);
    return ['scss', output];
};
function convertToFlatList(value, valueList = [], prefixes = []) {
    if (!value) {
        return valueList;
    }
    let newTokenValues = [];
    if (Array.isArray(value)) {
        newTokenValues = value.reduce((allNewTokens, value, index) => {
            return [...allNewTokens, [prefixes.concat(index + 1), value]];
        }, []);
    }
    else if (typeof value === 'object') {
        newTokenValues = Object.keys(value).reduce((allNewTokens, key) => {
            return [
                ...allNewTokens,
                ...convertToFlatList(value[key], valueList, prefixes.concat(key)),
            ];
        }, []);
    }
    else {
        newTokenValues = [[prefixes, value]];
    }
    return [...valueList, ...newTokenValues];
}
function isTokenObject(value) {
    return value != null && typeof value === 'object';
}
