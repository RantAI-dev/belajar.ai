/** `/latihan?mapel=ipa&topik=…`: Latihan's setup with the subject and topic filled in. */
export const latihanHref = (subjectId: string | null, topic: string): string => {
  const q = new URLSearchParams();
  if (subjectId !== null) q.set("mapel", subjectId);
  if (topic !== "") q.set("topik", topic);
  const s = q.toString();
  return s === "" ? "/latihan" : `/latihan?${s}`;
};
