import { useState } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { EyeSlashIcon, EyeIcon, ClipboardDocumentIcon } from "@heroicons/react/24/solid";

const Credential = ({ secret }: any) => {
    const [hidden, setHidden] = useState(true);
    return (
        <div className="flex flex-row items-center gap-2">
            <p className="flex-1" style={{ whiteSpace: 'pre-line' }}>
                {hidden ? secret.replace(/[^\n]/g, '*') : secret}
            </p>
            <div className="flex flex-row gap-1">
                <Button buttonStyle={{color: "gray", size: "sm"}} onClick={async () => await navigator.clipboard?.writeText?.(secret)}>
                    <ClipboardDocumentIcon className="text-purple-900 w-4 h-4"/>
                </Button>
                <Button buttonStyle={{color: "gray", size: "sm"}} onClick={() => setHidden(!hidden)}>
                    {hidden ? <EyeIcon className="text-purple-900 w-4 h-4"/> : <EyeSlashIcon className="text-purple-900 w-4 h-4"/>}
                </Button>
            </div>
        </div>
    )
}

export default Credential;