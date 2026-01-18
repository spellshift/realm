import { useState } from 'react'
import { RadioGroup } from '@headlessui/react'
import { classNames, safelyJsonParse } from '../../../utils/utils'
import { CheckCircleIcon } from '@heroicons/react/24/solid'
import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState'
import FreeTextSearch from '../../../components/tavern-base-ui/FreeTextSearch'
import { TomeNode } from '../../../utils/interfacesQuery'
import TomeAccordion from '../../../components/TomeAccordion'

type TomeRadioGroupProps = {
    label: string;
    data: TomeNode[];
    selected: TomeNode | null;
    setSelected: (tome: TomeNode) => void;
}

const getOptionClassName = (checked: boolean): string => {
    return classNames(
        'bg-white relative flex cursor-pointer rounded-lg shadow-md focus:outline-none',
        checked ? 'ring-2 ring-white/60 ring-offset-2 ring-offset-purple-800' : ''
    );
};

type SelectionIndicatorProps = {
    isSelected: boolean;
};

const SelectionIndicator = ({ isSelected }: SelectionIndicatorProps) => {
    if (isSelected) {
        return (
            <div className="shrink-0 text-purple-800">
                <CheckCircleIcon className="w-6 h-6" />
            </div>
        );
    }
    return (
        <span
            aria-hidden="true"
            className="h-6 w-6 rounded-full border-2 border-black border-opacity-10"
        />
    );
};

const TomeRadioGroup = ({ label, data, selected, setSelected }: TomeRadioGroupProps) => {
    const [filteredData, setFilteredData] = useState(data);
    const [isExpanded, setIsExpanded] = useState(false);

    const handleSearch = (text: string) => {
        const searchText = text.toLowerCase();
        const fd = data.filter((tome) => {
            const tomeName = tome.name.toLowerCase();
            const tomeDesc = tome?.description?.toLowerCase() || '';
            const tomeParams = tome?.paramDefs?.toLowerCase() || '';
            const isCurrentSelection = selected?.name === tome?.name;

            return (
                tomeParams.includes(searchText) ||
                tomeDesc.includes(searchText) ||
                tomeName.includes(searchText) ||
                isCurrentSelection
            );
        });
        setFilteredData(fd);
    };

    const createAccordionToggleHandler = (checked: boolean) => (expandedIndex: number) => {
        if (checked) {
            setIsExpanded(expandedIndex >= 0);
        }
    };

    return (
        <div className="w-full">
            <div className="mx-auto w-full flex flex-col gap-2">
                <FreeTextSearch placeholder="Search by tome definition" setSearch={handleSearch} />
                <RadioGroup value={selected || undefined} onChange={setSelected} className="flex flex-col gap-3">
                    <RadioGroup.Label className="sr-only">
                        <span className="text-sm font-semibold text-gray-900">{label}</span>
                    </RadioGroup.Label>
                    <div className="space-y-2 md-scroll-container p-2">
                        {filteredData.length === 0 && data.length > 0 && (
                            <EmptyState label="No tomes matching your search term" type={EmptyStateType.noMatches} />
                        )}
                        {filteredData.map((tome) => {
                            const { params } = safelyJsonParse(tome?.paramDefs || "");
                            return (
                                <RadioGroup.Option
                                    key={tome.name}
                                    value={tome}
                                    className={({ checked }) => getOptionClassName(checked)}
                                >
                                    {({ checked }) => {
                                        const isSelected = checked || selected?.id === tome?.id;
                                        return (
                                            <TomeAccordion
                                                tome={tome}
                                                params={params || []}
                                                showParamValues={false}
                                                isExpanded={checked && isExpanded}
                                                onToggle={createAccordionToggleHandler(checked)}
                                                leftContent={<SelectionIndicator isSelected={isSelected} />}
                                                showDetailsButton={checked}
                                            />
                                        );
                                    }}
                                </RadioGroup.Option>
                            );
                        })}
                    </div>
                </RadioGroup>
            </div>
        </div>
    );
};

export default TomeRadioGroup;
