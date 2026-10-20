import DOMPurify from "dompurify";

export function sanitizePreparedHtml(html: string) {
  return DOMPurify.sanitize(html, {
    FORBID_TAGS: ["script", "iframe", "object", "embed", "form"],
    FORBID_ATTR: ["style"],
    ALLOW_DATA_ATTR: true,
  });
}
