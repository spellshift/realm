import { ReactNode } from "react";
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import { CodeBlock, tomorrow } from "react-code-blocks";
import { TomeNode } from "../utils/interfacesQuery";
import { FieldInputParams } from "../utils/interfacesUI";

type Props = {
    tome: TomeNode;
    params: Array<FieldInputParams>;
    showParamValues?: boolean;
    isExpanded?: boolean;
    onToggle?: (expandedIndex: number) => void;
    leftContent?: ReactNode;
    showDetailsButton?: boolean;
    showIcon?: boolean;
}

const TomeAccordion = (props: Props) => {
    const {
        tome,
        params,
        showParamValues = true,
        isExpanded,
        onToggle,
        leftContent,
        showDetailsButton = true,
    } = props;

    const isControlled = isExpanded !== undefined;
    const accordionIndex = isControlled ? (isExpanded ? 0 : -1) : undefined;

    return (
        <Accordion
            index={accordionIndex}
            allowToggle
            className='w-full'
            onChange={onToggle ? (expandedIndex: number) => onToggle(expandedIndex) : undefined}
        >
            <AccordionItem>
                <h2>
                    <AccordionButton>
                        <div className='flex flex-row gap-4 w-full justify-between items-center'>
                            {leftContent}
                            <Box as="div" flex='1' textAlign='left' className='flex flex-col w-full gap-1'>
                                <div className="text-gray-600 break-all">
                                    {tome.name}
                                </div>
                                {showParamValues && params && params.length > 0 && (
                                    params.map((paramDef: FieldInputParams) => {
                                        if (paramDef.value) {
                                            return (
                                                <div className="flex flex-row gap-1 text-sm text-gray-600" key={paramDef.name}>
                                                    <div className="capitalize">{paramDef.name}:</div>
                                                    <div className="break-all">{paramDef.value}</div>
                                                </div>
                                            )
                                        }
                                        return null;
                                    })
                                )}
                                {!showParamValues && params && params.length > 0 &&
                                    <div className="flex flex-row flex-wrap gap-1 text-sm text-gray-600">
                                        Parameters:
                                        {params.map((element: FieldInputParams, index: number) => {
                                            return <div key={`${index}_${element.name}`}>{element.label}{index < (params.length - 1) && ","}</div>
                                        })}
                                    </div>
                                }
                                {tome.tactic && tome.tactic !== "UNSPECIFIED" && <div className="text-sm text-gray-600 gap-2">Tactic: <span className="lowercase">{tome?.tactic}</span></div>}
                            </Box>
                            {showDetailsButton &&
                                <div className='text-sm items-center px-2'>
                                    <AccordionIcon />
                                </div>
                            }
                        </div>
                    </AccordionButton>
                </h2>
                {tome.eldritch &&
                    <AccordionPanel pb={4}>
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
    );
}
export default TomeAccordion;
