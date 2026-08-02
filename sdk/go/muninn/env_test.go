package muninn

import "testing"

func TestTrustEdgeHeaderFromEnv(t *testing.T) {
	t.Setenv("MUNINN_TRUST_EDGE_HEADER", "")
	if got := TrustEdgeHeaderFromEnv(); got != "X-Tenant-Id" {
		t.Fatalf("default: got %q, want X-Tenant-Id", got)
	}

	t.Setenv("MUNINN_TRUST_EDGE_HEADER", "X-Workspace-Id")
	if got := TrustEdgeHeaderFromEnv(); got != "X-Workspace-Id" {
		t.Fatalf("override: got %q, want X-Workspace-Id", got)
	}
}
