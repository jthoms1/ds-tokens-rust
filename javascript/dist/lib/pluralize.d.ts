/**
 * Pluralize or singularize a word based on the passed in count.
 *
 * @param  {string}  word      The word to pluralize
 * @param  {number}  count     How many of the word exist
 * @param  {boolean} inclusive Whether to prefix with the number (e.g. 3 ducks)
 * @return {string}
 */
declare function pluralize(word: string, count: number, inclusive: boolean): string;
declare namespace pluralize {
    var plural: (word: string) => string;
    var isPlural: (word: string) => boolean;
    var singular: (word: string) => string;
    var isSingular: (word: string) => boolean;
    var addPluralRule: (rule: string | RegExp, replacement: string) => void;
    var addSingularRule: (rule: string | RegExp, replacement: string) => void;
    var addUncountableRule: (word: string | RegExp) => void;
    var addIrregularRule: (single: string, plural: string) => void;
}
export default pluralize;
