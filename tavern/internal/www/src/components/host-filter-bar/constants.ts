import { FilterOptionGroup } from "../beacon-filter-bar/types";
import { FilterBarOption, HostFilterField, HostFilterPredicate, HostFilterRow } from "../../utils/interfacesUI";
import { getBeaconFilterNameByTypes } from "../../utils/utils";

export interface HostFilterFieldDef {
    value: HostFilterField;
    label: string;
    input: 'enum' | 'free';
}

export const HOST_FILTER_FIELDS: HostFilterFieldDef[] = [
    { value: 'platform', label: 'Platform', input: 'enum' },
    { value: 'transport', label: 'Transport', input: 'enum' },
    { value: 'status', label: 'Status', input: 'enum' },
    { value: 'service', label: 'Service tag', input: 'enum' },
    { value: 'group', label: 'Group tag', input: 'enum' },
    { value: 'principal', label: 'Principal', input: 'free' },
    { value: 'primaryIP', label: 'Primary IP', input: 'free' },
    { value: 'externalIP', label: 'External IP', input: 'free' },
    { value: 'host', label: 'Host', input: 'free' },
    { value: 'beacon', label: 'Beacon', input: 'free' },
];

export interface HostFilterPredicateDef {
    value: HostFilterPredicate;
    label: string;
    multi: boolean;
}

export const HOST_FILTER_PREDICATES: HostFilterPredicateDef[] = [
    { value: 'is', label: 'is', multi: false },
    { value: 'is-not', label: 'is not', multi: false },
    { value: 'is-in', label: 'is in', multi: true },
    { value: 'is-not-in', label: 'is not in', multi: true },
    { value: 'contains', label: 'contains', multi: false },
    { value: 'does-not-contain', label: 'does not contain', multi: false },
];

export function isMultiPredicate(predicate: HostFilterPredicate): boolean {
    return predicate === 'is-in' || predicate === 'is-not-in';
}

export function getFieldDef(field: HostFilterField): HostFilterFieldDef {
    return HOST_FILTER_FIELDS.find((def) => def.value === field) ?? HOST_FILTER_FIELDS[0];
}

export function getPredicatesForField(field: HostFilterField): HostFilterPredicateDef[] {
    if (getFieldDef(field).input === 'enum') {
        return HOST_FILTER_PREDICATES.filter((predicate) => predicate.value !== 'contains' && predicate.value !== 'does-not-contain');
    }
    return HOST_FILTER_PREDICATES;
}

export function createHostFilterRow(field: HostFilterField = 'platform'): HostFilterRow {
    return {
        id: `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
        field,
        predicate: 'is',
        values: [],
    };
}

export interface HostFilterSuggestion {
    value: string;
    label: string;
}

function toSuggestions(options: { value?: string; label?: string; name: string }[] | undefined): HostFilterSuggestion[] {
    return (options ?? []).map((option) => ({
        value: option.value ?? option.name,
        label: option.label ?? option.name,
    }));
}

// Suggestion lists mirror the legacy dropdown: fixed server/fetched lists for
// enum-like fields, autocomplete candidates for free fields. Host keeps IDs as
// values (exact matches filter by host ID); beacon and tags use names.
export function getFieldSuggestions(field: HostFilterField, groups: FilterOptionGroup[]): HostFilterSuggestion[] {
    const byLabel = new Map(groups.map((group) => [group.label, group.options]));

    switch (field) {
        case 'platform':
        case 'transport':
        case 'status':
        case 'principal':
        case 'primaryIP':
        case 'externalIP': {
            const label = field === 'primaryIP' ? 'PrimaryIPs' : field === 'externalIP' ? 'ExternalIPs' : field === 'platform' ? 'Platform' : field === 'transport' ? 'Transport' : field === 'status' ? 'Status' : 'Principal';
            return toSuggestions(byLabel.get(label));
        }
        case 'service':
        case 'group': {
            const label = field === 'service' ? 'Service' : 'Group';
            return (byLabel.get(label) ?? []).map((option) => ({
                value: option.name,
                label: option.label ?? option.name,
            }));
        }
        case 'host':
            return toSuggestions(byLabel.get('Host'));
        case 'beacon':
            return (byLabel.get('Beacon') ?? []).map((option) => ({
                value: option.name,
                label: option.label ?? option.name,
            }));
        default:
            return [];
    }
}

export function toValueOption(value: string, suggestions: HostFilterSuggestion[]): HostFilterSuggestion {
    return suggestions.find((suggestion) => suggestion.value === value) ?? { value, label: value };
}

// Converts legacy multi-select options (quest modal initial filters, locked
// context filters) into rows. Host values stay IDs, beacon/tag/principal/IP
// values stay names, matching what each side of the query expects. Status
// options are skipped: the beacon picker enforces online-only separately.
export function convertLegacyBeaconFilters(options: FilterBarOption[]): HostFilterRow[] {
    const grouped = getBeaconFilterNameByTypes(options);
    const rows: HostFilterRow[] = [];

    const pushRow = (field: HostFilterField, values: string[]) => {
        if (values.length < 1) {
            return;
        }
        rows.push({
            id: `${field}-${rows.length}`,
            field,
            predicate: values.length === 1 ? 'is' : 'is-in',
            values: [...values],
        });
    };

    pushRow('platform', grouped.platform);
    pushRow('transport', grouped.transport);
    pushRow('service', grouped.service);
    pushRow('group', grouped.group);
    pushRow('principal', grouped.principal);
    pushRow('primaryIP', grouped.primaryIP);
    pushRow('externalIP', grouped.externalIP);
    pushRow('host', grouped.host);
    pushRow('beacon', grouped.beacon);
    return rows;
}
