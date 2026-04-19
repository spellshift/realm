import { useState } from "react";
import { ArrowDownToLine } from "lucide-react";
import { Tooltip, useToast } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";

type DownloadButtonProps = {
    assetName: string;
};

const DownloadButton = ({ assetName }: DownloadButtonProps) => {
    const [isDownloading, setIsDownloading] = useState(false);
    const toast = useToast();

    const handleDownload = async (e: React.MouseEvent) => {
        e.stopPropagation();

        if (isDownloading) {
            return;
        }

        setIsDownloading(true);
        try {
            const response = await fetch(`/assets/download/${assetName}`);
            if (!response.ok) {
                throw new Error(`Download failed: ${response.statusText}`);
            }

            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const link = document.createElement("a");
            link.href = url;
            link.download = assetName;
            document.body.appendChild(link);
            link.click();
            link.remove();
            window.URL.revokeObjectURL(url);
        } catch (err: any) {
            toast({
                title: "Download failed",
                description: err.message,
                status: "error",
                duration: 4000,
                isClosable: true,
            });
        } finally {
            setIsDownloading(false);
        }
    };

    return (
        <Tooltip label="Download" bg="white" color="black">
            <Button
                buttonVariant="ghost"
                buttonStyle={{ color: "gray", size: "xs" }}
                leftIcon={<ArrowDownToLine className="w-4 h-4" />}
                isLoading={isDownloading}
                onClick={handleDownload}
                aria-label="Download"
            />
        </Tooltip>
    );
};

export default DownloadButton;
