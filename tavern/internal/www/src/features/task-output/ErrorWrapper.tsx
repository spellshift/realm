import React from "react";
import { CodeBlock, tomorrow } from "react-code-blocks";

const ErrorWrapper = ({ error }: { error: string }) => {
    return (
        <div className="flex flex-col gap-2">
            <h3 className="text-2xl text-gray-800">Error</h3>
            <div className="bg-gray-200 rounded-md p-0.5 ">
                <CodeBlock
                    text={error}
                    language={""}
                    showLineNumbers={false}
                    theme={tomorrow}
                    codeBlock
                />
            </div>
        </div>
    );
}
export default ErrorWrapper;
