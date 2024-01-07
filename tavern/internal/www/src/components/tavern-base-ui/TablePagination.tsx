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
    const {totalCount, pageInfo, refetchTable, page, setPage, rowLimit} = props;

    function handlePreviousClick(){
        if(refetchTable && pageInfo.hasPreviousPage){
            setPage((page:number)=> page-1);
            refetchTable(undefined, pageInfo.startCursor);
        }
    }
    function handleNextClick(){
        if(refetchTable && pageInfo.hasNextPage){
            setPage((page:number)=> page+1);
            refetchTable( pageInfo.endCursor, undefined);
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
        <div className="flex flex-1 justify-between sm:justify-end">
          <button
            disabled={!pageInfo.hasPreviousPage}
            onClick={()=>handlePreviousClick()}
            className="relative inline-flex items-center rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus-visible:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Previous
          </button>
          <button
            disabled={!pageInfo.hasNextPage}
            onClick={()=> handleNextClick()}
            className="relative ml-3 inline-flex items-center rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus-visible:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
          </button>
        </div>
      </nav>
    )
};