import { ReactNode } from "react"

type PageHeaderType = {
    title: string,
    description?: string
    children?: ReactNode
}
const PageHeader = ({ title, description, children }: PageHeaderType) => {
    return (
        <div className="border-b border-gray-200 pb-5 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex-1 flex flex-col gap-2">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">{title}</h3>
                {description &&
                    <div className="max-w-2xl text-sm">
                        {description}
                    </div>
                }
                {children && <div className="max-w-2xl text-sm">{children}</div>}
            </div>
        </div>
    )
}
export default PageHeader;
