import { HomeIcon } from '@heroicons/react/20/solid'
import { Link } from 'react-router-dom'

export default function Breadcrumbs({ pages }: { pages: Array<{ label: string | undefined, link: string | undefined }> }) {
    return (
        <nav aria-label="Breadcrumb" className="flex">
            <ol className="flex space-x-4 py-4">
                <li className="flex">
                    <div className="flex items-center">
                        <Link to="/" className="text-gray-400 hover:text-gray-500">
                            <HomeIcon aria-hidden="true" className="h-5 w-5 flex-shrink-0" />
                            <span className="sr-only">Home</span>
                        </Link>
                    </div>
                </li>
                {pages.map((page, index) => (
                    <li key={`${page.label}-${index}`} className="flex">
                        <div className="flex items-center">
                            <svg
                                fill="currentColor"
                                viewBox="0 0 24 44"
                                preserveAspectRatio="none"
                                aria-hidden="true"
                                className="h-full w-2 flex-shrink-0 text-gray-400"
                            >
                                <path d="M.293 0l22 22-22 22h1.414l22-22-22-22H.293z" />
                            </svg>
                            <Link
                                to={page.link || ""}
                                aria-current={(pages.length - 1 === index) ? 'page' : undefined}
                                className="ml-4 text-sm font-medium text-gray-500 hover:text-gray-700"
                            >{page.label}
                            </Link>
                        </div>
                    </li>
                ))}
            </ol>
        </nav>
    )
}
