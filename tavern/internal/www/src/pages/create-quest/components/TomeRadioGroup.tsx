import { useState } from 'react'
import { RadioGroup } from '@headlessui/react'
import { Heading } from '@chakra-ui/react'
import { safelyJsonParse } from '../../../utils/utils'
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

const TomeRadioGroup = ({ label, data, selected, setSelected }: TomeRadioGroupProps) => {
    const [filteredData, setFilteredData] = useState(data);
    const [isExpanded, setIsExpanded] = useState(false);

    const handleSearch = (text: string) => {
        const fd = data.filter((tome) => {
            let tomeName = tome.name.toLowerCase();
            let tomeDesc = tome?.description?.toLowerCase();
            let tomeParams = tome?.paramDefs?.toLowerCase();

            let searchText = text.toLowerCase();
            return tomeParams?.includes(searchText) || tomeDesc?.includes(searchText) || tomeName.includes(searchText) || (selected && selected.name === tome?.name);
        })
        setFilteredData(fd);
    }

    return (
        <div className="w-full">
            <div className="mx-auto w-full flex flex-col gap-2">
                <FreeTextSearch placeholder='Search by tome definition' setSearch={handleSearch} />
                <RadioGroup value={selected || undefined} onChange={setSelected} className="flex flex-col gap-3">
                    <RadioGroup.Label className="sr-only">
                        <Heading size="sm" >{label}</Heading>
                    </RadioGroup.Label>
                    <div className="space-y-2 md-scroll-container p-2">
                        {(filteredData.length === 0 && data.length > 0) &&
                            <EmptyState label='No tomes matching your search term' type={EmptyStateType.noMatches} />
                        }
                        {filteredData.map((tome) => (
                            <RadioGroup.Option
                                key={tome.name}
                                value={tome}
                                className={({ checked }) =>
                                    `${checked
                                        ? 'ring-2 ring-white/60 ring-offset-2 ring-offset-purple-800'
                                        : ''
                                    }
                                        bg-white relative flex cursor-pointer rounded-lg shadow-md focus:outline-none`
                                }
                            >
                                {({ checked }) => {
                                    const isSavedInForm = selected?.id === tome?.id;
                                    const { params } = safelyJsonParse(tome?.paramDefs || "");
                                    const handleAccordionClick = (expandedIndex: number) => {
                                        if (checked) {
                                            setIsExpanded(expandedIndex >= 0 ? true : false);
                                        }
                                    }

                                    const selectionIndicator = (checked || isSavedInForm) ? (
                                        <div className="shrink-0 text-purple-800">
                                            <CheckCircleIcon className="w-6 h-6" />
                                        </div>
                                    ) : (
                                        <span
                                            aria-hidden="true"
                                            className={`h-6 w-6 rounded-full border-2 border-black border-opacity-10 ${(checked || isSavedInForm) && 'bg-purple-800'}`}
                                        />
                                    );

                                    return (
                                        <TomeAccordion
                                            tome={tome}
                                            params={params || []}
                                            showParamValues={false}
                                            isExpanded={checked && isExpanded}
                                            onToggle={handleAccordionClick}
                                            leftContent={selectionIndicator}
                                            showDetailsButton={checked}
                                        />
                                    )
                                }}
                            </RadioGroup.Option>
                        ))}
                    </div>
                </RadioGroup>
            </div>
        </div>
    )
}
export default TomeRadioGroup;
