export default function Pager(props: {
  page: number;
  pageSize: number;
  total: number;
  onPageChange: (page: number) => void;
}) {
  const totalPages = Math.max(1, Math.ceil(props.total / props.pageSize));
  return (
    <div className="mt-6 flex items-center justify-center gap-4">
      <button
        type="button"
        className="bg-white text-gray-900 border border-black/6 px-3 py-1.5 text-[13px] shadow-sm rounded-md transition-all duration-150 hover:border-black/15 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={props.page <= 1}
        onClick={() => props.onPageChange(props.page - 1)}
      >
        上一页
      </button>
      <span className="text-[13px] text-gray-500 tabular-nums">
        {props.page}/{totalPages} (total: {props.total})
      </span>
      <button
        type="button"
        className="bg-white text-gray-900 border border-black/6 px-3 py-1.5 text-[13px] shadow-sm rounded-md transition-all duration-150 hover:border-black/15 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={props.page >= totalPages}
        onClick={() => props.onPageChange(props.page + 1)}
      >
        下一页
      </button>
    </div>
  );
}
