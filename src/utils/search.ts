export function matchesQuery(query: string, ...parts: Array<string | number | null | undefined>) {
  const keyword = query.trim().toLocaleLowerCase();
  if (keyword.length === 0) {
    return true;
  }
  return parts.some((part) =>
    String(part ?? '')
      .toLocaleLowerCase()
      .includes(keyword)
  );
}
