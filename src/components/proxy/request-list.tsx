import type { RequestSummary } from "../../types";
import { Badge } from "../ui/badge";
import { formatBytes, getMethodColor } from "../../lib/utils";
import clsx from "clsx";

interface RequestListProps {
  requests: RequestSummary[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  filterMethod: string;
  onFilterChange: (method: string) => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
}

export function RequestList({
  requests,
  selectedId,
  onSelect,
  filterMethod,
  onFilterChange,
  searchQuery,
  onSearchChange,
}: RequestListProps) {
  const filtered = requests.filter((req) => {
    const matchesMethod = !filterMethod || req.method === filterMethod;
    const matchesSearch =
      !searchQuery || req.url.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesMethod && matchesSearch;
  });

  if (requests.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-cursor-dark/40 text-sm">
        No requests captured yet
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Filters */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border-primary bg-surface-400">
        <input
          type="text"
          placeholder="Search URL..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="flex-1 bg-transparent text-xs text-cursor-dark placeholder:text-cursor-dark/30 focus:outline-none"
        />
        <select
          value={filterMethod}
          onChange={(e) => onFilterChange(e.target.value)}
          className="bg-surface-300 text-xs text-cursor-dark rounded-comfortable border border-border-primary px-2 py-0.5 focus:outline-none"
        >
          <option value="">All</option>
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
          <option value="PATCH">PATCH</option>
        </select>
      </div>

      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border-primary bg-surface-400 text-xs text-cursor-dark/55">
        <span className="w-16">Method</span>
        <span className="w-12">Status</span>
        <span className="flex-1">URL</span>
        <span className="w-20">Size</span>
        <span className="w-20">Time</span>
      </div>

      {/* Rows */}
      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="flex items-center justify-center h-full text-cursor-dark/30 text-xs">
            No matches
          </div>
        ) : (
          filtered.map((req) => (
            <RequestRow
              key={req.id}
              request={req}
              isSelected={selectedId === req.id}
              onClick={() => onSelect(req.id)}
            />
          ))
        )}
      </div>

      {/* Footer count */}
      <div className="px-3 py-1 border-t border-border-primary bg-surface-400 text-[10px] text-cursor-dark/40">
        {filtered.length} of {requests.length} requests
      </div>
    </div>
  );
}

function RequestRow({
  request,
  isSelected,
  onClick,
}: {
  request: RequestSummary;
  isSelected: boolean;
  onClick: () => void;
}) {
  const methodColor = getMethodColor(request.method);
  const statusColor = getStatusColor(request.status);

  return (
    <button
      onClick={onClick}
      className={clsx(
        "w-full flex items-center gap-2 px-3 py-1.5 text-xs border-b border-border-primary/30",
        "transition-colors duration-150 ease text-left",
        isSelected
          ? "bg-surface-300"
          : "hover:bg-surface-300/50"
      )}
    >
      <Badge color={methodColor} className="w-16 justify-center text-[10px]">
        {request.method}
      </Badge>
      <span
        className={clsx(
          "w-12 text-center font-mono",
          statusColor ? "text-cursor-dark" : "text-cursor-dark/30"
        )}
      >
        {request.status ?? "—"}
      </span>
      <span className="flex-1 truncate text-cursor-dark/70 font-mono">
        {request.url}
      </span>
      <span className="w-20 text-right text-cursor-dark/40 font-mono">
        {formatBytes(request.content_length)}
      </span>
      <span className="w-20 text-right text-cursor-dark/40 font-mono">
        {request.duration_ms ? `${request.duration_ms}ms` : "—"}
      </span>
    </button>
  );
}

function getStatusColor(status: number | null): string {
  if (!status) return "text-cursor-dark/30";
  if (status >= 200 && status < 300) return "text-success";
  if (status >= 300 && status < 400) return "text-grep";
  if (status >= 400 && status < 500) return "text-thinking";
  if (status >= 500) return "text-error";
  return "text-cursor-dark/55";
}
