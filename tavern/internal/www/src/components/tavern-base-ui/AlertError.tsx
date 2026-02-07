import { XCircleIcon } from '@heroicons/react/20/solid'
import { FC } from 'react'

type AlertErrorProps = {
    label: string,
    details: string
}
const AlertError: FC<AlertErrorProps> = ({ label, details }) => {
    return (
        <div className="rounded-md bg-red-50 py-2 px-4">
            <div className="flex flex-row gap-4">
                <div className="flex-shrink-0 mt-2">
                    <XCircleIcon className="h-6 w-6 text-red-400" aria-hidden="true" />
                </div>
                <div className="flex flex-col">
                    <h3 className="text-sm font-semibold text-red-700">{label}</h3>
                    <div className="text-sm text-red-700">
                        {details}
                    </div>
                </div>
            </div>
        </div>
    )
}
export default AlertError;
