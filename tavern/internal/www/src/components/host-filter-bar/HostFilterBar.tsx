import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Select, { MultiValue, SingleValue } from "react-select";
import CreatableSelect from "react-select/creatable";
import { Input } from "@chakra-ui/react";
import { debounce } from "lodash";
import { Plus, X } from "lucide-react";
import { HostFilterField, HostFilterPredicate, HostFilterRow } from "../../utils/interfacesUI";
import { useBeaconFilterBar } from "../beacon-filter-bar/useBeaconFilterBar";
import {
    HOST_FILTER_FIELDS,
    HostFilterFieldDef,
    HostFilterSuggestion,
    createHostFilterRow,
    getFieldDef,
    getFieldSuggestions,
    getPredicatesForField,
    isMultiPredicate,
    toValueOption,
} from "./constants";
import { HostFilterBarProps } from "./types";

function fieldLabel(field: HostFilterField): string {
    return getFieldDef(field).label;
}

export function HostFilterBar({ value, onChange, title = 'Host fields', fields }: HostFilterBarProps) {
    const { options, isLoading } = useBeaconFilterBar();
    const fieldDefs = fields ?? HOST_FILTER_FIELDS;

    const handleAddRow = useCallback(() => {
        onChange([...value, createHostFilterRow()]);
    }, [onChange, value]);

    const handleRemoveRow = useCallback((id: string) => {
        onChange(value.filter((row) => row.id !== id));
    }, [onChange, value]);

    const handleFieldChange = useCallback((id: string, field: HostFilterField) => {
        onChange(value.map((row) => {
            if (row.id !== id) {
                return row;
            }
            const supported = getPredicatesForField(field).some((predicate) => predicate.value === row.predicate);
            return { ...row, field, predicate: supported ? row.predicate : 'is' as HostFilterPredicate, values: [] };
        }));
    }, [onChange, value]);

    const handlePredicateChange = useCallback((id: string, predicate: HostFilterPredicate) => {
        onChange(value.map((row) => {
            if (row.id !== id) {
                return row;
            }
            if (isMultiPredicate(predicate) === isMultiPredicate(row.predicate)) {
                return { ...row, predicate };
            }
            return { ...row, predicate, values: isMultiPredicate(predicate) ? row.values : row.values.slice(0, 1) };
        }));
    }, [onChange, value]);

    const handleValuesChange = useCallback((id: string, values: string[]) => {
        onChange(value.map((row) => (row.id === id ? { ...row, values } : row)));
    }, [onChange, value]);

    return (
        <div className="flex flex-col gap-2" data-testid="host-filter-bar">
            <label className="font-medium text-gray-700">{title}</label>
            {value.map((row, index) => (
                <HostFilterRowEditor
                    key={row.id}
                    row={row}
                    index={index}
                    groups={options}
                    isLoading={isLoading}
                    fieldDefs={fieldDefs}
                    onFieldChange={handleFieldChange}
                    onPredicateChange={handlePredicateChange}
                    onValuesChange={handleValuesChange}
                    onRemove={handleRemoveRow}
                />
            ))}
            <div>
                <button
                    type="button"
                    onClick={handleAddRow}
                    data-testid="host-filter-add"
                    className="inline-flex items-center gap-1 rounded-md border border-gray-300 px-2 py-1 text-sm text-gray-700 hover:bg-gray-50"
                >
                    <Plus className="h-4 w-4" />
                    Add filter
                </button>
            </div>
        </div>
    );
}

interface RowEditorProps {
    row: HostFilterRow;
    index: number;
    groups: Parameters<typeof getFieldSuggestions>[1];
    isLoading: boolean;
    fieldDefs: HostFilterFieldDef[];
    onFieldChange: (id: string, field: HostFilterField) => void;
    onPredicateChange: (id: string, predicate: HostFilterPredicate) => void;
    onValuesChange: (id: string, values: string[]) => void;
    onRemove: (id: string) => void;
}

function HostFilterRowEditor({ row, index, groups, isLoading, fieldDefs, onFieldChange, onPredicateChange, onValuesChange, onRemove }: RowEditorProps) {
    const suggestions = useMemo(() => getFieldSuggestions(row.field, groups), [row.field, groups]);

    const fieldOptions = useMemo(
        () => fieldDefs.map((field) => ({ value: field.value, label: field.label })),
        [fieldDefs]
    );
    const predicateOptions = useMemo(
        () => getPredicatesForField(row.field).map((predicate) => ({ value: predicate.value, label: predicate.label })),
        [row.field]
    );

    return (
        <div className="flex flex-col gap-2 rounded-md bg-gray-50 p-2" data-testid={`host-filter-row-${index}`}>
            <div className="flex items-center gap-2">
                <div className="min-w-0 flex-1" data-testid={`host-filter-field-${index}`}>
                    <label className="sr-only" htmlFor={`host-filter-${row.id}-field`}>Filter field</label>
                    <Select
                        inputId={`host-filter-${row.id}-field`}
                        aria-label="Filter field"
                        options={fieldOptions}
                        value={{ value: row.field, label: fieldLabel(row.field) }}
                        onChange={(option) => option && onFieldChange(row.id, option.value as HostFilterField)}
                        isSearchable={false}
                    />
                </div>
                <button
                    type="button"
                    onClick={() => onRemove(row.id)}
                    data-testid={`host-filter-remove-${index}`}
                    aria-label={`Remove filter ${index + 1}`}
                    className="shrink-0 rounded-md p-1 text-gray-500 hover:bg-gray-100 hover:text-gray-700"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>
            <div className="flex items-center gap-2">
                <div className="w-40 shrink-0" data-testid={`host-filter-predicate-${index}`}>
                    <label className="sr-only" htmlFor={`host-filter-${row.id}-predicate`}>Filter predicate</label>
                    <Select
                        inputId={`host-filter-${row.id}-predicate`}
                        aria-label="Filter predicate"
                        options={predicateOptions}
                        value={predicateOptions.find((option) => option.value === row.predicate)}
                        onChange={(option) => option && onPredicateChange(row.id, option.value as HostFilterPredicate)}
                        isSearchable={false}
                    />
                </div>
                <div className="min-w-0 flex-1" data-testid={`host-filter-value-${index}`}>
                    <label className="sr-only" htmlFor={`host-filter-${row.id}-value`}>Filter value</label>
                    <HostFilterValueInput
                        row={row}
                        suggestions={suggestions}
                        isLoading={isLoading}
                        onValuesChange={onValuesChange}
                    />
                </div>
            </div>
        </div>
    );
}

interface ValueInputProps {
    row: HostFilterRow;
    suggestions: HostFilterSuggestion[];
    isLoading: boolean;
    onValuesChange: (id: string, values: string[]) => void;
}

function HostFilterValueInput({ row, suggestions, isLoading, onValuesChange }: ValueInputProps) {
    return isMultiPredicate(row.predicate)
        ? <MultiValueInput row={row} suggestions={suggestions} isLoading={isLoading} onValuesChange={onValuesChange} />
        : <SingleValueInput row={row} suggestions={suggestions} isLoading={isLoading} onValuesChange={onValuesChange} />;
}

function SingleValueInput({ row, suggestions, isLoading, onValuesChange }: ValueInputProps) {
    const freeInput = getFieldDef(row.field).input === 'free';
    const value = row.values.length > 0 ? toValueOption(row.values[0], suggestions) : null;
    const handleChange = (option: SingleValue<HostFilterSuggestion>) => {
        onValuesChange(row.id, option ? [option.value] : []);
    };

    if (row.predicate === 'contains' || row.predicate === 'does-not-contain') {
        return <ContainsValueInput rowId={row.id} suggestions={suggestions} initialValue={row.values[0] ?? ''} onValuesChange={onValuesChange} />;
    }

    return freeInput
        ? <FreeSingleValueInput rowId={row.id} suggestions={suggestions} isLoading={isLoading} value={value} onChange={handleChange} />
        : <EnumSingleValueInput rowId={row.id} suggestions={suggestions} isLoading={isLoading} value={value} onChange={handleChange} />;
}

interface ContainsInputProps {
    rowId: string;
    suggestions: HostFilterSuggestion[];
    initialValue: string;
    onValuesChange: (id: string, values: string[]) => void;
}

// Substring predicates take a plain typed value, not a dropdown pick.
// Suggestions are offered through a datalist; typing filters as you go.
function ContainsValueInput({ rowId, suggestions, initialValue, onValuesChange }: ContainsInputProps) {
    const [inputValue, setInputValue] = useState(initialValue);
    const datalistId = `host-filter-${rowId}-value-suggestions`;

    const debouncedCommit = useRef(
        debounce((committed: string, targetId: string) => {
            onValuesChange(targetId, committed ? [committed] : []);
        }, 400)
    ).current;

    useEffect(() => {
        return () => {
            debouncedCommit.cancel();
        };
    }, [debouncedCommit]);

    // Sync typed text when the row is reset or changed elsewhere
    useEffect(() => {
        setInputValue(initialValue);
    }, [initialValue]);

    const commitNow = (committed: string) => {
        debouncedCommit.cancel();
        onValuesChange(rowId, committed ? [committed] : []);
    };

    return (
        <>
            <Input
                id={`host-filter-${rowId}-value`}
                aria-label="Filter value"
                type="text"
                value={inputValue}
                list={datalistId}
                placeholder="Type to filter…"
                autoComplete="off"
                onChange={(event) => {
                    setInputValue(event.target.value);
                    debouncedCommit(event.target.value, rowId);
                }}
                onBlur={() => commitNow(inputValue)}
                onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                        commitNow((event.target as HTMLInputElement).value);
                    }
                }}
            />
            <datalist id={datalistId}>
                {suggestions.map((suggestion) => (
                    <option key={suggestion.value} value={suggestion.label} />
                ))}
            </datalist>
        </>
    );
}

function MultiValueInput({ row, suggestions, isLoading, onValuesChange }: ValueInputProps) {
    const freeInput = getFieldDef(row.field).input === 'free';
    const value = row.values.map((item) => toValueOption(item, suggestions));
    const handleChange = (selected: MultiValue<HostFilterSuggestion>) => {
        onValuesChange(row.id, [...selected].map((option) => option.value));
    };

    return freeInput
        ? <FreeMultiValueInput rowId={row.id} suggestions={suggestions} isLoading={isLoading} value={value} onChange={handleChange} />
        : <EnumMultiValueInput rowId={row.id} suggestions={suggestions} isLoading={isLoading} value={value} onChange={handleChange} />;
}

interface SingleInputProps {
    rowId: string;
    suggestions: HostFilterSuggestion[];
    isLoading: boolean;
    value: HostFilterSuggestion | null;
    onChange: (option: SingleValue<HostFilterSuggestion>) => void;
}

function FreeSingleValueInput({ rowId, suggestions, isLoading, value, onChange }: SingleInputProps) {
    return (
        <CreatableSelect
            inputId={`host-filter-${rowId}-value`}
            aria-label="Filter value"
            isClearable
            isLoading={isLoading}
            options={suggestions}
            value={value}
            onChange={onChange}
            formatCreateLabel={(inputValue) => `Use "${inputValue}"`}
            placeholder="Select or type a value…"
        />
    );
}

function EnumSingleValueInput({ rowId, suggestions, isLoading, value, onChange }: SingleInputProps) {
    return (
        <Select
            inputId={`host-filter-${rowId}-value`}
            aria-label="Filter value"
            isClearable
            isLoading={isLoading}
            options={suggestions}
            value={value}
            onChange={onChange}
            placeholder="Select a value…"
        />
    );
}

interface MultiInputProps {
    rowId: string;
    suggestions: HostFilterSuggestion[];
    isLoading: boolean;
    value: HostFilterSuggestion[];
    onChange: (selected: MultiValue<HostFilterSuggestion>) => void;
}

function FreeMultiValueInput({ rowId, suggestions, isLoading, value, onChange }: MultiInputProps) {
    return (
        <CreatableSelect
            inputId={`host-filter-${rowId}-value`}
            aria-label="Filter value"
            isMulti
            isLoading={isLoading}
            options={suggestions}
            value={value}
            onChange={onChange}
            formatCreateLabel={(inputValue) => `Use "${inputValue}"`}
            placeholder="Select or type values…"
        />
    );
}

function EnumMultiValueInput({ rowId, suggestions, isLoading, value, onChange }: MultiInputProps) {
    return (
        <Select
            inputId={`host-filter-${rowId}-value`}
            aria-label="Filter value"
            isMulti
            isLoading={isLoading}
            options={suggestions}
            value={value}
            onChange={onChange}
            placeholder="Select values…"
        />
    );
}
