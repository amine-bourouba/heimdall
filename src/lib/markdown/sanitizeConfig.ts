// DOMPurify's explicit allowlist (ADR 0010 point 1: "DOMPurify uses explicit tag and attribute
// allowlists"). An allowlist strips anything not named here by construction -- `style` and every
// `on*` handler are already gone just by never appearing in ALLOWED_ATTR, with no extra
// FORBID_ATTR needed. `img` is deliberately absent from ALLOWED_TAGS too: `renderMarkdown.ts`'s
// own renderer rule never emits one in the first place (images render as alt text instead), and
// leaving it off here is defense in depth against any other path that might one day produce one.
//
// markdown-it's own table rule renders column alignment as `style="text-align:..."`
// (`markdown-it/lib/rules_block/table.mjs`) -- not allowing `style` at all means aligned table
// columns lose that alignment. The ADR's "strips style" is unconditional (arbitrary `style`
// values are a real CSS-based exfiltration/clickjacking vector even without script execution),
// so this is an accepted, honest cosmetic loss, not a bug to work around.

export const sanitizeConfig = {
  ALLOWED_TAGS: [
    'p', 'br', 'hr',
    'strong', 'em', 'del', 's',
    'a',
    'ul', 'ol', 'li',
    'blockquote',
    'pre', 'code', 'span',
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'table', 'thead', 'tbody', 'tr', 'th', 'td',
  ],
  ALLOWED_ATTR: ['href', 'title', 'class'],
  // Only http(s), mailto and same-document fragment links (ADR 0010 point 1) -- overrides
  // DOMPurify's own, more permissive default (which also allows relative/schemeless URIs).
  ALLOWED_URI_REGEXP: /^(?:https?:|mailto:|#)/i,
};
