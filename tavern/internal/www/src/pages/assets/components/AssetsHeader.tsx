import { Upload } from "lucide-react";
import { BugAntIcon } from "@heroicons/react/24/outline";
import { FC } from "react";
import { useNavigate } from "react-router-dom";
import Button from "../../../components/tavern-base-ui/button/Button";
import Breadcrumbs from "../../../components/Breadcrumbs";

type AssetsHeaderProps = {
    setOpen: (arg: boolean) => void;
    onGenerateAgent?: () => void;
};

const AssetsHeader: FC<AssetsHeaderProps> = ({ setOpen, onGenerateAgent }) => {
    const navigate = useNavigate();
    return (
        <div className="flex flex-col gap-4 justify-between">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs
                    pages={[{
                        label: "Assets",
                        link: "/assets"
                    }]}
                />
                <div className="flex items-center gap-2">
                    <Button
                        onClick={onGenerateAgent ? onGenerateAgent : () => navigate("/assets/build")}
                        buttonVariant="solid"
                        buttonStyle={{ color: "purple", size: "md" }}
                        leftIcon={<BugAntIcon className="-ml-0.5 mr-1.5 h-5 w-5" />}
                    >
                        Generate agent
                    </Button>
                    <Button
                        onClick={() => setOpen(true)}
                        buttonVariant="outline"
                        buttonStyle={{ color: "gray", size: "md" }}
                        leftIcon={<Upload className="-ml-0.5 mr-1.5 h-5 w-5" />}
                    >
                        Upload Assets
                    </Button>
                </div>
            </div>
        </div>
    );
};

export default AssetsHeader;
