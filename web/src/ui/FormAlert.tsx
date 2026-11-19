import { cx } from "./cx";

/** One sentence about a whole form: an error (read out at once) or a success notice. */
export function FormAlert({
  tone,
  children,
}: {
  readonly tone: "error" | "success";
  readonly children: string;
}) {
  return (
    <p
      role={tone === "error" ? "alert" : "status"}
      className={cx(
        "m-0 rounded-[14px] px-4 py-3 text-[16px]",
        tone === "error"
          ? "bg-oops-soft font-bold text-oops-ink"
          : "bg-success-soft font-extrabold text-success-edge",
      )}
    >
      {children}
    </p>
  );
}
