// Ported from HighlightedText.swift's `fuzzyMatch`/`bestScore` -- used by the sessions store's
// own sidebar search (M2-14) and, per docs/spec/session-views.md §3, by the sessions list's
// further local search too (M2-15), so this is a shared module rather than inlined into either.
// Only the matching/scoring half is ported here, not `attributedString`/`fuzzyAttributedString`:
// those exist purely to highlight matched characters in rendered text, a concern for whichever
// task actually renders a highlighted row, not for a Pinia store with no template of its own.

export interface FuzzyResult {
  indices: number[];
  score: number;
}

const WORD_BOUNDARY = new Set([' ', '-', '/', '.', '_']);

/** Fuzzy-matches `query` against `text` case-insensitively. `null` means no match at all (either
 * the query's characters don't appear in order, or the match was too scattered to be useful). */
export function fuzzyMatch(text: string, query: string): FuzzyResult | null {
  if (query.length === 0) return { indices: [], score: 0 };

  // A single character requires an exact substring hit, not a fuzzy scatter match.
  if (query.length < 2) {
    const index = text.toLowerCase().indexOf(query.toLowerCase());
    return index === -1 ? null : { indices: [index], score: 10 };
  }

  const textLower = text.toLowerCase();
  const queryLower = query.toLowerCase();

  const indices: number[] = [];
  let score = 0;
  let previousMatchIndex: number | null = null;
  let queryIndex = 0;

  for (let textIndex = 0; textIndex < textLower.length && queryIndex < queryLower.length; textIndex++) {
    if (textLower[textIndex] !== queryLower[queryIndex]) continue;

    indices.push(textIndex);
    score += 10;

    if (textIndex === 0) {
      score += 8;
    } else if (previousMatchIndex !== null && previousMatchIndex + 1 === textIndex) {
      score += 5;
    } else if (WORD_BOUNDARY.has(textLower[textIndex - 1])) {
      score += 3;
    }

    if (previousMatchIndex !== null) score -= textIndex - previousMatchIndex - 1;

    previousMatchIndex = textIndex;
    queryIndex++;
  }

  if (queryIndex !== queryLower.length) return null;

  // Reject weak scattered matches: at least 5 points per query character.
  if (score < query.length * 5) return null;

  return { indices, score };
}

/** The best fuzzy score across a project's own name and every one of its session titles --
 * `Number.MIN_SAFE_INTEGER` (the Swift reference app's `Int.min`) when nothing at all matches, so callers can
 * always compare two scores without special-casing "no match". */
export function bestScore(projectName: string, sessionTitles: string[], query: string): number {
  let best = fuzzyMatch(projectName, query)?.score ?? Number.MIN_SAFE_INTEGER;
  for (const title of sessionTitles) {
    const result = fuzzyMatch(title, query);
    if (result) best = Math.max(best, result.score);
  }
  return best;
}
