import { FC, useState, useEffect } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useCreateLink } from "../useAssets";
import { format, add } from "date-fns";
import { Clipboard } from "lucide-react";
import { Checkbox, Tabs, TabList, TabPanels, Tab, TabPanel } from "@chakra-ui/react";

type CreateLinkModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    assetId: string;
    assetName: string;
    onSuccess?: () => void;
};

const generateRandomString = (length: number) => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    const randomValues = new Uint32Array(length);
    crypto.getRandomValues(randomValues);
    for (let i = 0; i < length; i++) {
        result += chars[randomValues[i] % chars.length];
    }
    return result;
};

const CreateLinkModal: FC<CreateLinkModalProps> = ({ isOpen, setOpen, assetId, assetName, onSuccess }) => {
    const { createLink, loading } = useCreateLink();
    const [downloadLimit, setDownloadLimit] = useState<number>(1);
    const [hasDownloadLimit, setHasDownloadLimit] = useState<boolean>(false);
    const [expiryMode, setExpiryMode] = useState<number>(0); // 0: 10m, 1: 1h, 2: Custom
    const [expiresAt, setExpiresAt] = useState<string>(
        format(new Date(Date.now() + 24 * 60 * 60 * 1000), "yyyy-MM-dd'T'HH:mm")
    );
    const [path, setPath] = useState<string>("");
    const [createdLink, setCreatedLink] = useState<string | null>(null);

    useEffect(() => {
        if (isOpen) {
            setPath(generateRandomString(12));
            setCreatedLink(null);
            setHasDownloadLimit(false);
            setDownloadLimit(1);
            setExpiryMode(0);
        }
    }, [isOpen]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        let finalExpiresAt = new Date();
        if (expiryMode === 0) {
            finalExpiresAt = add(new Date(), { minutes: 10 });
        } else if (expiryMode === 1) {
            finalExpiresAt = add(new Date(), { hours: 1 });
        } else {
            finalExpiresAt = new Date(expiresAt);
        }

        try {
            const { data } = await createLink({
                variables: {
                    input: {
                        assetID: assetId,
                        downloadLimit: hasDownloadLimit ? Number(downloadLimit) : null,
                        expiresAt: finalExpiresAt.toISOString(),
                        path: path,
                    },
                },
            });
            if (data?.createLink?.path) {
                const link = `${window.location.origin}/cdn/${data.createLink.path}`;
                setCreatedLink(link);
                if (onSuccess) {
                    onSuccess();
                }
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleCopy = () => {
        if (createdLink) {
            navigator.clipboard.writeText(createdLink);
        }
    };

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="md">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Create link for {assetName}
                    </h3>
                    <p className="mt-1 text-sm text-gray-500 font-normal">
                        Create a temporary download link for this asset.
                    </p>
                </div>

                {createdLink ? (
                    <div className="flex flex-col gap-4">
                        <h4 className="font-medium text-gray-900">Link Created!</h4>
                        <div className="flex flex-row items-center gap-2">
                            <input
                                type="text"
                                readOnly
                                value={createdLink}
                                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            />
                            <Button
                                onClick={handleCopy}
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                leftIcon={<Clipboard className="h-5 w-5" />}
                            >
                                Copy
                            </Button>
                        </div>
                        <Button
                            onClick={() => {
                                setCreatedLink(null);
                                setOpen(false);
                            }}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                        >
                            Done
                        </Button>
                    </div>
                ) : (
                    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                        <div>
                            <div className="flex items-center gap-2 mb-2">
                                <Checkbox
                                    isChecked={hasDownloadLimit}
                                    onChange={(e) => setHasDownloadLimit(e.target.checked)}
                                    colorScheme="purple"
                                >
                                    <span className="text-sm font-medium text-gray-700">Limit Downloads</span>
                                </Checkbox>
                            </div>

                            {hasDownloadLimit && (
                                <input
                                    type="number"
                                    min="1"
                                    required
                                    value={downloadLimit}
                                    onChange={(e) => setDownloadLimit(Number(e.target.value))}
                                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                                />
                            )}
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                                Expires In
                            </label>
                            <Tabs index={expiryMode} onChange={setExpiryMode} variant="soft-rounded" colorScheme="purple" size="sm">
                                <TabList>
                                    <Tab>10min</Tab>
                                    <Tab>1hr</Tab>
                                    <Tab>Custom</Tab>
                                </TabList>
                                <TabPanels>
                                    <TabPanel p={0} pt={2}>
                                        <p className="text-sm text-gray-500">Link will expire in 10 minutes.</p>
                                    </TabPanel>
                                    <TabPanel p={0} pt={2}>
                                        <p className="text-sm text-gray-500">Link will expire in 1 hour.</p>
                                    </TabPanel>
                                    <TabPanel p={0} pt={2}>
                                        <input
                                            type="datetime-local"
                                            required={expiryMode === 2}
                                            value={expiresAt}
                                            onChange={(e) => setExpiresAt(e.target.value)}
                                            className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                                        />
                                    </TabPanel>
                                </TabPanels>
                            </Tabs>
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700">
                                Path
                            </label>
                            <div className="flex rounded-md shadow-sm mt-1">
                                <span className="inline-flex items-center rounded-l-md border border-r-0 border-gray-300 bg-gray-50 px-3 text-gray-500 sm:text-sm">
                                    /cdn/
                                </span>
                                <input
                                    type="text"
                                    required
                                    value={path}
                                    onChange={(e) => setPath(e.target.value)}
                                    className="block w-full min-w-0 flex-1 rounded-none rounded-r-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                                />
                            </div>
                        </div>

                        <div className="flex justify-end gap-2 mt-4">
                            <Button
                                type="button"
                                onClick={() => setOpen(false)}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "md" }}
                            >
                                Cancel
                            </Button>
                            <Button
                                type="submit"
                                disabled={loading}
                                buttonVariant="solid"
                                buttonStyle={{ color: "purple", size: "md" }}
                            >
                                {loading ? "Creating..." : "Create Link"}
                            </Button>
                        </div>
                    </form>
                )}
            </div>
        </Modal>
    );
};

export default CreateLinkModal;
