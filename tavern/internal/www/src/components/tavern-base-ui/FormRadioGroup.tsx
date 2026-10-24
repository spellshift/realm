import React from "react";
import { RadioGroup } from '@headlessui/react'
import { Heading } from '@chakra-ui/react'
import { EmptyState, EmptyStateType } from './EmptyState'

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(' ')
}

type Props = {
    label: string;
    data: Array<any>;
    selected: any,
    setSelected: (arg: any) => void;
 }
export const FormRadioGroup = (props: Props) => {
    const { label, data, selected, setSelected } = props;
    return (
        <RadioGroup value={selected} onChange={setSelected}>
        <RadioGroup.Label> 
            <Heading size="sm" >{label}</Heading>
        </RadioGroup.Label>
        <div className="-space-y-px rounded-md bg-white mt-4 flex flex-col gap-2 max-h-72 overflow-y-scroll">
            {data.length === 0 && (
                <EmptyState type={EmptyStateType.noData} label="No data found"/>
            )}
            {data?.map((item, tomeIdx) => (
            <RadioGroup.Option
                key={item.name}
                value={item}
                className={({ checked }) =>
                classNames(
                    tomeIdx === 0 ? 'rounded-tl-md rounded-tr-md' : '',
                    tomeIdx === data.length - 1 ? 'rounded-bl-md rounded-br-md' : '',
                    checked ? 'z-10 border-purple-200 bg-purple-50' : 'border-gray-200',
                    'relative flex cursor-pointer border p-4 focus:outline-none'
                )
                }
            >
                {({ active, checked }) => (
                <>
                    <span
                    className={classNames(
                        checked ? 'bg-purple-700 border-transparent' : 'bg-white border-gray-300',
                        active ? 'ring-2 ring-offset-2 ring-purple-700' : '',
                        'mt-0.5 h-4 w-4 shrink-0 cursor-pointer rounded-full border flex items-center justify-center'
                    )}
                    aria-hidden="true"
                    >
                    <span className="rounded-full bg-white w-1.5 h-1.5" />
                    </span>
                    <span className="ml-3 flex flex-col">
                    <RadioGroup.Label
                        as="span"
                        className={classNames(checked ? 'text-purple-900' : 'text-gray-900', 'block text-sm font-medium')}
                    >
                        {item.name}
                    </RadioGroup.Label>
                    <RadioGroup.Description
                        as="span"
                        className={classNames(checked ? 'text-purple-700' : 'text-gray-500', 'block text-sm')}
                    >
                        {item.description}
                    </RadioGroup.Description>
                    </span>
                </>
                )}
            </RadioGroup.Option>
            ))}
        </div>
        </RadioGroup>
    )
}
