# Shared, global OAuth proxy stack. Deployed exactly once per Google OAuth client, not per
# tenant: it owns the single Google OAuth client whose one Authorized redirect URI
# ("https://<its own domain>/oauth/callback") is the only redirect URI that will ever need to be
# registered by hand in the GCP Console. Every tenant Tavern deployment (terraform/main.tf) then
# only needs its own `terraform apply` with `oauth_proxy_url` pointed at this service's URL --
# see docs/_docs/admin-guide/tavern.md for the full setup flow.

terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "5.15.0"
    }
  }
  backend "gcs" {
    bucket = "tavern_tf_state" # Paste the output from Step 1 of the Tavern setup docs
    prefix = "terraform/oauth-proxy/state"
  }
}

provider "google" {
  project = var.gcp_project
  region  = var.gcp_region
}

data "google_project" "project" {}

locals {
  resource_prefix      = var.resource_prefix
  default_proxy_domain = "${local.resource_prefix}-${data.google_project.project.number}.${var.gcp_region}.run.app"
  proxy_domain         = var.proxy_domain != "" ? var.proxy_domain : local.default_proxy_domain
}

output "proxy_domain" {
  description = "The oauth proxy's own domain. Register \"https://<this value>/oauth/callback\" as the sole Authorized redirect URI on the shared Google OAuth client."
  value       = local.proxy_domain
}

output "proxy_url" {
  description = "Pass this as oauth_proxy_url to each tenant's terraform/main.tf."
  value       = "https://${local.proxy_domain}"
}

resource "google_project_service" "run_api" {
  service            = "run.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "secret_manager" {
  service            = "secretmanager.googleapis.com"
  disable_on_destroy = false
}

resource "google_service_account" "svcoauthproxy" {
  account_id  = "${local.resource_prefix}-svc"
  description = "The service account the Realm OAuth proxy uses to connect to GCP based services. Managed by Terraform."
}

# The proxy's ed25519 signing key is generated on first run and persisted here (via
# tavern/internal/crypto + tavern/internal/secrets, the same mechanism Tavern itself uses for its
# own keys), so its published JWKS stays stable across revisions/restarts.
resource "google_secret_manager_secret" "oauthproxy-signing-key" {
  secret_id = "${local.resource_prefix}-REALM_tavern_ed25519_private_key"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_iam_binding" "oauthproxy-secrets-read-binding" {
  project   = var.gcp_project
  secret_id = google_secret_manager_secret.oauthproxy-signing-key.secret_id
  role      = "roles/secretmanager.secretAccessor"
  members = [
    "serviceAccount:${google_service_account.svcoauthproxy.email}",
  ]
}

resource "google_secret_manager_secret_iam_binding" "oauthproxy-secrets-write-binding" {
  project   = var.gcp_project
  secret_id = google_secret_manager_secret.oauthproxy-signing-key.secret_id
  role      = "roles/secretmanager.secretVersionAdder"
  members = [
    "serviceAccount:${google_service_account.svcoauthproxy.email}",
  ]
}

resource "google_cloud_run_service" "oauthproxy" {
  name     = local.resource_prefix
  location = var.gcp_region

  template {
    spec {
      service_account_name = google_service_account.svcoauthproxy.email

      containers {
        name  = "oauthproxy"
        image = var.oauthproxy_container_image

        ports {
          container_port = 8080
        }

        env {
          name  = "OAUTH_PROXY_RESOURCE_PREFIX"
          value = local.resource_prefix
        }
        env {
          name  = "OAUTH_PROXY_DOMAIN"
          value = "https://${local.proxy_domain}"
        }
        env {
          name  = "OAUTH_PROXY_ALLOWED_SUFFIXES"
          value = var.allowed_suffixes
        }
        env {
          name  = "OAUTH_CLIENT_ID"
          value = var.oauth_client_id
        }
        env {
          name  = "OAUTH_CLIENT_SECRET"
          value = var.oauth_client_secret
        }
        env {
          name  = "GCP_PROJECT_ID"
          value = var.gcp_project
        }
      }
    }

    metadata {
      annotations = {
        "autoscaling.knative.dev/minScale" = var.min_scale
        "autoscaling.knative.dev/maxScale" = var.max_scale
        "run.googleapis.com/client-name"   = "terraform"
      }
    }
  }

  traffic {
    percent         = 100
    latest_revision = true
  }

  depends_on = [
    google_project_service.run_api,
    google_secret_manager_secret_iam_binding.oauthproxy-secrets-read-binding,
    google_secret_manager_secret_iam_binding.oauthproxy-secrets-write-binding,
  ]
}

resource "google_cloud_run_service_iam_binding" "no-auth-required" {
  location = google_cloud_run_service.oauthproxy.location
  service  = google_cloud_run_service.oauthproxy.name
  role     = "roles/run.invoker"
  members = [
    "allUsers"
  ]
}

resource "google_cloud_run_domain_mapping" "oauthproxy-domain" {
  count    = var.proxy_domain == "" ? 0 : 1 # Only create a mapping if a custom domain is configured.
  location = google_cloud_run_service.oauthproxy.location
  name     = var.proxy_domain

  metadata {
    namespace = google_cloud_run_service.oauthproxy.project
  }

  spec {
    route_name = google_cloud_run_service.oauthproxy.name
  }
}
