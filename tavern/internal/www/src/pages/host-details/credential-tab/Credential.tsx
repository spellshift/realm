import { useState } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";
import Tooltip from "../../../components/tavern-base-ui/Tooltip";
import { EyeSlashIcon, EyeIcon } from "@heroicons/react/24/solid";
import { Copy } from "lucide-react";
import { useToast } from "@chakra-ui/react";

const MAX_MASK_LENGTH = 32;

interface CredentialProps {
    secret: string;
}

const Credential = ({ secret }: CredentialProps) => {
    const [hidden, setHidden] = useState(true);
    const toast = useToast();
    const safeSecret = secret ?? "";
    const maskedSecret = "*".repeat(Math.min(safeSecret.length, MAX_MASK_LENGTH));
    const toggleLabel = hidden ? "Show secret" : "Hide secret";

    const copySecret = async () => {
        try {
            await navigator.clipboard.writeText(safeSecret);
            toast({ title: "Copied to clipboard", status: "success", duration: 2000 });
        } catch {
            toast({ title: "Failed to copy secret", status: "error", duration: 2000 });
        }
    };

    return (
        <>
            <span className="truncate min-w-0 flex-1">
                {hidden ? maskedSecret : safeSecret}
            </span>
            <div className="ml-auto flex shrink-0 items-center">
                <Tooltip label={toggleLabel}>
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "xs" }}
                        onClick={() => setHidden(!hidden)}
                        aria-label={toggleLabel}
                    >
                        {hidden ? <EyeIcon className="w-4 h-4" /> : <EyeSlashIcon className="w-4 h-4" />}
                    </Button>
                </Tooltip>
                <Tooltip label="Copy secret">
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "xs" }}
                        onClick={copySecret}
                        aria-label="Copy secret"
                    >
                        <Copy className="w-4 h-4" />
                    </Button>
                </Tooltip>
            </div>
        </>
    );
};

export default Credential;
