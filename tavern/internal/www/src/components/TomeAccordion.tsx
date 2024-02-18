import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import { CodeBlock, tomorrow } from "react-code-blocks";
import { Tome, TomeParams } from "../utils/consts";
import { safelyJsonParse } from "../utils/utils";

type Props = {
    tome: Tome,
    params: any,
    paramKeys: Array<string>
}
const TomeAccordion = (props: Props) => {
    const { tome, params, paramKeys } = props;
    const { params: paramDefs } = safelyJsonParse(tome?.paramDefs);
    return (
        <Accordion allowToggle className='w-full'>
            <AccordionItem>
                <h2>
                    <AccordionButton>
                        <div className='flex flex-row gap-2 w-full items-center'>
                            <Box as="div" flex='1' textAlign='left' className='flex flex-col w-full gap-1'>
                                <div
                                    className={`items-center font-medium text-gray-900`}
                                >
                                    {tome.name}
                                </div>
                                <div
                                    className={`flex flex-col gap- w-full text-sm text-gray-600 gap-2`}
                                >
                                    <p>{tome.description}</p>
                                    {paramDefs && paramDefs.length > 0 &&
                                        <div className="flex flex-row gap-1">
                                            Parameters:
                                            {paramDefs && paramDefs.map((element: TomeParams, index: number) => {
                                                return <div key={`${index}_${element.name}`}>{element.label}{index < (paramDefs.length - 1) && ","}</div>
                                            })}
                                        </div>
                                    }
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
                        {paramKeys.length > 0 && (
                            <div className="flex flex-row gap-8 flex-wrap">
                                {paramKeys.map((value: string) => {
                                    return (
                                        <div className="flex flex-col gap-0" key={value}>
                                            <div className="font-semibold">{value}</div>
                                            <div>{params[value]}</div>
                                        </div>
                                    )
                                })}
                            </div>
                        )}
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
    );
}
export default TomeAccordion;
