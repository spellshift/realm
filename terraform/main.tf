# 1. Configure OAuth Consent Flow
# 2. Add CNAME Record for OAUTH Domain
# 3. Run Terraform :)

terraform {
  required_providers {
    google = {
      source = "hashicorp/google"
      version = "4.51.0"
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
  default = "kcarretto/tavern:latest"
}

variable "gcp_project" {
  type = string
  description = "GCP Project ID for deployment"
  validation {
    condition = length(var.gcp_project) > 0
    error_message = "Must provide a valid gcp_project"
  }
}
variable "gcp_region" {
  type = string
  description = "GCP Region for deployment"
  default = "us-east4"
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
  default = "100"
}

provider "google" {
  project = var.gcp_project
  region  = var.gcp_region
}

resource "google_project_service" "compute_api" {
  service = "compute.googleapis.com"
}

resource "google_project_service" "cloud_run_api" {
  service = "run.googleapis.com"
}

resource "google_project_service" "cloud_sqladmin_api" {
  service = "sqladmin.googleapis.com"
}

resource "google_sql_database_instance" "tavern-sql-instance" {
  name             = "tavern-db"
  database_version = "MYSQL_8_0"
  region           = var.gcp_region
  deletion_protection = false

  settings {
    tier = var.mysql_tier

    database_flags {
      name  = "default_authentication_plugin"
      value = "caching_sha2_password"
    }
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

resource "google_cloud_run_service" "tavern" {
  name     = "tavern"
  location = var.gcp_region

  traffic {
    percent         = 100
    latest_revision = true
  }

  template {
    spec {
      containers {
        image = var.tavern_container_image
        ports {
          container_port = 80
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
      }
    }

    metadata {
      annotations = {
        "autoscaling.knative.dev/minScale"      = var.min_scale
        "autoscaling.knative.dev/maxScale"      = var.max_scale
        "run.googleapis.com/cloudsql-instances" = google_sql_database_instance.tavern-sql-instance.connection_name
        "run.googleapis.com/client-name"        = "terraform"
        "run.googleapis.com/sessionAffinity"    = true
      }
    }
  }
  autogenerate_revision_name = true

  depends_on = [
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