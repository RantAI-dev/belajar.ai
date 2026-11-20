/** The query flag that makes Latihan's setup start generating as soon as it opens. */
export const AUTOSTART_PARAM = "mulai";

/**
 * `/latihan?mapel=ipa&topik=…`: Latihan's setup with the subject and topic filled in. With
 * `start`, the setup also begins making the set straight away (one tap from Progres).
 */
export const latihanHref = (
  subjectId: string | null,
  topic: string,
  options: { readonly start?: boolean } = {},
): string => {
  const q = new URLSearchParams();
  if (subjectId !== null) q.set("mapel", subjectId);
  if (topic !== "") q.set("topik", topic);
  if (options.start === true && topic !== "") q.set(AUTOSTART_PARAM, "1");
  const s = q.toString();
  return s === "" ? "/latihan" : `/latihan?${s}`;
};

/** Whether a setup opened from `params` should start generating by itself. */
export const shouldAutostart = (params: URLSearchParams): boolean =>
  params.get(AUTOSTART_PARAM) === "1" && (params.get("topik") ?? "").trim() !== "";
