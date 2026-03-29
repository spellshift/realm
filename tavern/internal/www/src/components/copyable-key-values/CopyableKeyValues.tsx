import { useCallback } from "react";
import { Heading, Tooltip, useToast } from "@chakra-ui/react";
import { Copy } from "lucide-react";

import { FieldInputParams } from "../../utils/interfacesUI";
import { toDisplayString } from "../../utils/utils";

interface CopyableKeyValuesProps {
    params: FieldInputParams[];
    heading?: string;
}

const MAX_VALUE_LENGTH = 24;

const truncateValue = (value: string): string => {
    if (value.length <= MAX_VALUE_LENGTH) return value;
    return `${value.substring(0, MAX_VALUE_LENGTH)}...`;
};

export const CopyableKeyValues = ({
    params,
    heading = "Parameters",
}: CopyableKeyValuesProps) => {
    const toast = useToast();

    const handleCopy = useCallback((text: string) => {
        navigator.clipboard.writeText(text);
        toast({
            title: "Copied to clipboard",
            status: "success",
            duration: 2000,
            isClosable: true,
        });
    }, [toast]);

    if (params.length === 0) {
        return null;
    }

    return (
        <div className="flex flex-col gap-3">
            <Heading size="sm">{heading}</Heading>
            <div className="rounded-md border border-gray-200 divide-y divide-gray-200">
                {params.map((param) => {
                    const valueStr = toDisplayString(param.value);

                    return (
                        <div key={param.name} className="flex flex-row">
                            <div className="text-sm bg-gray-50 px-4 py-3 w-1/3 flex-shrink-0 border-r border-gray-200 text-wrap font-medium">{param.label}</div>
                            {valueStr ? (
                                <div
                                    className="flex-1 flex justify-end items-center px-4 py-3"
                                    onClick={() => handleCopy(valueStr)}
                                    onKeyDown={(e) => {
                                        if (e.key === "Enter" || e.key === " ") {
                                            e.preventDefault();
                                            handleCopy(valueStr);
                                        }
                                    }}
                                    tabIndex={0}
                                    role="button"
                                    aria-label={`Copy ${param.label} value: ${valueStr}`}
                                >
                                    <Tooltip label={valueStr} bg="white" color="black">
                                        <div className="flex flex-row gap-2 items-center cursor-pointer hover:text-purple-600 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-1 rounded">
                                            <code className="text-sm px-2 py-1 border border-gray-200 rounded text-wrap">
                                                {valueStr.length > MAX_VALUE_LENGTH ? truncateValue(valueStr) : valueStr}
                                            </code>
                                            <Copy className="w-3 h-3" />
                                        </div>
                                    </Tooltip>
                                </div>
                            ) : (
                                <div className="flex-1 text-sm text-red-400 italic flex justify-end items-center px-4 py-3">Not set</div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

export default CopyableKeyValues;
