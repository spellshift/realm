type Option<T extends string | number> = { label: string; value: T };

type Props<T extends string | number> = {
    label: string;
    options: Option<T>[];
    value: T;
    onChange: (value: T) => void;
};

export const SegmentedControl = <T extends string | number>({ label, options, value, onChange }: Props<T>) => (
    <div className="inline-flex max-w-full flex-wrap gap-0.5 rounded-md bg-gray-100 p-1" role="group" aria-label={label}>
        {options.map((option) => (
            <button
                key={option.value}
                type="button"
                aria-pressed={value === option.value}
                onClick={() => onChange(option.value)}
                className={`rounded px-3 py-2 text-sm font-semibold focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-purple-500 ${
                    value === option.value
                        ? "bg-white text-purple-700 shadow-sm"
                        : "text-gray-600 hover:text-gray-900"
                }`}
            >
                {option.label}
            </button>
        ))}
    </div>
);
