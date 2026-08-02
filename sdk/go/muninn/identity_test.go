package muninn

import (
	"strings"
	"testing"
)

func TestTagsForKey(t *testing.T) {
	cases := []struct {
		name      string
		key       string
		userID    string
		sessionID string
		want      []string
	}{
		{"user+session+prefix", "user.email", "u1", "s1", []string{"user:u1", "session:s1", "user"}},
		{"user only, prefix", "user.email", "u1", "", []string{"user:u1", "user"}},
		{"no user, no session", "prefs.lang", "", "", []string{"prefs"}},
		{"no dot in key → no prefix tag", "name", "u1", "s1", []string{"user:u1", "session:s1"}},
		{"leading dot (idx 0) → no prefix tag", ".hidden", "u1", "", []string{"user:u1"}},
		{"empty everything", "", "", "", nil},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := TagsForKey(tc.key, tc.userID, tc.sessionID)
			if !equalStrings(got, tc.want) {
				t.Fatalf("TagsForKey(%q,%q,%q) = %v, want %v", tc.key, tc.userID, tc.sessionID, got, tc.want)
			}
		})
	}
}

func TestProfileTags(t *testing.T) {
	got := ProfileTags("user.email", "u1")
	want := []string{"user:u1", "user", ProfileTag}
	if !equalStrings(got, want) {
		t.Fatalf("ProfileTags = %v, want %v", got, want)
	}
	// Profile facts must carry ProfileTag and never a session tag.
	for _, tag := range got {
		if strings.HasPrefix(tag, "session:") {
			t.Fatalf("profile fact must not carry a session tag, got %v", got)
		}
	}
	if !HasAllTags(got, []string{ProfileTag, "user:u1"}) {
		t.Fatalf("profile tags missing profile/user markers: %v", got)
	}
}

func TestExtractUserSession(t *testing.T) {
	cases := []struct {
		tags        []string
		wantUser    string
		wantSession string
	}{
		{[]string{"user:u1", "session:s1", "user"}, "u1", "s1"},
		{[]string{"user:u1"}, "u1", ""},
		{[]string{"session:s1"}, "", "s1"},
		{[]string{"profile", "user"}, "", ""},
		{nil, "", ""},
	}
	for _, tc := range cases {
		uid, sid := ExtractUserSession(tc.tags)
		if uid != tc.wantUser || sid != tc.wantSession {
			t.Fatalf("ExtractUserSession(%v) = (%q,%q), want (%q,%q)", tc.tags, uid, sid, tc.wantUser, tc.wantSession)
		}
	}
}

func TestHasAllTags(t *testing.T) {
	engram := []string{"user:u1", "session:s1", "profile", "user"}
	if !HasAllTags(engram, []string{"profile", "user:u1"}) {
		t.Fatal("expected all required tags present")
	}
	if HasAllTags(engram, []string{"profile", "missing"}) {
		t.Fatal("expected false when a required tag is absent")
	}
	if !HasAllTags(engram, nil) {
		t.Fatal("empty required set should pass")
	}
	if HasAllTags(nil, []string{"profile"}) {
		t.Fatal("absent tags should not satisfy a requirement")
	}
}

// equalStrings compares two slices treating nil and []string{} as equal, since
// TagsForKey may return a nil slice for empty input.
func equalStrings(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
