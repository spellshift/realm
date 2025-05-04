import { useState } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { EyeSlashIcon, EyeIcon } from "@heroicons/react/24/solid";

const Credential = ({ secret }: any) => {
    const [hidden, setHidden] = useState(true);
    return (
        <>
            {hidden ? '*'.repeat(secret.length) : secret}
            <Button buttonStyle={{color: "gray", size: "xs"}} onClick={() => setHidden(!hidden)} className="float-right">
                {hidden ? <EyeIcon className="text-purple-900 w-4 h-4"/> : <EyeSlashIcon className="text-purple-900 w-4 h-4"/>}
            </Button>
        </>
    )
}

export default Credential;