import { Tag, type TagProps } from "antd";
import "../../styles/ui/display-chip.css";

export function DisplayChip({ className, ...props }: TagProps) {
  return (
    <Tag
      {...props}
      className={className ? `display-chip ${className}` : "display-chip"}
    />
  );
}
