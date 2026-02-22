import { ReactNode } from "react";
import { Steps, Accordion } from "@chakra-ui/react";
import CodeBlock from "./tavern-base-ui/CodeBlock";
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
}

type ParamDisplayProps = {
    params: Array<FieldInputParams>;
};

const ParamValuesDisplay = ({ params }: ParamDisplayProps) => (
    <>
        {params.map((paramDef: FieldInputParams) => {
            if (!paramDef.value) return null;
            return (
                <div className="flex flex-row gap-1 text-sm text-gray-600" key={paramDef.name}>
                    <div className="capitalize">{paramDef.name}:</div>
                    <div className="break-all">{paramDef.value}</div>
                </div>
            );
        })}
    </>
);

const ParamCodeBlock = ({ params }: ParamDisplayProps) => (
    <div className="mx-3 flex flex-col gap-0">
        {params.map((paramDef: FieldInputParams) => {
            if (!paramDef.value) return null;
            return (
                <div className="flex flex-row items-center justify-left text-gray-800" key={paramDef.name}>
                    <div className="capitalize text-sm">{paramDef.name}:</div>
                    <CodeBlock inlineButton code={paramDef.value} language="md" showCopyButton/>
                </div>
            );
        })}
    </div>
);

const ParamLabelsDisplay = ({ params }: ParamDisplayProps) => (
    <div className="flex flex-row flex-wrap gap-1 text-sm text-gray-600">
        Parameters:
        {params.map((element: FieldInputParams, index: number) => (
            <div key={`${index}_${element.name}`}>
                {element.label}{index < (params.length - 1) && ","}
            </div>
        ))}
    </div>
);

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
    const hasParams = params && params.length > 0;
    const hasTactic = tome.tactic && tome.tactic !== "UNSPECIFIED";

    return (
        <Accordion.Root
            value={String(accordionIndex)}
            collapsible
            className="w-full"
            onValueChange={(e: any) => {
                if (onToggle) {
                    onToggle(e.value);
                }
            }}
        >
            <Accordion.Item border="none" value='item-0'>
                <h2>
                    <Accordion.ItemTrigger>
                        <div className="flex flex-row gap-4 w-full justify-between items-center">
                            {leftContent}
                            <div className="flex-1 text-left flex flex-col w-full gap-1">
                                <div className="text-gray-600 break-all">
                                    {tome.name}
                                </div>
                                {showParamValues && hasParams && <ParamValuesDisplay params={params} />}
                                {!showParamValues && hasParams && <ParamLabelsDisplay params={params} />}
                                {hasTactic && (
                                    <div className="text-sm text-gray-600 gap-2">
                                        Tactic: <span className="lowercase">{tome.tactic}</span>
                                    </div>
                                )}
                            </div>
                            {showDetailsButton && (
                                <div className="text-sm items-center px-2">
                                    <Accordion.ItemIndicator />
                                </div>
                            )}
                        </div>
                    </Accordion.ItemTrigger>
                </h2>
                {tome.eldritch && (
                    <Accordion.ItemContent pb={2} pl={10} pr={4}><Accordion.ItemBody>
                            <>
                                {showParamValues && hasParams && <ParamCodeBlock params={params} />}
                                <CodeBlock code={tome.eldritch} language="python" />
                            </>
                        </Accordion.ItemBody></Accordion.ItemContent>
                )}
            </Accordion.Item>
        </Accordion.Root>
    );
}
export default TomeAccordion;
