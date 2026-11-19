import { Button } from "./Button";
import { Mascot } from "./Mascot";

/** The mascot thinking while a screen's data arrives. */
export function LoadingState({ text = "Sebentar, aku siapkan dulu…" }: { readonly text?: string }) {
  return (
    <div
      role="status"
      className="flex flex-col items-center gap-3 py-10 text-center text-[17px] font-bold text-ink-muted"
    >
      <Mascot mood="thinking" size={96} animate />
      <span>{text}</span>
    </div>
  );
}

/** A friendly failure with a way to try again. */
export function ErrorState({
  message,
  onRetry,
}: {
  readonly message: string;
  readonly onRetry?: () => void;
}) {
  return (
    <div role="alert" className="flex flex-col items-center gap-4 py-10 text-center">
      <Mascot mood="hmm" size={96} />
      <p className="m-0 max-w-[460px] text-[17px] font-bold">{message}</p>
      {onRetry !== undefined && (
        <Button size="lg" onClick={onRetry}>
          Coba lagi
        </Button>
      )}
    </div>
  );
}
