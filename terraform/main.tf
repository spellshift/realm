# 1. Configure OAuth Consent Flow
# 2. Add CNAME Record for OAUTH Domain
# 3. Run Terraform :)

terraform {
  required_providers {
    google = {
      source = "hashicorp/google"
      version = "5.15.0"
    }
  }
}

resource "random_password" "defaultmysql" {
  length           = 16
  special          = true
  override_special = "!#$%&*()-_=+[]{}<>:?"
}

variable "tavern_container_image" {
  type = string
  description = "Docker container to deploy"
  default = "spellshift/tavern:latest"
}

variable "tavern_request_timeout_seconds" {
  type = number
  description = "How many seconds before a request is dropped, defaults to 3600 (the maximum) to accommodate reverse shells (which are killed when this timeout is reached)"
  default = 3600

  validation {
    condition = var.tavern_request_timeout_seconds >= 1 && var.tavern_request_timeout_seconds <= 3600
    error_message = "tavern_request_timeout_seconds must be a value between 1 and 3600 seconds"
  }
}

variable "gcp_project" {
  type = string
  description = "GCP Project ID for deployment"
  validation {
    condition = length(var.gcp_project) > 0
    error_message = "Must provide a valid gcp_project"
  }
}

data "google_project" "project" {
  project_id = var.gcp_project
}

data "google_dns_managed_zones" "zones" {
  project = var.gcp_project
}

# Lookup the oauth_domain_zone if able
locals {
  domain_zone_map = {
    for zone in data.google_dns_managed_zones.zones.managed_zones :
    zone.dns_name => zone.name
  }
  oauth_top_domain_arr = split(".", var.oauth_domain)
  oauth_top_domain = format("%s.", join(".", slice(local.oauth_top_domain_arr, length(local.oauth_top_domain_arr)-2, length(local.oauth_top_domain_arr))))
  oauth_domain_zone = contains(keys(local.domain_zone_map), local.oauth_top_domain) ? local.domain_zone_map[local.oauth_top_domain] : ""
}

output "oauth_domain_configuration" {
  value = local.oauth_domain_zone == "" ? format("Unable to find a managed DNS zone for the domain: %s. Skipping auotamatic DNS record configuration consult the documentation for manual setup https://docs.realm.pub/admin-guide/tavern", var.oauth_domain) : "DNS Zone found automatic configuration will be attempted"
}

data "google_dns_managed_zone" "oauth_dns_zone" {
  count = local.oauth_domain_zone == "" ? 0 : 1
  name = local.oauth_domain_zone
}

resource "google_dns_record_set" "oauth_dns_record" {
  count = local.oauth_domain_zone == "" ? 0 : 1
  name = format("%s.", var.oauth_domain)
  type = "CNAME"
  ttl  = 300

  managed_zone = data.google_dns_managed_zone.oauth_dns_zone[count.index].name

  rrdatas = ["ghs.googlehosted.com."]
}


variable "gcp_region" {
  type = string
  description = "GCP Region for deployment"
  default = "us-east4"
}

variable "disable_gcp_pubsub" {
  type = bool
  description = "Disables GCP pubsub setup and instead defaults to inmem pubsub, suitable for use-cases where only one tavern instance will exist and distributed orchestration is unnecessary"
  default = false
}

variable "gcp_pubsub_topic_shell_input" {
  type = string
  description = "Name of the GCP pubsub topic to create for shell input"
  default = "shell_input"
}

variable "gcp_pubsub_topic_shell_output" {
  type = string
  description = "Name of the GCP pubsub topic to create for shell output"
  default = "shell_output"
}

variable "mysql_user" {
  type = string
  description = "Username to set for the configured MySQL instance"
  default = "tavern"
}
variable "mysql_passwd" {
  type = string
  description = "Password to set for the configured MySQL instance"
  sensitive = true
  default = ""
}
variable "mysql_dbname" {
  type = string
  description = "Name of the DB to create for the configured MySQL instance"
  default = "tavern"
}
variable "mysql_tier" {
  type = string
  description = "Instance tier to run the SQL database on, see `gcloud sql tiers list` for options"
  default = "db-custom-2-7680"
}
variable "oauth_client_id" {
  type = string
  description = "OAUTH_CLIENT_ID used to configure Tavern OAuth"
  default = ""
}
variable "oauth_client_secret" {
  type = string
  description = "OAUTH_CLIENT_SECRET used to configure Tavern OAuth"
  default = ""
  sensitive = true
}
variable "oauth_domain" {
  type = string
  description = "OAUTH_DOMAIN used to configure Tavern OAuth"
  default = ""
}
variable "min_scale" {
  type = string
  description = "Minimum number of CloudRun containers to keep running"
  default = "0"
}
variable "max_scale" {
  type = string
  description = "Maximum number of CloudRun containers to run"
  default = "10"
}
variable "enable_metrics" {
  type = bool
  description = "Enable prometheus sidecar and Tavern metrics collection"
  default = false
}

provider "google" {
  project = var.gcp_project
  region  = var.gcp_region
}

resource "google_project_service" "compute_api" {
  service = "compute.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "cloud_run_api" {
  service = "run.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "secret_manager" {
  service = "secretmanager.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "cloud_sqladmin_api" {
  service = "sqladmin.googleapis.com"
  disable_on_destroy = false
}

resource "google_sql_database_instance" "tavern-sql-instance" {
  name             = "tavern-db"
  database_version = "MYSQL_8_0"
  region           = var.gcp_region
  deletion_protection = false

  settings {
    tier = var.mysql_tier

    # database_flags {
    #   name  = "default_authentication_plugin"
    #   value = "caching_sha2_password"
    # }
  }

  depends_on = [
    google_project_service.compute_api,
    google_project_service.cloud_sqladmin_api
  ]
}

resource "google_sql_user" "tavern-user" {
  instance = google_sql_database_instance.tavern-sql-instance.name
  name     = var.mysql_user
  password = var.mysql_passwd == "" ? random_password.defaultmysql.result : var.mysql_passwd
}

resource "google_sql_database" "tavern-db" {
  name     = var.mysql_dbname
  instance = google_sql_database_instance.tavern-sql-instance.name
}

locals {
  tavern_container_name = "tavern"
  prometheus_container_name = "prometheus-sidecar"
}

resource "google_service_account" "svctavern" {
  account_id = "svctavern"
  description = "The service account Realm's Tavern uses to connect to GCP based services. Managed by Terraform."
}

resource "google_secret_manager_secret" "tavern-grpc-priv-key" {
  secret_id = "REALM_tavern_ed25519_private_key"

  replication {
    auto {
    }
  }
}

resource "google_secret_manager_secret_iam_binding" "tavern-secrets-read-binding" {
  project = var.gcp_project
  secret_id = google_secret_manager_secret.tavern-grpc-priv-key.secret_id
  role = "roles/secretmanager.secretAccessor"
  members = [
    "serviceAccount:${google_service_account.svctavern.email}",
  ]
}

resource "google_secret_manager_secret_iam_binding" "tavern-secrets-write-binding" {
  project = var.gcp_project
  secret_id = google_secret_manager_secret.tavern-grpc-priv-key.secret_id
  role = "roles/secretmanager.secretVersionAdder"
  members = [
    "serviceAccount:${google_service_account.svctavern.email}",
  ]
}

resource "google_project_iam_member" "tavern-sqlclient-binding" {
  project = var.gcp_project
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.svctavern.email}"
}

resource "google_project_iam_member" "tavern-metricwriter-binding" {
  project = var.gcp_project
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.svctavern.email}"
}

resource "google_project_iam_member" "tavern-logwriter-binding" {
  project = var.gcp_project
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.svctavern.email}"
}

resource "google_project_iam_member" "tavern-pubsub-binding" {
  project = var.gcp_project
  role    = "roles/pubsub.editor"
  member  = "serviceAccount:${google_service_account.svctavern.email}"
}

resource "google_pubsub_topic" "shell_input" {
  count = var.disable_gcp_pubsub ? 0 : 1
  name = var.gcp_pubsub_topic_shell_input
}
resource "google_pubsub_subscription" "shell_input-sub" {
  count = var.disable_gcp_pubsub ? 0 : 1
  name  = format("%s-sub", var.gcp_pubsub_topic_shell_input)
  topic = google_pubsub_topic.shell_input[0].id
}
resource "google_pubsub_topic" "shell_output" {
  count = var.disable_gcp_pubsub ? 0 : 1
  name = var.gcp_pubsub_topic_shell_output
}
resource "google_pubsub_subscription" "shell_output-sub" {
  count = var.disable_gcp_pubsub ? 0 : 1
  name  = format("%s-sub", var.gcp_pubsub_topic_shell_output)
  topic = google_pubsub_topic.shell_output[0].id
}

resource "google_cloud_run_service" "tavern" {
  name     = "tavern"
  location = var.gcp_region

  traffic {
    percent         = 100
    latest_revision = true
  }

  template {
    spec {
      service_account_name = google_service_account.svctavern.email
      // Controls request timeout, must be long-lived to enable reverse shell support
      timeout_seconds = var.tavern_request_timeout_seconds

      containers {
        name = local.tavern_container_name
        image = var.tavern_container_image

        ports {
          container_port = 8000
        }
        env {
          name = "MYSQL_NET"
          value = "unix"
        }
        env {
          name = "MYSQL_USER"
          value = google_sql_user.tavern-user.name
        }
        env {
          name = "MYSQL_PASSWD"
          value = google_sql_user.tavern-user.password
        }
        env {
          name = "MYSQL_DB"
          value = google_sql_database.tavern-db.name
        }
        env {
          name = "MYSQL_ADDR"
          value = format("/cloudsql/%s", google_sql_database_instance.tavern-sql-instance.connection_name)
        }
        env {
          name = "OAUTH_CLIENT_ID"
          value = var.oauth_client_id
        }
        env {
          name = "OAUTH_CLIENT_SECRET"
          value = var.oauth_client_secret
        }
        env {
          name = "OAUTH_DOMAIN"
          value = format("https://%s", var.oauth_domain)
        }
        env {
          name = "GCP_PROJECT_ID"
          value = var.gcp_project
        }

        // Only configure GCP pubsub if it is not disabled
        dynamic "env" {
          for_each = var.disable_gcp_pubsub ? [] : [
            {
              name = "PUBSUB_TOPIC_SHELL_INPUT"
              value = format("gcppubsub://%s", google_pubsub_topic.shell_input[0].id)
            },
            {
              name = "PUBSUB_SUBSCRIPTION_SHELL_INPUT"
              value = format("gcppubsub://%s", google_pubsub_subscription.shell_input-sub[0].id)
            },
            {
              name = "PUBSUB_TOPIC_SHELL_OUTPUT"
              value = format("gcppubsub://%s", google_pubsub_topic.shell_output[0].id)
            },
            {
              name = "PUBSUB_SUBSCRIPTION_SHELL_OUTPUT"
              value = format("gcppubsub://%s", google_pubsub_subscription.shell_output-sub[0].id)
            }
          ]
          content {
            name = env.value.name
            value = env.value.value
          }
        }

        env {
          name = "DISABLE_DEFAULT_TOMES"
          value = ""
        }
        env {
          name = "ENABLE_DEBUG_LOGGING"
          value = ""
        }
        env {
          name = "ENABLE_JSON_LOGGING"
          value = "1"
        }
        env {
          name = "ENABLE_INSTANCE_ID_LOGGING"
          value = "1"
        }
        env {
          name = "ENABLE_GRAPHQL_RAW_QUERY_LOGGING"
          value = "1"
        }

        env {
          name = "ENABLE_METRICS"
          value = var.enable_metrics ? "1" : ""
        }
      }

      // Only create prometheus sidecar if metrics enabled
      dynamic "containers" {
        for_each = var.enable_metrics ? [{
            image = "us-docker.pkg.dev/cloud-ops-agents-artifacts/cloud-run-gmp-sidecar/cloud-run-gmp-sidecar:1.0.0"
            name = local.prometheus_container_name
          }] : []
        content {
          name = containers.value.name
          image = containers.value.image
        }
      }
    }

    metadata {
      annotations = {
        for k, v in {
        "autoscaling.knative.dev/minScale"      = var.min_scale
        "autoscaling.knative.dev/maxScale"      = var.max_scale
        "run.googleapis.com/cloudsql-instances" = google_sql_database_instance.tavern-sql-instance.connection_name
        "run.googleapis.com/client-name"        = "terraform"
        "run.googleapis.com/sessionAffinity"    = true
        "run.googleapis.com/container-dependencies" = var.enable_metrics ? jsonencode({"${local.prometheus_container_name}" = [local.tavern_container_name]}) : ""
      }: k => v if v != ""
      }
    }
  }
  autogenerate_revision_name = true

  depends_on = [
    google_project_iam_member.tavern-sqlclient-binding,
    google_secret_manager_secret_iam_binding.tavern-secrets-read-binding,
    google_secret_manager_secret_iam_binding.tavern-secrets-write-binding,
    google_project_iam_member.tavern-metricwriter-binding,
    google_project_iam_member.tavern-logwriter-binding,
    google_project_iam_member.tavern-pubsub-binding,
    google_project_service.cloud_run_api,
    google_project_service.cloud_sqladmin_api,
    google_sql_user.tavern-user,
    google_sql_database.tavern-db
  ]
}

resource "google_cloud_run_service_iam_binding" "no-auth-required" {
  location = google_cloud_run_service.tavern.location
  service  = google_cloud_run_service.tavern.name
  role     = "roles/run.invoker"
  members = [
    "allUsers"
  ]
}



# === Redirectors ===
# Lookup the redirector domain if able
locals {
  domain_zone_map_of = {
    for zone in data.google_dns_managed_zones.zones.managed_zones :
    zone.dns_name => zone.name
  }
  redirectors_top_domains = {
    for redir in var.redirectors : redir.domain => format("%s.", join(".", slice(split(".", redir.domain), length(split(".", redir.domain))-2, length(split(".", redir.domain)))))
  }
  redirector_zones = [
    for redir in var.redirectors: {
      "domain": redir.domain,
      "zone": lookup(local.domain_zone_map, local.redirectors_top_domains[redir.domain], "")
    }
    if lookup(local.domain_zone_map, local.redirectors_top_domains[redir.domain], "") != ""
  ]
  redirector_zones_missing = [
    for redir in var.redirectors: redir.domain
    if lookup(local.domain_zone_map, local.redirectors_top_domains[redir.domain], "") == ""
  ]
}

output "redir_domain_configuration" {
  value = length(local.redirector_zones_missing) == 0 ? "All domains configured automatically" : format("Unable to configure DNS for the following, please configure a CNAME pointing to `ghs.googlehosted.com.`: %s", join(",", local.redirector_zones_missing))
}

# Setup DNS for grpc / http transports
data "google_dns_managed_zone" "redir_dns_zone" {
  count = length(local.redirector_zones)
  name = local.redirector_zones[count.index].zone
}

resource "google_dns_record_set" "redir_dns_record" {
  count = length(local.redirector_zones)
  name = format("%s.", local.redirector_zones[count.index].domain)
  type = "CNAME"
  ttl  = 300

  managed_zone = data.google_dns_managed_zone.redir_dns_zone[count.index].name

  rrdatas = ["ghs.googlehosted.com."]
}

variable "redirector_upstream" {
  type = string
  description = "Upstream that redirectors should point to."
  default = ""
}

variable "redirectors" {
  type = list(object({
    domain    = string
    transport = string
  }))
  description = "List of redirectors domains and transpoarts to configure."
}

resource "google_service_account" "svctavern_redirector" {
  account_id = "svctavern-redirector"
  description = "The service account Realm's Tavern Redirector uses to connect to GCP based services. Managed by Terraform."
}

resource "google_project_iam_member" "redirector-metricwriter-binding" {
  project = var.gcp_project
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.svctavern_redirector.email}"
}

resource "google_project_iam_member" "redirector-logwriter-binding" {
  project = var.gcp_project
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.svctavern_redirector.email}"
}

resource "google_cloud_run_service" "redirector" {
  count    = length(var.redirectors)
  name     = "tavern-redirector-${count.index}"
  location = var.gcp_region

  traffic {
    percent         = 100
    latest_revision = true
  }

  template {
    spec {
      service_account_name = google_service_account.svctavern_redirector.email
      // Controls request timeout, must be long-lived to enable reverse shell support
      timeout_seconds = var.tavern_request_timeout_seconds

      containers {
        name = local.tavern_container_name
        image = var.tavern_container_image
        command = ["/app/tavern", "redirector", "--transport", var.redirectors[count.index].transport, var.redirector_upstream]
        env {
          name = "ENABLE_DEBUG_LOGGING"
          value = "true"
        }
        ports {
          container_port = 8080
        }
      }
    }

    metadata {
      annotations = {
        for k, v in {
        "autoscaling.knative.dev/minScale"      = var.min_scale
        "autoscaling.knative.dev/maxScale"      = var.max_scale
        "run.googleapis.com/client-name"        = "terraform"
        "run.googleapis.com/sessionAffinity"    = true
      }: k => v if v != ""
      }
    }
  }
  autogenerate_revision_name = true

  depends_on = [
    google_project_iam_member.redirector-logwriter-binding,
    google_project_iam_member.redirector-logwriter-binding,
    google_project_service.cloud_run_api
  ]
}

resource "google_cloud_run_service_iam_binding" "no-auth-required-redirector" {
  count    = length(var.redirectors)
  location = google_cloud_run_service.redirector[count.index].location
  service  = google_cloud_run_service.redirector[count.index].name
  role     = "roles/run.invoker"
  members = [
    "allUsers"
  ]
}

resource "google_cloud_run_domain_mapping" "redirector-domain" {
  count    = length(var.redirectors)
  location = google_cloud_run_service.redirector[count.index].location
  name     = var.redirectors[count.index].domain

  metadata {
    namespace = google_cloud_run_service.redirector[count.index].project
  }

  spec {
    route_name = google_cloud_run_service.redirector[count.index].name
  }
}

resource "google_cloud_run_domain_mapping" "tavern-domain" {
  count = var.oauth_domain == "" ? 0 : 1 # Only create mapping if OAUTH is configured
  location = google_cloud_run_service.tavern.location
  name     = var.oauth_domain

  metadata {
    namespace = google_cloud_run_service.tavern.project
  }

  spec {
    route_name = google_cloud_run_service.tavern.name
  }
}
