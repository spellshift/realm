import { useState } from "react";
import { Highlight, themes } from "prism-react-renderer";
import { ClipboardDocumentIcon, ClipboardDocumentCheckIcon } from "@heroicons/react/24/outline";
import Button from "./button/Button";

type CodeBlockProps = {
    code: string;
    language?: string;
    showCopyButton?: boolean;
};

const CodeBlock = ({ code, language = "", showCopyButton = false }: CodeBlockProps) => {
    const [copied, setCopied] = useState(false);

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(code);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error("Failed to copy text:", err);
        }
    };

    return (
        <div className="relative">
            {showCopyButton && (
                <Button
                    onClick={handleCopy}
                    buttonVariant="ghost"
                    buttonStyle={{ color: "gray", size: "xs" }}
                    className="absolute top-2 right-2"
                    aria-label={copied ? "Copied" : "Copy code"}
                    leftIcon={copied
                        ? <ClipboardDocumentCheckIcon className="w-4 h-4 text-green-600" />
                        : <ClipboardDocumentIcon className="w-4 h-4" />
                    }
                />
            )}

            <Highlight theme={themes.vsLight} code={code} language={language || "text"}>
                {({ style, tokens, getLineProps, getTokenProps }) => (
                    <pre
                        style={{
                            ...style,
                            margin: 0,
                            padding: "0.75rem",
                            paddingRight: showCopyButton ? "3rem" : "0.75rem",
                            marginRight: showCopyButton ? "3rem" : "0rem",
                            borderRadius: "0.375rem",
                            fontSize: "0.8rem",
                            overflowX: "auto",
                        }}
                    >
                        {tokens.map((line, i) => (
                            <div key={i} {...getLineProps({ line })}>
                                {line.map((token, key) => (
                                    <span key={key} {...getTokenProps({ token })} />
                                ))}
                            </div>
                        ))}
                    </pre>
                )}
            </Highlight>
        </div>
    );
};

export default CodeBlock;
