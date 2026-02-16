import { useState } from "react";
import { Highlight, themes } from "prism-react-renderer";
import { ClipboardDocumentIcon, ClipboardDocumentCheckIcon } from "@heroicons/react/24/outline";
import Button from "./button/Button";

/**
 * Languages supported by prism-react-renderer
 */
type CodeBlockLanguageSupport =
    | "markup"
    | "html"
    | "xml"
    | "svg"
    | "mathml"
    | "ssml"
    | "atom"
    | "rss"
    | "css"
    | "clike"
    | "javascript"
    | "js"
    | "jsx"
    | "typescript"
    | "ts"
    | "tsx"
    | "coffeescript"
    | "coffee"
    | "actionscript"
    | "c"
    | "cpp"
    | "objc"
    | "objectivec"
    | "kotlin"
    | "kt"
    | "kts"
    | "swift"
    | "go"
    | "rust"
    | "python"
    | "py"
    | "sql"
    | "json"
    | "yaml"
    | "yml"
    | "markdown"
    | "md"
    | "graphql"
    | "regex"
    | "reason"
    | "flow"
    | "n4js"
    | "n4jsd"
    | "jsdoc"
    | "javadoclike"
    | "webmanifest";

type CodeBlockProps = {
    code: string;
    language?: CodeBlockLanguageSupport;
    showCopyButton?: boolean;
    inlineButton?: boolean;
    copyButtonPosition?: "left" | "right";
    copyButtonSize?: "xs" | "sm" | "md" | "lg" | "xl";
};

const CodeBlock = ({ code, language = 'markdown', showCopyButton = false, inlineButton = false, copyButtonPosition = 'right', copyButtonSize = 'xs' }: CodeBlockProps) => {
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
                    buttonStyle={{ color: "gray", size: copyButtonSize }}
                    className={`absolute top-2 ${copyButtonPosition === 'left' ? 'left-2' : 'right-2'}`}
                    aria-label={copied ? "Copied" : "Copy code"}
                    leftIcon={copied
                        ? <ClipboardDocumentCheckIcon className="w-4 h-4 text-green-600" />
                        : <ClipboardDocumentIcon className="w-4 h-4" />
                    }
                />
            )}

            <Highlight theme={themes.vsLight} code={code} language={language}>
                {({ style, tokens, getLineProps, getTokenProps }) => (
                    <pre
                        style={{
                            ...style,
                            margin: 0,
                            padding: "0.75rem",
                            paddingLeft: !inlineButton && showCopyButton && copyButtonPosition === 'left' ? "3rem" : "0.75rem",
                            paddingRight: !inlineButton && showCopyButton && copyButtonPosition === 'right' ? "3rem" : "0.75rem",
                            marginLeft: showCopyButton && copyButtonPosition === 'left' ? "2rem" : "0rem",
                            marginRight: showCopyButton && copyButtonPosition === 'right' ? "2rem" : "0rem",
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
