import { useState } from 'react'
import { RadioGroup } from '@headlessui/react'
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box, Heading, Input, InputGroup, InputLeftElement } from '@chakra-ui/react'
import { TomeParams } from '../../../utils/consts'
import { safelyJsonParse } from '../../../utils/utils'
import { CheckCircleIcon } from '@heroicons/react/24/solid'
import { CodeBlock, tomorrow } from 'react-code-blocks'
import { SearchIcon } from '@chakra-ui/icons'
import { EmptyState, EmptyStateType } from '../../../components/tavern-base-ui/EmptyState'

const TomeRadioGroup = (
    { label, data, selected, setSelected }: {
        label: string;
        data: Array<any>;
        selected: any,
        setSelected: (arg: any) => void;
    }
) => {
    const [filteredData, setFilteredData] = useState(data);
    const [isExpanded, setIsExpanded] = useState(false);

    const handleSearch = (data: Array<any>, text: string) => {
        const fd = data.filter((tome) => {
            let tomeName = tome.name.toLowerCase();
            let searchText = text.toLowerCase();
            return tomeName.includes(searchText) || (selected && selected.name === tome?.name);
        })
        setFilteredData(fd);
    }

    return (
        <div className="w-full">
            <div className="mx-auto w-full">
                <RadioGroup value={selected} onChange={setSelected} className="flex flex-col gap-3">
                    <RadioGroup.Label className="sr-only">
                        <Heading size="sm" >{label}</Heading>
                    </RadioGroup.Label>
                    <InputGroup>
                        <InputLeftElement pointerEvents='none'>
                            <SearchIcon color='gray.300' />
                        </InputLeftElement>
                        <Input placeholder='Search by tome name' colorScheme="purple" onChange={(event) => handleSearch(data, event.target.value)} />
                    </InputGroup>
                    <div className="space-y-2 md-scroll-container py-2 px-4">
                        {(filteredData.length === 0 && data.length > 0) &&
                            <EmptyState label='No tomes matching your search term' type={EmptyStateType.noMatches} />
                        }
                        {filteredData.map((tome) => (
                            <RadioGroup.Option
                                key={tome.name}
                                value={tome}
                                className={({ active, checked }) =>
                                    `${active
                                        ? 'ring-2 ring-white/60 ring-offset-2 ring-offset-purple-300'
                                        : ''
                                    }
                                        bg-white relative flex cursor-pointer rounded-lg shadow-md focus:outline-none`
                                }
                            >
                                {({ active, checked }) => {
                                    const isSavedInForm = selected?.id === tome?.id;
                                    const { params } = safelyJsonParse(tome?.paramDefs);
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
                                                        <div className='flex flex-row gap-2 w-full items-center'>
                                                            {(checked || isSavedInForm) && (
                                                                <div className="shrink-0 text-purple-500">
                                                                    <CheckCircleIcon className="w-8 h-8" />
                                                                </div>
                                                            )}
                                                            <Box as="div" flex='1' textAlign='left' className='flex flex-col w-full gap-1'>
                                                                <RadioGroup.Label
                                                                    as="div"
                                                                    className={`items-center font-medium text-gray-900`}
                                                                >
                                                                    {tome.name}
                                                                </RadioGroup.Label>
                                                                <RadioGroup.Description
                                                                    as="div"
                                                                    className={`flex flex-col gap- w-full text-sm text-gray-600`}
                                                                >
                                                                    <p>{tome.description}</p>
                                                                    {params &&
                                                                        <div className="flex flex-row gap-1">
                                                                            Parameters:
                                                                            {params && params.map((element: TomeParams, index: number) => {
                                                                                return <div key={`${index}_${element.name}`}>{element.label}{index < (params.length - 1) && ","}</div>
                                                                            })}
                                                                        </div>
                                                                    }
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
                                                            className="w-full"
                                                            text={tome.eldritch}
                                                            language={"python"}
                                                            showLineNumbers={false}
                                                            theme={tomorrow}
                                                            codeBlock
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
