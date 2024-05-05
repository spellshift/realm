import Button from "./button/Button";

type PageInfo = {
  hasNextPage: boolean,
  hasPreviousPage: boolean,
  startCursor: string,
  endCursor: string
}
type Props = {
  totalCount: number;
  pageInfo: PageInfo;
  refetchTable: (endCursor: string | undefined, startCursor: string | undefined) => void;
  page: number;
  setPage: any;
  rowLimit: number;
}
export default function TablePagination(props: Props) {
  const { totalCount, pageInfo, refetchTable, page, setPage, rowLimit } = props;

  function handlePreviousClick() {
    if (refetchTable && pageInfo.hasPreviousPage) {
      setPage((page: number) => page - 1);
      refetchTable(undefined, pageInfo.startCursor);
    }
  }
  function handleNextClick() {
    if (refetchTable && pageInfo.hasNextPage) {
      setPage((page: number) => page + 1);
      refetchTable(pageInfo.endCursor, undefined);
    }
  }
  const getPageCount = () => {
    return Math.ceil(totalCount / rowLimit);
  }

  return (
    <nav
      className="flex items-center justify-between border-t border-gray-200 bg-white px-4 py-3 sm:px-6"
      aria-label="Pagination"
    >
      <div className="hidden sm:block">
        <p className="text-sm text-gray-800">
          Page <span className="font-semibold">{page}</span> of <span className="font-semibold">{getPageCount()}</span> {`(${totalCount} results)`}
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
