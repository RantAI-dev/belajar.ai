import { cx } from "./cx";
import { AVATAR_COLORS, avatarColorIndex, initialsOf } from "./initials";

export function Avatar({
  name,
  size = 40,
  className,
}: {
  readonly name: string;
  readonly size?: number;
  readonly className?: string;
}) {
  const color = AVATAR_COLORS[avatarColorIndex(name)] ?? { bg: "#e0f0fd", fg: "#00598a" };
  return (
    <span
      aria-hidden="true"
      className={cx(
        "flex shrink-0 items-center justify-center rounded-full font-extrabold",
        className,
      )}
      style={{
        width: size,
        height: size,
        background: color.bg,
        color: color.fg,
        fontSize: size * 0.38,
      }}
    >
      {initialsOf(name) || "?"}
    </span>
  );
}
