import Button from "../../../../components/tavern-base-ui/button/Button";
import { Clipboard } from "lucide-react";

interface LinkCreatedProps {
    createdLink: string;
    close: () => void;
}

const LinkCreated = ({ createdLink, close }: LinkCreatedProps) => (
    <div className="flex flex-col gap-4">
        <h4 className="font-medium text-gray-900">Link Created</h4>
        <div className="flex flex-col gap-2">
            <p className="text-sm text-gray-700 break-all select-all">
                {createdLink}
            </p>
        </div>
        <div className="flex justify-end gap-2 mt-2">
            <Button
                onClick={close}
                buttonVariant="outline"
                buttonStyle={{ color: "gray", size: "md" }}
            >
                Close
            </Button>
            <Button
                onClick={() => {
                    navigator.clipboard.writeText(createdLink);
                    close();
                }}
                buttonVariant="solid"
                buttonStyle={{ color: "purple", size: "md" }}
                leftIcon={<Clipboard className="h-4 w-4" />}
            >
                Copy & Close
            </Button>
        </div>
    </div>);

export default LinkCreated;
