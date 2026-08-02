package muninn

import "os"

// DefaultTrustEdgeHeader is the default name of the edge-auth trust header
// (overridable via MUNINN_TRUST_EDGE_HEADER). Trusted backends set it
// per-request via WithTrustedVaultHeader to bind a call to a tenant; the
// server honors it when running in edge-auth mode.
const DefaultTrustEdgeHeader = "X-Tenant-Id"

// TrustEdgeHeaderFromEnv returns the edge-auth trust header name configured
// via MUNINN_TRUST_EDGE_HEADER, defaulting to X-Tenant-Id. Callers that prefer
// non-env config may use DefaultTrustEdgeHeader or pass a literal to
// WithTrustedVaultHeader directly.
func TrustEdgeHeaderFromEnv() string {
	if h := os.Getenv("MUNINN_TRUST_EDGE_HEADER"); h != "" {
		return h
	}
	return DefaultTrustEdgeHeader
}
