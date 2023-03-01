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

variable "gcp_creds_file" {
  type = string
  description = "Path to GCP credentials JSON file"
  default = ""
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
  validation {
    condition = length(var.mysql_passwd) > 0
    error_message = "Must provide a valid mysql_passwd"
  }
}
variable "mysql_dbname" {
  type = string
  description = "Name of the DB to create for the configured MySQL instance"
  default = "tavern"
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
  credentials = file(var.gcp_creds_file)

  project = var.gcp_project
  region  = vars.gcp_region
}

resource "google_sql_database_instance" "tavern-db-instance" {
  name             = "tavern-db"
  database_version = "MYSQL_8_0"
  region           = var.gcp_region

  settings {
    # Second-generation instance tiers are based on the machine
    # type. See argument reference below.
    tier = "db-f1-micro"

    database_flags {
      name  = "default_authentication_plugin"
      value = "caching_sha2_password"
    }
  }
}

resource "google_sql_user" "tavern-user" {
  instance = google_sql_database_instance.tavern-db-instance.name
  name     = var.mysql_user
  password = var.mysql_passwd
}

resource "google_sql_database" "tavern-db" {
  name     = var.mysql_dbname
  instance = google_sql_database_instance.instance.name
}

resource "google_cloud_run_service" "tavern" {
  name     = "Tavern"
  location = var.gcp_region

  template {
    spec {
      containers {
        image = "kcarretto/tavern:latest"
        ports {
          container_port = 80
        }
        env {
          MYSQL_NET = "unix"
          MYSQL_USER = google_sql_user.tavern-user.name
          MYSQL_PASSWD = google_sql_user.tavern-user.password
          MYSQL_DB = google_sql_database.tavern-db.name
          MYSQL_ADDR = format("/cloudsql/%s", google_sql_database_instance.tavern-db-instance.connection_name)

          OAUTH_CLIENT_ID = var.oauth_client_id
          OAUTH_CLIENT_SECRET = var.oauth_client_secret
          OAUTH_DOMAIN = var.oauth_domain
        }
      }
    }

    metadata {
      annotations = {
        "autoscaling.knative.dev/minScale"      = var.min_scale
        "autoscaling.knative.dev/maxScale"      = var.max_scale
        "run.googleapis.com/cloudsql-instances" = google_sql_database_instance.tavern-db-instance.connection_name
        "run.googleapis.com/client-name"        = "terraform"
      }
    }
  }
  autogenerate_revision_name = true
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