export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function getMethodColor(method: string): "grep" | "read" | "edit" | "thinking" | "error" | "default" {
  switch (method.toUpperCase()) {
    case "GET": return "grep";
    case "POST": return "read";
    case "PUT": return "edit";
    case "DELETE": return "error";
    case "PATCH": return "thinking";
    default: return "default";
  }
}
