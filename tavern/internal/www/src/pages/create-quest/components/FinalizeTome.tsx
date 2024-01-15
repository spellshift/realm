import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel, Box } from "@chakra-ui/react";
import { CodeBlock, tomorrow } from "react-code-blocks";
import { Tome, TomeParams } from "../../../utils/consts";

type Props = {
    tome: Tome
    params: Array<TomeParams>
}
const FinalizeTome = ({ tome, params }: Props) => {
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
                                    className={`flex flex-col gap-2 w-full text-sm text-gray-600`}
                                >
                                    <p>{tome.description}</p>
                                    {params &&
                                        <div className="flex flex-col gap-1">
                                            {params && params.map((element: TomeParams, index: number) => {
                                                return (
                                                    <div key={`${index}_${element.name}`} className="flex flex-row gap-1">
                                                        <label className="font-medium">{element.label}:</label>
                                                        <p>{element.value}</p>
                                                    </div>
                                                )
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
    );
}
export default FinalizeTome;
