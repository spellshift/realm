import React from "react";
import { AdjustmentsHorizontalIcon, MagnifyingGlassIcon } from "@heroicons/react/24/outline";
import { X } from "lucide-react";
import { ButtonDialogPopover } from "../ButtonDialogPopover";

export type HostsFilterTheme = "dark" | "light";

export interface HostsFilterState {
    name: string;
    ip: string;
    input: string;
    output: string;
}

export const EMPTY_HOSTS_FILTERS: HostsFilterState = {
    name: "",
    ip: "",
    input: "",
    output: "",
};

export const countActiveHostFilters = (filters: HostsFilterState): number => {
    let count = 0;
    if (filters.name.trim()) count += 1;
    if (filters.ip.trim()) count += 1;
    if (filters.input.trim()) count += 1;
    if (filters.output.trim()) count += 1;
    return count;
};

export const hasActiveHostFilters = (filters: HostsFilterState): boolean =>
    countActiveHostFilters(filters) > 0;

interface FilterThemeClasses {
    barBorder: string;
    triggerButton: string;
    title: string;
    clearBtn: string;
    label: string;
    searchIcon: string;
    input: string;
    chip: string;
    chipClose: string;
}

const FILTER_THEME_CLASSES: Record<HostsFilterTheme, FilterThemeClasses> = {
    dark: {
        barBorder: "border-gray-800 bg-gray-900",
        triggerButton:
            "!text-xs !font-medium !px-2.5 !py-1 !text-gray-200 hover:!text-white hover:!bg-gray-800 border border-gray-700 !rounded-md",
        title: "text-sm font-semibold text-gray-100",
        clearBtn: "text-xs font-medium text-purple-400 hover:text-purple-300 disabled:opacity-40",
        label: "text-xs font-medium text-gray-300",
        searchIcon: "text-gray-400",
        input:
            "w-full rounded-md border border-gray-700 bg-gray-800 pl-8 pr-2.5 py-1.5 text-xs text-gray-100 placeholder-gray-400 focus:border-purple-500 focus:outline-none",
        chip: "inline-flex items-center gap-1 rounded bg-gray-800 border border-gray-700 px-1.5 py-0.5 text-[11px] text-gray-200",
        chipClose: "text-gray-400 hover:text-white",
    },
    light: {
        barBorder: "border-gray-200 bg-white",
        triggerButton:
            "!text-xs !font-medium !px-2.5 !py-1 !text-gray-700 hover:!text-gray-900 hover:!bg-gray-100 border border-gray-300 !rounded-md",
        title: "text-sm font-semibold text-gray-800",
        clearBtn: "text-xs font-medium text-purple-700 hover:text-purple-800 disabled:opacity-40",
        label: "text-xs font-medium text-gray-700",
        searchIcon: "text-gray-400",
        input:
            "w-full rounded-md border border-gray-300 bg-gray-50 pl-8 pr-2.5 py-1.5 text-xs text-gray-900 placeholder-gray-400 focus:border-purple-600 focus:bg-white focus:outline-none",
        chip: "inline-flex items-center gap-1 rounded bg-gray-100 border border-gray-200 px-1.5 py-0.5 text-[11px] text-gray-700",
        chipClose: "text-gray-500 hover:text-gray-900",
    },
};

const FILTER_FIELD_META: Array<{
    key: keyof HostsFilterState;
    label: string;
    placeholder: string;
    testId: string;
}> = [
    {
        key: "name",
        label: "Name",
        placeholder: "Host, beacon, shell, or pivot name",
        testId: "hosts-tree-filter-name",
    },
    {
        key: "ip",
        label: "IP",
        placeholder: "IP address, CIDR subnet, or SSH target",
        testId: "hosts-tree-filter-ip",
    },
    {
        key: "input",
        label: "Input",
        placeholder: "Command or Eldritch input",
        testId: "hosts-tree-filter-input",
    },
    {
        key: "output",
        label: "Output",
        placeholder: "Task output or terminal I/O",
        testId: "hosts-tree-filter-output",
    },
];

export interface HostsFilterPopoverProps {
    theme?: HostsFilterTheme;
    filters: HostsFilterState;
    onChange: (next: HostsFilterState) => void;
    onClear: () => void;
}

export const HostsFilterPopover: React.FC<HostsFilterPopoverProps> = ({
    theme = "dark",
    filters,
    onChange,
    onClear,
}) => {
    const styles = FILTER_THEME_CLASSES[theme];
    const activeCount = countActiveHostFilters(filters);

    const activeEntries = FILTER_FIELD_META.filter(
        ({ key }) => filters[key].trim().length > 0
    );

    const updateField = (key: keyof HostsFilterState, value: string) => {
        onChange({
            ...filters,
            [key]: value,
        });
    };

    return (
        <div className={`px-3 py-1.5 border-b ${styles.barBorder} space-y-1.5`}>
            <div className="flex items-center justify-between gap-2">
                <ButtonDialogPopover
                    label={`Filters (${activeCount})`}
                    leftIcon={<AdjustmentsHorizontalIcon className="w-4 h-4" />}
                    theme={theme}
                    buttonClassName={styles.triggerButton}
                    buttonTestId="hosts-tree-filters-button"
                    panelTestId="hosts-tree-filters-popover"
                    panelWidthPx={320}
                >
                    <div className="flex flex-col gap-3">
                        <div className="flex items-center justify-between border-b border-gray-700/30 pb-2">
                            <h3 className={styles.title}>Filters</h3>
                            <button
                                type="button"
                                data-testid="hosts-tree-filters-clear-all"
                                onClick={onClear}
                                disabled={activeCount === 0}
                                className={styles.clearBtn}
                            >
                                Clear all
                            </button>
                        </div>

                        {FILTER_FIELD_META.map((field) => (
                            <div key={field.key} className="flex flex-col gap-1">
                                <label
                                    htmlFor={field.testId}
                                    className={styles.label}
                                >
                                    {field.label}
                                </label>
                                <div className="relative">
                                    <MagnifyingGlassIcon
                                        data-testid={`hosts-tree-filter-icon-${field.key}`}
                                        className={`w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none ${styles.searchIcon}`}
                                    />
                                    <input
                                        id={field.testId}
                                        type="text"
                                        data-testid={field.testId}
                                        aria-label={`Filter by ${field.label}`}
                                        value={filters[field.key]}
                                        onChange={(e) => updateField(field.key, e.target.value)}
                                        placeholder={field.placeholder}
                                        className={styles.input}
                                    />
                                </div>
                            </div>
                        ))}
                    </div>
                </ButtonDialogPopover>

                {activeCount > 0 && (
                    <button
                        type="button"
                        data-testid="hosts-tree-clear-search"
                        onClick={onClear}
                        className={styles.clearBtn}
                    >
                        Clear
                    </button>
                )}
            </div>

            {activeEntries.length > 0 && (
                <div
                    data-testid="hosts-tree-active-filters"
                    className="flex flex-wrap items-center gap-1"
                >
                    {activeEntries.map(({ key, label }) => (
                        <span key={key} className={styles.chip}>
                            <span className="opacity-75">{label}:</span>
                            <span className="font-mono truncate max-w-[110px]">
                                {filters[key].trim()}
                            </span>
                            <button
                                type="button"
                                aria-label={`Remove ${label} filter`}
                                onClick={() => updateField(key, "")}
                                className={styles.chipClose}
                            >
                                <X className="w-3 h-3" />
                            </button>
                        </span>
                    ))}
                </div>
            )}
        </div>
    );
};

export default HostsFilterPopover;
