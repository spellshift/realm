import { describe, it, expect } from 'vitest';
import { asPlainObject, toDisplayString, constructTomeParams, combineTomeValueAndFields } from './utils';

describe('asPlainObject', () => {
    it('returns the object when given a plain object', () => {
        const obj = { name: 'test', value: 123 };
        expect(asPlainObject(obj)).toBe(obj);
    });

    it('returns empty object for non-plain-object values', () => {
        expect(asPlainObject(['a', 'b'])).toEqual({});
        expect(asPlainObject(null)).toEqual({});
        expect(asPlainObject(undefined)).toEqual({});
        expect(asPlainObject('string')).toEqual({});
    });

    it('prevents access to Array.prototype methods like "filter"', () => {
        const arr: unknown = [];
        const safe = asPlainObject(arr);
        expect(safe["filter"]).toBeUndefined();
    });
});

describe('toDisplayString', () => {
    it('returns strings as-is', () => {
        expect(toDisplayString('hello')).toBe('hello');
    });

    it('returns null for empty/nullish values', () => {
        expect(toDisplayString('')).toBeNull();
        expect(toDisplayString(null)).toBeNull();
        expect(toDisplayString(undefined)).toBeNull();
    });

    it('converts primitives to strings', () => {
        expect(toDisplayString(123)).toBe('123');
        expect(toDisplayString(false)).toBe('false');
    });

    it('returns null for functions', () => {
        // This is the bug case - Array.prototype.filter was being rendered
        expect(toDisplayString(Array.prototype.filter)).toBeNull();
        expect(toDisplayString(() => {})).toBeNull();
    });

    it('JSON stringifies objects and arrays', () => {
        expect(toDisplayString({ key: 'value' })).toBe('{"key":"value"}');
        expect(toDisplayString([1, 2])).toBe('[1,2]');
    });
});

describe('constructTomeParams', () => {
    const sampleParamDefs = JSON.stringify([
        { name: 'password', label: 'Password', type: 'string', placeholder: 'secret' },
        { name: 'filter', label: 'Filter', type: 'string', placeholder: 'ldap' },
    ]);

    it('returns empty array when inputs are null', () => {
        expect(constructTomeParams(null, sampleParamDefs)).toEqual([]);
        expect(constructTomeParams('{}', null)).toEqual([]);
    });

    it('constructs params with values from questParameters', () => {
        const questParams = JSON.stringify({ password: 'secret123', filter: 'Admin*' });
        const result = constructTomeParams(questParams, sampleParamDefs);

        expect(result).toHaveLength(2);
        expect(result[0]).toMatchObject({ name: 'password', value: 'secret123' });
        expect(result[1]).toMatchObject({ name: 'filter', value: 'Admin*' });
    });

    it('handles "[]" questParameters without exposing Array.prototype.filter', () => {
        // THE BUG: JSON.parse("[]") returns array, arr["filter"] === Array.prototype.filter
        const result = constructTomeParams('[]', sampleParamDefs);

        const filterParam = result.find(p => p.name === 'filter');
        expect(filterParam?.value).toBe('');
        expect(typeof filterParam?.value).not.toBe('function');
    });
});

describe('combineTomeValueAndFields', () => {
    const sampleFields = [
        { name: 'password', label: 'Password', type: 'string', placeholder: 'secret' },
        { name: 'filter', label: 'Filter', type: 'string', placeholder: 'ldap' }
    ];

    it('combines values with field definitions', () => {
        const result = combineTomeValueAndFields({ password: 'test123' }, sampleFields);
        expect(result[0]).toMatchObject({ name: 'password', value: 'test123' });
        expect(result[1]).toMatchObject({ name: 'filter', value: '' }); // missing = empty
    });

    it('handles array input without exposing Array.prototype.filter', () => {
        const result = combineTomeValueAndFields([] as unknown as Record<string, unknown>, sampleFields);
        expect(result.find(p => p.name === 'filter')?.value).toBe('');
    });
});
