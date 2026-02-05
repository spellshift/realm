import { Cursor, QueryPageInfo } from "../../../utils/interfacesQuery";
import Button from "../button/Button";

type Props = {
  totalCount: number;
  pageInfo: QueryPageInfo;
  refetchTable: (endCursor: Cursor, startCursor: Cursor) => void;
  page: number;
  setPage: any;
  rowLimit: number;
  loading?: boolean;
}
export default function TablePagination(props: Props) {
  const { totalCount, pageInfo, refetchTable, page, setPage, rowLimit, loading = false } = props;

  function handlePreviousClick() {
    if (page <= 1) return;

    setPage((prevPage: number) => Math.max(1, prevPage - 1));
    if (refetchTable && pageInfo.startCursor) {
      refetchTable(null, pageInfo.startCursor);
    }
  }

  function handleNextClick() {
    const maxPage = getPageCount();
    if (page >= maxPage) return;

    setPage((prevPage: number) => Math.min(maxPage, prevPage + 1));
    if (refetchTable && pageInfo.endCursor) {
      refetchTable(pageInfo.endCursor, null);
    }
  }

  const getPageCount = () => {
    return Math.ceil(totalCount / rowLimit);
  }

  return (
    <nav
      className="sticky bottom-0 z-5 flex items-center justify-between border-t border-gray-200 bg-white px-4 sm:px-6 xl:px-8 py-3 shadow-md z-5"
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
          disabled={page <= 1 || loading}
          onClick={() => handlePreviousClick()}
        >
          Previous
        </Button>
        <Button
          buttonVariant="outline"
          disabled={page >= getPageCount() || loading}
          buttonStyle={{ color: 'gray', size: "md" }}
          onClick={() => handleNextClick()}
        >
          Next
        </Button>
      </div>
    </nav>
  )
};
