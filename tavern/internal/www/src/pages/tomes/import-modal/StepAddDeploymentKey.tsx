import { Heading } from "@chakra-ui/react";
import { FC, useState } from "react";
import { ClipboardDocumentIcon, ClipboardDocumentCheckIcon } from "@heroicons/react/24/outline";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { RepositoryNode } from "../../../utils/interfacesQuery";
import { useFetchRepositoryTome } from "../hooks/useFetchRepositoryTomes";
import Button from "../../../components/tavern-base-ui/button/Button";

type StepAddDeploymentKeyProps = {
    setCurrStep: (step: number) => void;
    newRepository: RepositoryNode;
    setOpen: (arg: any) => any;
}

const StepAddDeploymentKey: FC<StepAddDeploymentKeyProps> = ({ setCurrStep, newRepository, setOpen }) => {
    const [copied, setCopied] = useState(false);

    const handleOnSuccess = () => {
        setOpen(false);
    }
    const { importRepositoryTomes, loading, error } = useFetchRepositoryTome(handleOnSuccess);

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(newRepository?.publicKey || "");
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error("Failed to copy text:", err);
        }
    };

    return (
        <form className="flex flex-col gap-6">
            <div className="flex flex-col">
                <h3 className="font-bold text-lg">Add public key to repository</h3>
                <p className="text-sm">
                    To import tomes, you need to give Realm access to your git repository. Copy the public key below into your repositories deployment keys, often found in admin settings.
                </p>
                <ul className="text-sm list-disc px-4 py-2">
                    <li>Setup for <a className="external-link" rel="noreferrer" target="_blank" href="https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys#set-up-deploy-keys">Github</a></li>
                    <li>Setup for  <a className="external-link" rel="noreferrer" target="_blank" href="https://docs.gitlab.com/ee/user/project/deploy_keys/#create-a-project-deploy-key">Gitlab</a></li>
                    <li>Setup for  <a className="external-link" rel="noreferrer" target="_blank" href="https://bitbucket.org/blog/deployment-keys">Bitbucket</a></li>
                </ul>
            </div>
            {error !== "" && (
                <AlertError label={"Error importing tomes"} details={error} />
            )}
            <div className="flex flex-col gap-2">
                <Heading size="sm">Copy public key</Heading>
                <div className="flex flex-row items-start gap-2">
                    <Button
                        onClick={(e) => {
                            e.preventDefault();
                            handleCopy();
                        }}
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "sm" }}
                        aria-label={copied ? "Copied" : "Copy code"}
                        leftIcon={copied
                            ? <ClipboardDocumentCheckIcon className="w-5 h-5 text-green-600" />
                            : <ClipboardDocumentIcon className="w-5 h-5" />
                        }
                    />
                    <div className="flex-grow w-full">
                        <CodeBlock code={newRepository?.publicKey || ""} />
                    </div>
                </div>
            </div>
            <div className="flex flex-row gap-2">
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    buttonVariant="ghost"
                    onClick={() => setCurrStep(0)}
                    disabled={loading ? true : false}
                >
                    Back
                </Button>
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={(event) => {
                        event.preventDefault();
                        importRepositoryTomes(newRepository.id || "");
                    }}
                    type="submit"
                    disabled={loading ? true : false}
                    isLoading={loading}
                >
                    {loading === true ? "Importing" : "Import"} tomes
                </Button>
            </div>
        </form>
    );
}
export default StepAddDeploymentKey;
