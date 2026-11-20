import { buttonClass, Card, cx, FormAlert } from "../../ui";
import { GoogleMark } from "../auth";
import { GOOGLE_LINK_HREF, type LinkNotice } from "./googleLink";

export type GoogleLinkSettingProps = {
  readonly isLinked: boolean;
  readonly isEnabled: boolean;
  readonly notice: LinkNotice | undefined;
};

/**
 * Google in Akun: linked, or a button to link it. Hidden entirely when the server has no Google
 * and there is nothing to say. Unlinking is not offered.
 */
export function GoogleLinkSetting({ isLinked, isEnabled, notice }: GoogleLinkSettingProps) {
  if (!isLinked && !isEnabled && notice === undefined) return null;
  return (
    <Card as="section" aria-labelledby="google-title" className="flex flex-col gap-4">
      <h2 id="google-title" className="m-0 font-display text-[22px] font-semibold">
        Akun Google
      </h2>
      {notice !== undefined && <FormAlert tone={notice.tone}>{notice.text}</FormAlert>}
      {isLinked ? (
        <p className="m-0 flex items-center gap-3 text-[17px] font-bold">
          <GoogleMark />
          Terhubung dengan Google
        </p>
      ) : (
        isEnabled && (
          <>
            <p className="m-0 text-[16px] text-ink-muted">
              Hubungkan akun Google supaya kamu bisa masuk dengan Google juga.
            </p>
            {/* A plain link on purpose: the server answers with a redirect to Google. */}
            <a
              href={GOOGLE_LINK_HREF}
              className={cx(buttonClass({ variant: "secondary" }), "self-start")}
            >
              <GoogleMark />
              <span>Hubungkan dengan Google</span>
            </a>
          </>
        )
      )}
    </Card>
  );
}
