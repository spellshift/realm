---
title: Tavern
tags:
 - User Guide
description: Tavern User Guide
permalink: user-guide/tavern
---

## Overview

Welcome to Tavern!

This section outlines some basic usage and gotchas.

### Creating quests

Quests are how you interact with beacons think actions/tasks in other c2s.

Quests by default are group actions so scaling your activity is easy and built-in.

Each quest is made up of three main parts:
- Beacons - What you'll be executing on
- Tomes - What you'll be executing
- Input parametrs - arguments passed to the tome and eldritch interperet as `input_params{}`

To start click "Create new quest"

Give your quest a descriptive name so you can reference back to it later if you need to lookup the information again. (We recommend this so you can avoid duplicating work and exposrue on the system).

Select the tome you want to run.

If the tome has parameters you'll be prompted. Consult the placeholder text if you're unsure how to format the input.

Click next to start selecting beacons.

Beacons can be searched by name, group tag, and service tag.

Just start typing what you're looking for and Tavern will search across all three fields.

Now you can select individual beacons or use the "Select all" button to add everything that matches your search criteria.

*Note: `Service + group` searches are and'd while `service + service` or `group + group` searches are or'd.*


## Deployment

This section will walk you through deploying a production ready instance of Tavern to GCP. If you're just looking to play around with Tavern, feel free to run the [docker image (kcarretto/tavern:latest)](https://hub.docker.com/repository/docker/kcarretto/tavern/general) locally.

### 1. Create a GCP Project

Navigate to the [GCP Console](https://console.cloud.google.com/) and [create a new GCP project](https://console.cloud.google.com/projectcreate).
![assets/img/tavern/deploy/create-gcp-project.png](/assets/img/tavern/deploy/create-gcp-project.png)

Make a note of the created Project ID as you'll need that in a later step
![assets/img/tavern/deploy/gcp-project-info.png](/assets/img/tavern/deploy/gcp-project-info.png)

### 2. Setup OAuth (Optional)

_Note: These setup instructions assume you own a domain which you would like to host Tavern at._

If you want to configure OAuth for your Tavern Deployment, navigate to the [GCP OAuth Consent Screen](https://console.cloud.google.com/apis/credentials/consent) and create a new External consent flow. **If you do not configure OAuth, Tavern will not perform any authentication or authorization for requests.**

![assets/img/tavern/deploy/gcp-new-oauth-consent.png](/assets/img/tavern/deploy/gcp-new-oauth-consent.png)

Provide details that users will see when logging into Tavern, for example:

* App Name: "Tavern"
* User Support Email: "YOUR_EMAIL@EXAMPLE.COM"
* App Logo: Upload something cool if you'd like, but then you'll need to complete a verification process.
* App Domain: "https://tavern.mydomain.com/"
* Authorized Domains: "mydomain.com"
* Developer Contact Information: "YOUR_EMAIL@EXAMPLE.COM"

Add the ".../auth/userinfo.profile" scope, used by Tavern to obtain user names and photourls.
![assets/img/tavern/deploy/gcp-oauth-scope.png](/assets/img/tavern/deploy/gcp-oauth-scope.png)

Next, add yourself as a "Test User". **Until you publish your app, only test users may complete the OAuth consent flow.** If you didn't select any options that require verification, you may publish your app now (so you won't need to allowlist the users for your application).

Navigate to the [Credentials Tool](https://console.cloud.google.com/apis/credentials) and select "Create Credentials" -> "OAuth client ID". Be sure to add an "Authorized redirect URI" so that the consent flow redirects to the appropriate Tavern endpoint. For example "mydomain.com/oauth/authorize". Save the resulting Client ID and Client secret for later.
![assets/img/tavern/deploy/oauth-new-creds.png](/assets/img/tavern/deploy/oauth-new-creds.png)

Next, configure a CNAME record for the domain you'd like to host Tavern at (e.g. "tavern.mydomain.com") to point to "ghs.googlehosted.com.".
![assets/img/tavern/deploy/google-dns-cname.png](/assets/img/tavern/deploy/google-dns-cname.png)

And that's it! In the below sections on deployment, please ensure you properly configure your OAUTH_CLIENT_ID, OAUTH_CLIENT_SECRET, and OAUTH_DOMAIN to ensure Tavern is properly configured.

### 3. Google Cloud CLI

Follow [these instructions](https://cloud.google.com/sdk/docs/install) to install the gcloud CLI. This will enable you to quickly obtain credentials that terraform will use to authenticate. Alternatively, you may create a service account (with appropriate permissions) and obtain [Application Default Credentials](https://cloud.google.com/sdk/gcloud/reference/auth/application-default) for it. See [these Authentication Instructions](https://registry.terraform.io/providers/hashicorp/google/latest/docs/guides/provider_reference#authentication) for more information on how to configure GCP authentication for Terraform.

After installing the gcloud CLI, run `gcloud auth application-default login` to obtain Application Default Credentials.

### 4. Terraform

1. Follow [these instructions](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli) to install the Terraform CLI.
2. Clone [the repo](https://github.com/kcarretto/realm) and navigate to the `terraform` directory.
3. Run `terraform init` to install the Google provider for terraform.
4. Run `terraform apply -var="gcp_project=<PROJECT_ID>" -var="oauth_client_id=<OAUTH_CLIENT_ID>" -var="oauth_client_secret=<OAUTH_CLIENT_SECRET>" -var="oauth_domain=<OAUTH_DOMAIN>"` to deploy Tavern!

**Example:**

```sh
terraform apply -var="gcp_project=new-realm-deployment" -var="oauth_client_id=12345.apps.googleusercontent.com" -var="oauth_client_secret=ABCDEFG" -var="oauth_domain=test-tavern.redteam.toys"
```

After terraform completes successfully, head to the [DNS mappings for Cloud Run](https://console.cloud.google.com/run/domains) and wait for a certificate to successfully provision. This may take a while, so go enjoy a nice cup of coffee ☕

After your certificate has successfully provisioned, it may still take a while (e.g. an hour or two) before you are able to visit Tavern using your custom OAuth Domain (if configured).

#### CLI Variables

|Name|Required|Description|
|----|--------|-----------|
|gcp_project|Yes|Project ID of the GCP Project created in step 1.|
|gcp_region|No|Region to deploy to.|
|mysql_user|No|The MySQL user to create and connect with.|
|mysql_passwd|No|The MySQL password to set and connect with. Autogenerated by default. |
|mysql_dbname|No|MySQL Database to create and connect to.|
|mysql_tier|No|The type of instance to run the Cloud SQL Database on.|
|oauth_client_id|Only if OAuth is configured|The OAuth ClientID Tavern will use to connect to the IDP (Google).|
|oauth_client_secret|Only if OAuth is configured|The OAuth Client Secret Tavern will use to connect to the IDP (Google).|
|oauth_domain|Only if OAuth is configured|The OAuth Domain that the IDP should redirect to e.g. tavern.mydomain.com (should be the domain you set a CNAME record for while configuring OAuth).|
|min_scale|The minimum number of containers to run, if set to 0 you may see cold boot latency.|
|max_scale|The maximum number of containers to run.|

### Manual Deployment Tips

Below are some deployment gotchas and notes that we try to address with Terraform, but can be a bit tricky if trying to deploy Tavern manually.

* MySQL version 8.0 must be started with the flag `default_authentication_plugin=caching_sha2_password` for authentication to work properly. A new user must be created for authentication.
* When running in CloudRun, it's best to connect to CloudSQL via a unix socket (so ensure the `MYSQL_NET` env var is set to "unix" ).
  * After adding a CloudSQL connection to your CloudRun instance, this unix socket is available at `/cloudsql/<CONNECTION_STRING>` (e.g. `/cloudsql/realm-379301:us-east4:tavern-db`).
* You must create a new database in your CloudSQL instance before launching Tavern and ensure the `MYSQL_DB` env var is set accordingly.
