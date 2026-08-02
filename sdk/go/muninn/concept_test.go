package muninn

import (
	"strings"
	"testing"
	"unicode/utf8"
)

func TestTruncateConcept_ShortUnchanged(t *testing.T) {
	for _, s := range []string{"", "hi", "user.email"} {
		if got := TruncateConcept(s); got != s {
			t.Fatalf("TruncateConcept(%q) = %q, want unchanged", s, got)
		}
	}
}

func TestTruncateConcept_ExactLimitUnchanged(t *testing.T) {
	s := strings.Repeat("a", MaxConceptBytes) // 512 ASCII bytes
	if got := TruncateConcept(s); got != s {
		t.Fatalf("string at exactly MaxConceptBytes must be unchanged, got len %d", len(got))
	}
}

func TestTruncateConcept_OneByteOver(t *testing.T) {
	s := strings.Repeat("a", MaxConceptBytes+1)
	got := TruncateConcept(s)
	if len(got) > MaxConceptBytes {
		t.Fatalf("result exceeds MaxConceptBytes: %d bytes", len(got))
	}
	if !strings.HasSuffix(got, "…") {
		t.Fatalf("expected trailing ellipsis, got %q", got)
	}
}

func TestTruncateConcept_ByteSafeForCJK(t *testing.T) {
	// 400 CJK runes (3 bytes each) = 1200 bytes — the case the old rune-based
	// cap (400 runes) got wrong. Must come back under 512 bytes.
	s := strings.Repeat("語", 400)
	if len(s) <= MaxConceptBytes {
		t.Fatalf("fixture: expected >%d bytes, got %d", MaxConceptBytes, len(s))
	}
	got := TruncateConcept(s)
	if len(got) > MaxConceptBytes {
		t.Fatalf("CJK result exceeds MaxConceptBytes: %d bytes", len(got))
	}
	if !utf8.ValidString(got) {
		t.Fatalf("result is not valid UTF-8 (rune was split): %q", got)
	}
	if !strings.HasSuffix(got, "…") {
		t.Fatalf("expected trailing ellipsis, got %q", got)
	}
}

func TestTruncateConcept_NeverSplitsRune(t *testing.T) {
	// 4-byte runes (emoji): truncate must land on a boundary.
	s := strings.Repeat("😀", 200) // 800 bytes
	got := TruncateConcept(s)
	if len(got) > MaxConceptBytes {
		t.Fatalf("emoji result exceeds MaxConceptBytes: %d bytes", len(got))
	}
	if !utf8.ValidString(got) {
		t.Fatalf("result is not valid UTF-8 (rune was split): %q", got)
	}
}

func TestTruncateConcept_ASCIIFillsBudget(t *testing.T) {
	s := strings.Repeat("a", 1000)
	got := TruncateConcept(s)
	// ASCII: 509 bytes + "…" (3) = 512 exactly.
	if len(got) != MaxConceptBytes {
		t.Fatalf("ASCII result length = %d, want %d", len(got), MaxConceptBytes)
	}
}
