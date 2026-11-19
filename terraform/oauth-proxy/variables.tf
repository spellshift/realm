variable "gcp_project" {
  type        = string
  description = "GCP Project ID for the oauth proxy deployment"
  validation {
    condition     = length(var.gcp_project) > 0
    error_message = "Must provide a valid gcp_project"
  }
}

variable "gcp_region" {
  type        = string
  description = "GCP Region for the oauth proxy deployment"
  default     = "us-east4"
}

variable "resource_prefix" {
  type        = string
  description = "Prefix for the oauth proxy's GCP resource names."
  default     = "oauthproxy"
}

variable "oauth_client_id" {
  type        = string
  description = "The single, shared Google OAuth client id the proxy uses on behalf of every tenant. This client must have exactly one Authorized redirect URI configured: \"https://<proxy_domain>/oauth/callback\" (see the proxy_domain output)."
  validation {
    condition     = length(var.oauth_client_id) > 0
    error_message = "Must provide a valid oauth_client_id"
  }
}

variable "oauth_client_secret" {
  type        = string
  description = "The client secret for oauth_client_id."
  sensitive   = true
  validation {
    condition     = length(var.oauth_client_secret) > 0
    error_message = "Must provide a valid oauth_client_secret"
  }
}

variable "proxy_domain" {
  type        = string
  description = "Custom domain the proxy should be reachable at (and the domain whose CNAME you'll point at Cloud Run). Defaults to the proxy's deterministic Cloud Run URL when unset, which also works fine as the registered redirect URI."
  default     = ""
}

variable "allowed_suffixes" {
  type        = string
  description = "Comma-separated list of host suffixes the proxy is allowed to redirect back to after a successful login. A suffix starting with \".\" matches any subdomain of it (and the bare domain itself). Defaults to \".run.app\" so every tenant's default Cloud Run URL works with no further configuration; add your own custom tenant domains here as a tradeoff for simplicity over a per-tenant registry (see docs/_docs/admin-guide/tavern.md)."
  default     = ".run.app"
}

variable "oauthproxy_container_image" {
  type        = string
  description = "Docker container to deploy for the oauth proxy."
  default     = "ghcr.io/spellshift/realm/oauthproxy:latest"
}

variable "min_scale" {
  type        = string
  description = "Minimum number of CloudRun containers to keep running"
  default     = "0"
}

variable "max_scale" {
  type        = string
  description = "Maximum number of CloudRun containers to run"
  default     = "10"
}
