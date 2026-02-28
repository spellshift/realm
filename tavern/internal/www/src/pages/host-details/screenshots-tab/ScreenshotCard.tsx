import { FC, useState } from "react";
import { Dialog, DialogPanel } from "@headlessui/react";
import { formatDistanceToNow, parseISO } from "date-fns";

interface ScreenshotNode {
    id: string;
    createdAt: string;
}

interface ScreenshotCardProps {
    screenshot: ScreenshotNode;
}

export const ScreenshotCard: FC<ScreenshotCardProps> = ({ screenshot }) => {
    const [isOpen, setIsOpen] = useState(false);
    const imageUrl = `/cdn/screenshots/${screenshot.id}`;

    const parsedDate = parseISO(screenshot.createdAt);
    let timeAgo = 'Unknown';
    if (!isNaN(parsedDate.getTime())) {
        try {
            timeAgo = formatDistanceToNow(parsedDate, { addSuffix: true });
        } catch (e) {
            console.error("Error formatting date", e);
        }
    }

    return (
        <>
            <div className="rounded-lg shadow border-gray-200 border bg-white overflow-hidden flex flex-col h-full hover:shadow-md transition-shadow">
                <div className="flex flex-row justify-between items-center p-3 bg-gray-50 border-b border-gray-200">
                    <div className="font-semibold text-gray-700 text-sm">
                        Screenshot
                    </div>
                    <div className="text-sm text-gray-500" title={screenshot.createdAt}>
                        {timeAgo}
                    </div>
                </div>
                <div
                    className="flex-1 bg-black flex items-center justify-center overflow-hidden cursor-pointer"
                    onClick={() => setIsOpen(true)}
                >
                    <img
                        src={imageUrl}
                        alt="Screenshot"
                        className="object-contain w-full h-full max-h-[600px]"
                        loading="lazy"
                    />
                </div>
            </div>

            <Dialog open={isOpen} onClose={() => setIsOpen(false)} className="relative z-50">
                {/* Backdrop */}
                <div className="fixed inset-0 bg-black/80" aria-hidden="true" />

                <div className="fixed inset-0 flex items-center justify-center p-4">
                    <DialogPanel className="max-w-[90vw] max-h-[90vh] flex flex-col bg-transparent">
                        <img
                            src={imageUrl}
                            alt="Screenshot Fullscreen"
                            className="object-contain max-w-full max-h-[90vh] cursor-pointer"
                            onClick={() => setIsOpen(false)}
                        />
                        <div className="absolute top-4 right-4 text-white bg-black/50 px-3 py-1 rounded">
                            {timeAgo} (Click anywhere to close)
                        </div>
                    </DialogPanel>
                </div>
            </Dialog>
        </>
    );
};
