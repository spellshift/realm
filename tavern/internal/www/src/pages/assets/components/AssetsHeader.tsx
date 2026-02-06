import { Upload } from "lucide-react";
import { FC } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";
import Breadcrumbs from "../../../components/Breadcrumbs";

type AssetsHeaderProps = {
    setOpen: (arg: boolean) => void;
};

const AssetsHeader: FC<AssetsHeaderProps> = ({ setOpen }) => {
    return (
        <div className="flex flex-col gap-4 justify-between">
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs pages={[{
                                    label: "Assets",
                                    link: "/assets"
                                }]}
                />
                <div>
                <Button
                    onClick={() => setOpen(true)}
                    buttonVariant="solid"
                    buttonStyle={{ color: "purple", size: "md" }}
                    leftIcon={<Upload className="-ml-0.5 mr-1.5 h-5 w-5" />}
                >
                    Upload Asset
                    </Button>
                </div>
            </div>
        </div>
    );
};

export default AssetsHeader;
