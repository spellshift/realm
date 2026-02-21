import { Steps, Checkbox, Tabs } from "@chakra-ui/react";
import Button from "../../../../components/tavern-base-ui/button/Button";

const CreateLinkForm = ({ formik, setOpen }: any) => (
    <form onSubmit={formik.handleSubmit} className="flex flex-col gap-4">
        <div>
            <div className="flex items-center gap-2 mb-2">
                <Checkbox.Root
                    onCheckedChange={(e: any) => formik.setFieldValue("hasDownloadLimit", (e.checked as boolean))}
                    colorPalette="purple"
                    checked={formik.values.hasDownloadLimit}
                ><Checkbox.HiddenInput /><Checkbox.Control><Checkbox.Indicator /></Checkbox.Control><Checkbox.Label>
                    <span className="text-sm font-medium text-gray-700">Limit Downloads</span>
                </Checkbox.Label></Checkbox.Root>
            </div>

            {formik.values.hasDownloadLimit && (
                <div>
                    <input
                        type="number"
                        min="1"
                        name="downloadLimit"
                        required
                        value={formik.values.downloadLimit}
                        onChange={formik.handleChange}
                        onBlur={formik.handleBlur}
                        className={`mt-1 block w-full rounded-md shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm p-2 border ${formik.touched.downloadLimit && formik.errors.downloadLimit ? "border-red-500" : "border-gray-300"
                            }`}
                    />
                    {formik.touched.downloadLimit && formik.errors.downloadLimit && (
                        <p className="mt-1 text-sm text-red-600">{formik.errors.downloadLimit}</p>
                    )}
                </div>
            )}
        </div>

        <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
                Expires In
            </label>
            <Tabs.Root
                value={String(formik.values.expiryMode)}
                onValueChange={(details: any) => formik.setFieldValue("expiryMode", Number(details.value))}
                variant='subtle'
                colorPalette="purple"
                size="sm"
            >
                <Tabs.List>
                    <Tabs.Trigger value="0">10min</Tabs.Trigger>
                    <Tabs.Trigger value="1">1hr</Tabs.Trigger>
                    <Tabs.Trigger value="2">Custom</Tabs.Trigger>
                </Tabs.List>
                <Tabs.Content value="0" p={0} pt={2}>
                    <p className="text-sm text-gray-500">Link will expire in 10 minutes.</p>
                </Tabs.Content>
                <Tabs.Content value="1" p={0} pt={2}>
                    <p className="text-sm text-gray-500">Link will expire in 1 hour.</p>
                </Tabs.Content>
                <Tabs.Content value="2" p={0} pt={2}>
                    <input
                        type="datetime-local"
                        name="expiresAt"
                        required={formik.values.expiryMode === 2}
                        value={formik.values.expiresAt}
                        onChange={formik.handleChange}
                        onBlur={formik.handleBlur}
                        className={`block w-full rounded-md shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm p-2 border ${formik.touched.expiresAt && formik.errors.expiresAt ? "border-red-500" : "border-gray-300"
                            }`}
                    />
                    {formik.touched.expiresAt && formik.errors.expiresAt && (
                        <p className="mt-1 text-sm text-red-600">{formik.errors.expiresAt}</p>
                    )}
                </Tabs.Content>
            </Tabs.Root>
        </div>

        <div>
            <label htmlFor="path" className="block text-sm font-medium text-gray-700">
                Path
            </label>
            <div className="flex rounded-md shadow-sm mt-1">
                <span className="inline-flex items-center rounded-l-md border border-r-0 border-gray-300 bg-gray-50 px-3 text-gray-500 sm:text-sm">
                    /cdn/
                </span>
                <input
                    type="text"
                    id="path"
                    name="path"
                    required
                    value={formik.values.path}
                    onChange={formik.handleChange}
                    onBlur={formik.handleBlur}
                    className={`block w-full min-w-0 flex-1 rounded-none rounded-r-md focus:border-purple-500 focus:ring-purple-500 sm:text-sm p-2 border ${formik.touched.path && formik.errors.path ? "border-red-500" : "border-gray-300"
                        }`}
                />
            </div>
            {formik.touched.path && formik.errors.path && (
                <p className="mt-1 text-sm text-red-600">{formik.errors.path}</p>
            )}
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
                disabled={formik.isSubmitting}
                buttonVariant="solid"
                buttonStyle={{ color: "purple", size: "md" }}
            >
                {formik.isSubmitting ? "Creating..." : "Create Link"}
            </Button>
        </div>
    </form>);

export default CreateLinkForm;
