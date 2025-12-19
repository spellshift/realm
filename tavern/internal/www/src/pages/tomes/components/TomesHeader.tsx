import { ArrowUpTrayIcon, SparklesIcon } from "@heroicons/react/24/outline";
import Breadcrumbs from "../../../components/Breadcrumbs";
import Button from "../../../components/tavern-base-ui/button/Button";
import PageHeader from "../../../components/tavern-base-ui/PageHeader";
import { useEffect, useState } from "react";
import { checkAI } from "../../../services/aiService";
import CreateAITomeModal from "./CreateAITomeModal";

type TomesHeaderType = {
    setOpen: (arg: boolean) => void
}
const TomesHeader = ({ setOpen }: TomesHeaderType) => {
    const [isAIModalOpen, setAIModalOpen] = useState(false);
    const [isAIAvailable, setIsAIAvailable] = useState(false);

    useEffect(() => {
        checkAI().then(setIsAIAvailable);
    }, []);

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                    label: "Tomes",
                    link: "/tomes"
                }]} />
                <div className="flex gap-2">
                    {isAIAvailable && (
                        <Button
                            buttonStyle={{ color: "purple", "size": "md" }}
                            leftIcon={<SparklesIcon className="h-4 w-4" />}
                            onClick={() => setAIModalOpen(true)}
                        >
                            Create with AI
                        </Button>
                    )}
                    <Button
                        buttonStyle={{ color: "purple", "size": "md" }}
                        leftIcon={<ArrowUpTrayIcon className="h-4 w-4" />}
                        onClick={() => setOpen(true)}
                    >
                        Import tome repository
                    </Button>
                </div>
            </div>
            {isAIModalOpen && <CreateAITomeModal isOpen={isAIModalOpen} setOpen={setAIModalOpen} />}
            <PageHeader title="Tomes">
                <>
                    <span>A tome is a prebuilt bundle, which includes execution instructions and files. Tomes are how beacon actions are defined. </span>
                    <a className="external-link" target="_blank" rel="noreferrer" href="https://docs.realm.pub/user-guide/tomes">Learn more</a>
                    <span> about how to write, test, and import tome repositories.</span>
                </>
            </PageHeader>
        </div>
    );
}
export default TomesHeader;
