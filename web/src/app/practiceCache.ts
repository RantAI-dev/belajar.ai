import type { PracticeSetDto } from "../features/latihan";

// The set that just streamed in (and its graded copy after submit), so the next screen opens
// without a second request. A reload falls back to GET /v1/practice-sets/{id}.
const sets = new Map<string, PracticeSetDto>();

export const rememberSet = (s: PracticeSetDto): void => {
  sets.set(s.id, s);
};

export const rememberedSet = (id: string): PracticeSetDto | undefined => sets.get(id);
