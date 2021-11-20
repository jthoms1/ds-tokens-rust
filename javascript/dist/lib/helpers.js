"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.captializeFirstLetter = exports.toTitleCase = exports.dashToCapitalWords = exports.dashToPascalCase = exports.toDashCase = exports.toLowerCase = void 0;
exports.toLowerCase = (str) => str.toLowerCase();
exports.toDashCase = (str) => exports.toLowerCase(str
    .replace(/([A-Z0-9])/g, g => ' ' + g[0])
    .trim()
    .replace(/ /g, '-'));
exports.dashToPascalCase = (str) => exports.toLowerCase(str)
    .split('-')
    .map(segment => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join('');
exports.dashToCapitalWords = (str) => exports.toLowerCase(str)
    .split('-')
    .map(segment => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');
exports.toTitleCase = (str) => str.charAt(0).toUpperCase() + str.substr(1);
exports.captializeFirstLetter = (str) => str.charAt(0).toUpperCase() + str.slice(1);
