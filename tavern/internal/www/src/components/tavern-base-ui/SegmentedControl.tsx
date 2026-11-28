import { RadioGroup } from "@headlessui/react";

type Option<T extends string | number> = { label: string; value: T };

type Props<T extends string | number> = {
    label: string;
    options: Option<T>[];
    value: T;
    onChange: (value: T) => void;
};

export const SegmentedControl = <T extends string | number>({ label, options, value, onChange }: Props<T>) => (
    <RadioGroup
        value={value}
        onChange={onChange}
        aria-label={label}
        className="inline-flex w-fit max-w-full flex-wrap gap-0.5 self-start rounded-md bg-gray-100 p-1"
    >
        {options.map((option) => (
            <RadioGroup.Option
                key={option.value}
                value={option.value}
                className={({ checked }) =>
                    `cursor-pointer rounded px-3 py-2 text-sm font-semibold focus:outline-none focus-visible:ring-2 focus-visible:ring-purple-500 ${
                        checked
                            ? "bg-white text-purple-700 shadow-sm"
                            : "text-gray-600 hover:text-gray-900"
                    }`
                }
            >
                {option.label}
            </RadioGroup.Option>
        ))}
    </RadioGroup>
);
