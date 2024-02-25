import { Heading } from "@chakra-ui/react";
import { Ring } from "@uiball/loaders";
import { FC } from "react";
import { CopyBlock, tomorrow } from "react-code-blocks";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { RepositoryType } from "../../../utils/consts";
import { useFetchRepositoryTome } from "../hooks/useFetchRepostioryTomes";

type StepAddDeploymentKeyProps = {
    setCurrStep: (step: number) => void;
    newRepository: RepositoryType;
    setOpen: (arg: any) => any;
}

const StepAddDeploymentKey: FC<StepAddDeploymentKeyProps> = ({ setCurrStep, newRepository, setOpen }) => {
    const handleOnSuccess = () => {
        setOpen(false);
    }
    const { importRepositoryTomes, loading, error } = useFetchRepositoryTome(handleOnSuccess);

    return (
        <form className="flex flex-col gap-6">
            <div className="flex flex-col">
                <h3 className="font-bold text-lg">Add public key to repository</h3>
                <p className="text-sm">
                    To import tomes, you need to give Realm access to your git repository. Copy the public key below into your repositories deployment keys, often found in admin settings.
                </p>
                <ul className="text-sm list-disc px-4 py-2">
                    <li>Setup for <a className="external-link" target="_blank" href="https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys#set-up-deploy-keys">Github</a></li>
                    <li>Setup for  <a className="external-link" target="_blank" href="https://docs.gitlab.com/ee/user/project/deploy_keys/#create-a-project-deploy-key">Gitlab</a></li>
                    <li>Setup for  <a className="external-link" target="_blank" href="https://bitbucket.org/blog/deployment-keys">Bitbucket</a></li>
                </ul>
            </div>
            {error !== "" && (
                <AlertError label={"Error importing tomes"} details={error} />
            )}
            <div className="flex flex-col gap-2">
                <Heading size="sm">Copy public key</Heading>
                <div className="bg-gray-200 rounded-md p-0.5">
                    <CopyBlock
                        text={newRepository?.publicKey}
                        language={""}
                        showLineNumbers={false}
                        theme={tomorrow}
                        codeBlock
                    />
                </div>
            </div>
            <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={() => setCurrStep(0)}
                    disabled={loading ? true : false}
                >
                    Back
                </button>
                <button
                    className="btn-primary flex flex-row gap-2"
                    onClick={(event) => {
                        event.preventDefault();
                        importRepositoryTomes(newRepository.id || "");
                    }}
                    type="submit"
                    disabled={loading ? true : false}
                >
                    {loading === true && <Ring
                        size={16}
                        lineWeight={2}
                        speed={2}
                        color="white"
                    />}
                    {loading === true ? "Importing" : "Import"} tomes
                </button>
            </div>
        </form>
    );
}
export default StepAddDeploymentKey;
