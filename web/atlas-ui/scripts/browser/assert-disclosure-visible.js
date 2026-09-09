/* global document, setTimeout, getComputedStyle, location, navigator */
/* eslint-disable @typescript-eslint/no-unused-expressions -- Safari evaluates this browser probe as a function expression. */
// Run in the real Safari document after pointer or keyboard activation.
// Remove the final semicolon and append ({ label: "...", expectedText: "..." }).
// This intentionally uses real layout; jsdom cannot supply this evidence.
(async function assertDisclosureVisible({ label, expectedText }) {
  if (!label || !expectedText)
    throw new Error("Exact disclosure label and body text required");
  await new Promise((resolve) => setTimeout(resolve, 2000));
  const headers = [
    ...document.querySelectorAll(".detail-panel .ant-collapse-header"),
  ].filter(
    (header) =>
      header
        .querySelector(".ant-collapse-header-text")
        ?.textContent?.replace(/\s+/g, " ")
        .trim() === label,
  );
  if (headers.length !== 1)
    throw new Error(`Expected one disclosure ${label}; found ${headers.length}`);
  const header = headers[0];
  const content = header
    .closest(".ant-collapse-item")
    ?.querySelector(":scope > .ant-collapse-content");
  if (!content) throw new Error(`No body for disclosure ${label}`);
  const rect = content.getBoundingClientRect();
  const style = getComputedStyle(content);
  const visibleText = content.innerText.replace(/\s+/g, " ").trim();
  let hiddenAncestor = false;
  for (let element = content; element; element = element.parentElement) {
    const computed = getComputedStyle(element);
    if (
      computed.display === "none" ||
      computed.visibility !== "visible" ||
      Number(computed.opacity) === 0
    ) {
      hiddenAncestor = true;
      break;
    }
  }
  const evidence = {
    label,
    expectedText,
    expanded: header.getAttribute("aria-expanded"),
    height: rect.height,
    width: rect.width,
    scrollHeight: content.scrollHeight,
    opacity: style.opacity,
    hiddenAncestor,
    className: content.className,
    visibleText,
    url: location.href,
    userAgent: navigator.userAgent,
  };
  if (
    evidence.expanded !== "true" ||
    rect.height <= 0 ||
    rect.width <= 0 ||
    hiddenAncestor ||
    !visibleText.includes(expectedText) ||
    /ant-motion-collapse-.*-start/.test(content.className)
  ) {
    throw new Error(`Disclosure is not visibly expanded: ${JSON.stringify(evidence)}`);
  }
  return evidence;
});
