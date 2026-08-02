package muninn

import "strings"

// ProfileTag marks session-independent profile facts (name, email, ...) so
// recall can select them via a tags_all filter without inferring profile-ness
// from key naming or an empty session.
const ProfileTag = "profile"

// TagsForKey derives the recommended identity-scoping tags for an engram from
// its key and the caller's (userID, sessionID):
//
//   - "user:<userID>"   when userID is non-empty
//   - "session:<sessionID>" when sessionID is non-empty
//   - the key prefix before the first "." (e.g. "user" for "user.email")
//
// This is the canonical convention used by trusted backends (pREST, egents)
// so writes and recall agree on how identities map to tags. Callers wanting a
// different scheme may build the tag slice passed to Client.Write directly.
func TagsForKey(key, userID, sessionID string) []string {
	var tags []string
	if userID != "" {
		tags = append(tags, "user:"+userID)
	}
	if sessionID != "" {
		tags = append(tags, "session:"+sessionID)
	}
	if idx := strings.IndexByte(key, '.'); idx > 0 {
		tags = append(tags, key[:idx])
	}
	return tags
}

// ProfileTags derives the tags for a session-independent profile fact:
// TagsForKey with no session, plus ProfileTag. Use for registration-time facts
// (name, email, ...) so RecallProfile-style queries can select them.
func ProfileTags(key, userID string) []string {
	return append(TagsForKey(key, userID, ""), ProfileTag)
}

// ExtractUserSession parses the "user:<id>" and "session:<id>" tags written by
// TagsForKey back into their components. Either may be empty when absent.
func ExtractUserSession(tags []string) (userID, sessionID string) {
	for _, t := range tags {
		switch {
		case strings.HasPrefix(t, "user:"):
			userID = t[5:]
		case strings.HasPrefix(t, "session:"):
			sessionID = t[8:]
		}
	}
	return
}

// HasAllTags reports whether tags contains every required tag. Used for
// client-side filtering on list responses (the list endpoint does not support
// server-side tag filters).
func HasAllTags(tags, required []string) bool {
	set := make(map[string]struct{}, len(tags))
	for _, t := range tags {
		set[t] = struct{}{}
	}
	for _, req := range required {
		if _, ok := set[req]; !ok {
			return false
		}
	}
	return true
}
