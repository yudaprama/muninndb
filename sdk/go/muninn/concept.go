package muninn

import "unicode/utf8"

// MaxConceptBytes is the maximum number of bytes the server stores in an
// engram's Concept field. Writes whose Concept exceeds this are rejected by
// the storage encoder (internal/storage/erf). TruncateConcept caps a value
// to fit.
const MaxConceptBytes = 512

// conceptEllipsis is appended when TruncateConcept shortens a value so a
// truncated concept signals it was cut. U+2026 = 3 bytes in UTF-8.
const conceptEllipsis = "…"

// TruncateConcept returns s truncated to at most MaxConceptBytes UTF-8 bytes,
// cutting on a rune boundary so multi-byte sequences (CJK, emoji) are never
// split. When shortened, a trailing "…" is appended and counted against the
// budget. Use before Client.Write to guarantee a concept fits the server's
// Concept field regardless of script.
func TruncateConcept(s string) string {
	if len(s) <= MaxConceptBytes {
		return s
	}
	budget := MaxConceptBytes - len(conceptEllipsis)
	if budget < 0 {
		budget = 0
	}
	end := 0
	for end < len(s) {
		_, size := utf8.DecodeRuneInString(s[end:])
		if end+size > budget {
			break
		}
		end += size
	}
	return s[:end] + conceptEllipsis
}
