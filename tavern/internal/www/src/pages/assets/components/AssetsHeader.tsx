import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import { FC } from "react";
import Button from "../../../components/tavern-base-ui/button/Button";

type AssetsHeaderProps = {
    setOpen: (arg: boolean) => void;
};

const AssetsHeader: FC<AssetsHeaderProps> = ({ setOpen }) => {
    return (
        <div className="md:flex md:items-center md:justify-between">
            <div className="min-w-0 flex-1">
                <h2 className="text-2xl font-bold leading-7 text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight">
                    Assets
                </h2>
            </div>
            <div className="mt-4 flex md:ml-4 md:mt-0">
                <Button
                    onClick={() => setOpen(true)}
                    buttonVariant="solid"
                    buttonStyle={{ color: "purple", size: "md" }}
                    leftIcon={<ArrowUpTrayIcon className="-ml-0.5 mr-1.5 h-5 w-5" />}
                >
                    Upload Asset
                </Button>
            </div>
        </div>
    );
};

export default AssetsHeader;
