import { Button } from "@/components/ui/button";

export default function Pager(props: {
  page: number;
  pageSize: number;
  total: number;
  onPageChange: (page: number) => void;
}) {
  const totalPages = Math.max(1, Math.ceil(props.total / props.pageSize));
  return (
    <div className="mt-6 flex items-center justify-center gap-4">
      <Button
        variant="outline"
        size="sm"
        disabled={props.page <= 1}
        onClick={() => props.onPageChange(props.page - 1)}
      >
        上一页
      </Button>
      <span className="text-[13px] text-muted-foreground tabular-nums">
        {props.page}/{totalPages} (total: {props.total})
      </span>
      <Button
        variant="outline"
        size="sm"
        disabled={props.page >= totalPages}
        onClick={() => props.onPageChange(props.page + 1)}
      >
        下一页
      </Button>
    </div>
  );
}
