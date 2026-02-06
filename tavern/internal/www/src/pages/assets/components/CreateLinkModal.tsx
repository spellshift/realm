import { FC, useState } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import { useCreateLink } from "../useAssets";
import { format } from "date-fns";
import { Clipboard } from "lucide-react";

type CreateLinkModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    assetId: string;
    assetName: string;
};

const CreateLinkModal: FC<CreateLinkModalProps> = ({ isOpen, setOpen, assetId, assetName }) => {
    const { createLink, loading } = useCreateLink();
    const [downloadsRemaining, setDownloadsRemaining] = useState<number>(1);
    const [expiresAt, setExpiresAt] = useState<string>(
        format(new Date(Date.now() + 24 * 60 * 60 * 1000), "yyyy-MM-dd'T'HH:mm")
    );
    const [createdLink, setCreatedLink] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            const { data } = await createLink({
                variables: {
                    input: {
                        assetID: assetId,
                        downloadsRemaining: Number(downloadsRemaining),
                        expiresAt: new Date(expiresAt).toISOString(),
                    },
                },
            });
            if (data?.createLink?.path) {
                const link = `${window.location.origin}/cdn/${data.createLink.path}`;
                setCreatedLink(link);
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
                            <label className="block text-sm font-medium text-gray-700">
                                Downloads Remaining
                            </label>
                            <input
                                type="number"
                                min="1"
                                required
                                value={downloadsRemaining}
                                onChange={(e) => setDownloadsRemaining(Number(e.target.value))}
                                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            />
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700">
                                Expires At
                            </label>
                            <input
                                type="datetime-local"
                                required
                                value={expiresAt}
                                onChange={(e) => setExpiresAt(e.target.value)}
                                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            />
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
