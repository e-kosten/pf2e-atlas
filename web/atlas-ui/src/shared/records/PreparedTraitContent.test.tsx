import { fireEvent, render, screen } from "@testing-library/react";
import { PreparedContentBody } from "./PreparedContent";

it("renders prepared definition references with their checked destination and strips unsafe markup", () => {
  const open = vi.fn();
  const target = {
    record_key: "conditionitems:stupefied",
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  const { container } = render(
    <PreparedContentBody
      body={{
        kind: "html",
        html: '<p>Become <span data-atlas-reference="0">Stupefied 1</span>.</p><img src="x" onerror="alert(1)"><script>alert(1)</script>',
        controls: [],
      }}
      references={[{ ordinal: 0, target, status: "resolved_record", url: null }]}
      onReference={open}
    />,
  );
  const link = screen.getByRole("link", { name: "Stupefied 1" });
  expect(link).toHaveAttribute(
    "href",
    expect.stringContaining("/records/conditionitems:stupefied"),
  );
  fireEvent.click(link);
  expect(open).toHaveBeenCalledWith(target.record_key, link, target);
  expect(container.querySelector("script")).toBeNull();
  expect(container.querySelector("img")).not.toHaveAttribute("onerror");
});
