import type { ReactNode } from "react";

type IconProps = { readonly size?: number; readonly className?: string };

const Stroke = ({ size = 22, className, children }: IconProps & { children: ReactNode }) => (
  <svg
    aria-hidden="true"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.4"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
  >
    {children}
  </svg>
);

export const HomeIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M3 10.5 12 3l9 7.5V21a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z" />
  </Stroke>
);
export const ChatIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
  </Stroke>
);
export const BookIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2z" />
    <path d="M22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7z" />
  </Stroke>
);
export const CheckSquareIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M9 11l3 3 8-8" />
    <path d="M20 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
  </Stroke>
);
export const CheckIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M5 12.5 10 17.5 19 7" />
  </Stroke>
);
export const BangIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M12 5v9" />
    <path d="M12 19h.01" />
  </Stroke>
);
export const ArrowLeftIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M19 12H5" />
    <path d="m11 6-6 6 6 6" />
  </Stroke>
);
export const ArrowRightIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M5 12h14" />
    <path d="m13 6 6 6-6 6" />
  </Stroke>
);
export const SendIcon = (p: IconProps) => (
  <Stroke {...p}>
    <path d="M22 2 11 13" />
    <path d="M22 2 15 22l-4-9-9-4z" />
  </Stroke>
);
export const LockIcon = (p: IconProps) => (
  <Stroke {...p}>
    <rect x="4" y="11" width="16" height="10" rx="2" />
    <path d="M8 11V7a4 4 0 0 1 8 0v4" />
  </Stroke>
);
export const BulbIcon = ({ size = 26, className }: IconProps) => (
  <svg
    aria-hidden="true"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="#b07d00"
    strokeWidth="2.2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
  >
    <path d="M9 18h6" />
    <path d="M10 22h4" />
    <path d="M12 2a7 7 0 0 0-4 12.7V17h8v-2.3A7 7 0 0 0 12 2z" />
  </svg>
);

const STAR_PATH = "m12 2 3 6.6 7 .8-5.2 4.8 1.5 7L12 17.6 5.7 21.2l1.5-7L2 9.4l7-.8z";

export const StarIcon = ({
  size = 18,
  fill = "#f5b301",
  stroke,
  className,
}: IconProps & { readonly fill?: string; readonly stroke?: string }) => (
  <svg
    aria-hidden="true"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill={fill}
    stroke={stroke ?? "none"}
    strokeWidth={stroke === undefined ? 0 : 1}
    className={className}
  >
    <path d={STAR_PATH} />
  </svg>
);

export const FlameIcon = ({ size = 18, className }: IconProps) => (
  <svg
    aria-hidden="true"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="#f08c1a"
    className={className}
  >
    <path d="M12 2c1 4 5 5.5 5 11a5 5 0 0 1-10 0c0-2.5 1.2-4 2.5-5 .3 2 1.5 3 2.5 3-1-3 0-6 0-9z" />
  </svg>
);

export const Spinner = ({ size = 20 }: IconProps) => (
  <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24" className="animate-spin">
    <circle
      cx="12"
      cy="12"
      r="9"
      fill="none"
      stroke="currentColor"
      strokeOpacity="0.3"
      strokeWidth="3"
    />
    <path
      d="M21 12a9 9 0 0 0-9-9"
      fill="none"
      stroke="currentColor"
      strokeWidth="3"
      strokeLinecap="round"
    />
  </svg>
);
