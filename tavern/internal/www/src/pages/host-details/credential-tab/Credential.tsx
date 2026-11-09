import { useState } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";
import Tooltip from "../../../components/tavern-base-ui/Tooltip";
import { EyeSlashIcon, EyeIcon } from "@heroicons/react/24/solid";

const MAX_MASK_LENGTH = 32;

interface CredentialProps {
    secret: string;
}

const Credential = ({ secret }: CredentialProps) => {
    const [hidden, setHidden] = useState(true);
    const safeSecret = secret ?? "";
    const maskedSecret = "*".repeat(Math.min(safeSecret.length, MAX_MASK_LENGTH));
    const toggleLabel = hidden ? "Show secret" : "Hide secret";

    return (
        <>
            <span className="truncate">
                {hidden ? maskedSecret : safeSecret}
            </span>
            <Tooltip label={toggleLabel}>
                <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "gray", size: "xs" }}
                    onClick={() => setHidden(!hidden)}
                    className="float-right shrink-0"
                    aria-label={toggleLabel}
                >
                    {hidden ? <EyeIcon className="w-4 h-4" /> : <EyeSlashIcon className="w-4 h-4" />}
                </Button>
            </Tooltip>
        </>
    );
};

export default Credential;
