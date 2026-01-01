import { useState } from 'react'
import { RadioGroup } from '@headlessui/react'
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box, Heading } from '@chakra-ui/react'
import { safelyJsonParse } from '../../../utils/utils'
import { CheckCircleIcon } from '@heroicons/react/24/solid'
import { CodeBlock, tomorrow } from 'react-code-blocks'
import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState'
import FreeTextSearch from '../../../components/tavern-base-ui/FreeTextSearch'
import { FieldInputParams } from '../../../utils/interfacesUI'
import { TomeNode } from '../../../utils/interfacesQuery'

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
                                className={({ active, checked }) =>
                                    `${active
                                        ? 'ring-2 ring-white/60 ring-offset-2 ring-offset-purple-800'
                                        : ''
                                    }
                                        bg-white relative flex cursor-pointer rounded-lg shadow-md focus:outline-none`
                                }
                            >
                                {({ active, checked }) => {
                                    const isSavedInForm = selected?.id === tome?.id;
                                    const { params } = safelyJsonParse(tome?.paramDefs || "");
                                    const handleAccordionClick = (expandedIndex: number, checked: boolean) => {
                                        if (checked) {
                                            setIsExpanded(expandedIndex >= 0 ? true : false);
                                        }
                                    }
                                    return (
                                        <Accordion index={checked && isExpanded ? 0 : -1} allowToggle className='w-full' onChange={(expandedIndex: number) => handleAccordionClick(expandedIndex, checked)}>
                                            <AccordionItem>
                                                <h2>
                                                    <AccordionButton>
                                                        <div className='flex flex-row gap-4 w-full items-center'>
                                                            {(checked || isSavedInForm) ? (
                                                                <div className="shrink-0 text-purple-800">
                                                                    <CheckCircleIcon className="w-6 h-6" />
                                                                </div>
                                                            ) : (
                                                                <span
                                                                    aria-hidden="true"
                                                                    className={`h-6 w-6 rounded-full border-2 border-black border-opacity-10 ${(checked || isSavedInForm) && 'bg-purple-800'}`}
                                                                />
                                                            )}
                                                            <Box as="div" flex='1' textAlign='left' className='flex flex-col w-full gap-1'>
                                                                <RadioGroup.Label
                                                                    as="div"
                                                                    className={`flex flex-row gap-2 items-center`}
                                                                >
                                                                    <h4 className=' text-gray-900 font-semibold'>{tome.name}</h4>
                                                                </RadioGroup.Label>
                                                                <RadioGroup.Description
                                                                    as="div"
                                                                    className={`flex flex-col gap-1 w-full text-sm text-gray-600`}
                                                                >
                                                                    <p>{tome.description}</p>
                                                                    {params &&
                                                                        <div className="flex flex-row flex-wrap gap-1">
                                                                            Parameters:
                                                                            {params && params.map((element: FieldInputParams, index: number) => {
                                                                                return <div key={`${index}_${element.name}`}>{element.label}{index < (params.length - 1) && ","}</div>
                                                                            })}
                                                                        </div>
                                                                    }
                                                                    {tome.tactic && tome.tactic !== "UNSPECIFIED" && <div>Tactic: <span className="lowercase">{tome?.tactic}</span></div>}
                                                                </RadioGroup.Description>
                                                            </Box>
                                                            {checked &&
                                                                <div className='text-sm  items-center'>
                                                                    Details
                                                                    <AccordionIcon />
                                                                </div>
                                                            }
                                                        </div>
                                                    </AccordionButton>
                                                </h2>
                                                {tome.eldritch &&
                                                    <AccordionPanel pb={4} pl={12}>
                                                        <CodeBlock
                                                            text={tome.eldritch}
                                                            language={"python"}
                                                            showLineNumbers={false}
                                                            theme={tomorrow}
                                                        />
                                                    </AccordionPanel>
                                                }
                                            </AccordionItem>
                                        </Accordion>
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
