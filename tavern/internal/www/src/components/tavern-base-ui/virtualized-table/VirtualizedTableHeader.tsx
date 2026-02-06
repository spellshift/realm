import { VirtualizedTableHeaderProps } from "./types";

export const VirtualizedTableHeader = ({
    columns,
    gridCols,
    minWidth = '800px',
    className = '',
}: VirtualizedTableHeaderProps) => {
    return (
        <div
            className={`bg-gray-50 sticky top-0 z-10 grid gap-4 px-6 py-3 border-b border-gray-200 ${className}`}
            style={{
                gridTemplateColumns: gridCols,
                minWidth,
            }}
        >
            {columns.map((column, index) => (
                <div
                    key={index}
                    className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                    {column}
                </div>
            ))}
        </div>
    );
};
