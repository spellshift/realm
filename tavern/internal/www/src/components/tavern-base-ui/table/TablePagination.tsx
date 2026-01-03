import { Cursor, QueryPageInfo } from "../../../utils/interfacesQuery";
import Button from "../button/Button";

type Props = {
  totalCount: number;
  pageInfo: QueryPageInfo;
  refetchTable: (endCursor: Cursor, startCursor: Cursor) => void;
  page: number;
  setPage: any;
  rowLimit: number;
}
export default function TablePagination(props: Props) {
  const { totalCount, pageInfo, refetchTable, page, setPage, rowLimit } = props;

  function handlePreviousClick() {
    if (refetchTable && pageInfo.hasPreviousPage) {
      setPage((page: number) => page - 1);
      refetchTable(null, pageInfo.startCursor);
    }
  }
  function handleNextClick() {
    if (refetchTable && pageInfo.hasNextPage) {
      setPage((page: number) => page + 1);
      refetchTable(pageInfo.endCursor, null);
    }
  }
  const getPageCount = () => {
    return Math.ceil(totalCount / rowLimit);
  }

  return (
    <nav
      className="sticky bottom-0 z-10 flex items-center justify-between border-t border-gray-200 bg-white px-4 sm:px-6 xl:px-8 py-3 shadow-md"
      aria-label="Pagination"
    >
      <div className="hidden sm:block">
        <p className="text-sm text-gray-800">
          <span className="hidden md:inline">Page </span>
          <span className="font-semibold">{page}</span>
          <span className="hidden md:inline"> of </span>
          <span className="font-semibold hidden md:inline">{getPageCount()}</span>
          <span className="md:hidden font-semibold">/{getPageCount()}</span>
          <span className="hidden lg:inline"> ({totalCount} results)</span>
        </p>
      </div>
      <div className="flex flex-1 justify-between sm:justify-end gap-2">
        <Button
          buttonVariant="outline"
          buttonStyle={{ color: 'gray', size: "md" }}
          disabled={!pageInfo.hasPreviousPage}
          onClick={() => handlePreviousClick()}
        >
          Previous
        </Button>
        <Button
          buttonVariant="outline"
          disabled={!pageInfo.hasNextPage}
          buttonStyle={{ color: 'gray', size: "md" }}
          onClick={() => handleNextClick()}
        >
          Next
        </Button>
      </div>
    </nav>
  )
};
