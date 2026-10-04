import type { HTMLAttributes } from "react";
import "./skeleton.css";

type SkeletonProps = HTMLAttributes<HTMLSpanElement>;

export function Skeleton({ className = "", ...props }: SkeletonProps) {
  return (
    <span
      {...props}
      aria-hidden="true"
      className={`skeleton ${className}`.trim()}
    />
  );
}
