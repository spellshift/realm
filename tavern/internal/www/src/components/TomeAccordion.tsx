import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import { CodeBlock, tomorrow } from "react-code-blocks";
import { TomeNode } from "../utils/interfacesQuery";
import { FieldInputParams } from "../utils/interfacesUI";

type Props = {
    tome: TomeNode,
    params: Array<FieldInputParams>,
    noParamValues?: boolean
}
const TomeAccordion = (props: Props) => {
    const { tome, params } = props;
    return (
        <Accordion allowToggle className='w-full'>
            <AccordionItem>
                <h2>
                    <AccordionButton>
                        <div className='flex flex-row gap-2 w-full items-center'>
                            <Box as="div" flex='1' textAlign='left' className='flex flex-col w-full gap-1'>
                                <div
                                    className={`items-center font-semibold text-gray-900`}
                                >
                                    {tome.name}
                                </div>
                                <div
                                    className={`flex flex-col w-full text-sm text-gray-600 gap-1`}
                                >
                                    <p>{tome.description}</p>
                                    {params && params.length > 0 &&
                                        <div className="flex flex-col md:flex-row gap-1">
                                            Parameters:
                                            {params && params.map((element: FieldInputParams, index: number) => {
                                                return <div key={`${index}_${element.name}`}>{element.label}{index < (params.length - 1) && ","}</div>
                                            })}
                                        </div>
                                    }
                                    <div>
                                        {tome.tactic && tome.tactic !== "UNSPECIFIED" && <div>Tactic: <span className="lowercase">{tome?.tactic}</span></div>}
                                    </div>

                                </div>
                            </Box>
                            <div className='text-sm  items-center'>
                                Details
                                <AccordionIcon />
                            </div>
                        </div>
                    </AccordionButton>
                </h2>
                {tome.eldritch &&
                    <AccordionPanel pb={4} pl={4} className="flex flex-col gap-2">
                        {params && params.length > 0 && (
                            <div className="flex flex-row gap-8 flex-wrap">
                                {params.map((paramDef: FieldInputParams) => {
                                    if (paramDef.value) {
                                        return (
                                            <div className="flex flex-col gap-0 text-sm px-2" key={paramDef.name}>
                                                <div className="font-semibold">{paramDef.name}</div>
                                                <div>{paramDef.value}</div>
                                            </div>
                                        )
                                    }
                                    return null;
                                })}
                            </div>
                        )}
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
