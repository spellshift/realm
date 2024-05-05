---
title: Tomes
tags:
 - User Guide
description: Tomes User Guide
permalink: user-guide/tomes
---
## Tomes

A [Tome](/user-guide/terminology#tome) is an [Eldritch](/user-guide/terminology#eldritch) package that can be run on one or more [Beacons](/user-guide/terminology#beacon). By default, Tavern includes several core [Tomes](/user-guide/terminology#tome) to get you started. Please take a few minutes to read through the options available to you now, and be sure to refer to them as reference when creating your own [Tomes](/user-guide/terminology#tome). If you're looking for information on how to run [Tomes](/user-guide/terminology#tome) and aren't quite ready to write your own, check out our [Getting Started guide](/user-guide/getting-started). Otherwise, adventure onwards, but with a word of warning. [Eldritch](/user-guide/terminology#eldritch) provides a useful abstraction for many offensive operations, however it is under heavy active development at this time and is subject to change. After the release of [Realm](https://github.com/spellshift/realm) version `1.0.0`, [Eldritch](/user-guide/terminology#eldritch) will follow [Semantic Versioning](https://semver.org/), to prevent [Tomes](/user-guide/terminology#tome) from failing when breaking changes are introduced. Until then however, the [Eldritch](/user-guide/terminology#eldritch) API may change. This rapid iteration will enable the language to more quickly reach maturity and ensure we provide the best possible design for operators, so thank you for your patience.

## Anatomy of a Tome

A [Tome](/user-guide/terminology#tome) has a well-defined structure consisting of three key components:

1. `metadata.yml`: This file serves as the [Tome's](/user-guide/terminology#tome) blueprint, containing essential information in YAML format. More information about this file can be found below in the [Tome Metadata section.](/user-guide/tomes#tome-metadata)

2. `main.eldritch`: This file is where the magic happens. It contains the [Eldritch](/user-guide/terminology#eldritch) code evaluated by the [Tome](/user-guide/terminology#tome). Testing your code with [Golem](/user-guide/golem) before running it in production is highly recommended, since it enables signficantly faster developer velocity.

3. `assets/` (optional): [Tomes](/user-guide/terminology#tome) have the capability to leverage additional resources stored externally. These assets, which may include data files, configuration settings, or other tools, are fetched using the implant's callback protocol (e.g. `gRPC`) using the [Eldritch Assets API](/user-guide/eldritch#assets). More information about these files can be found below in the [Tome Assets section.](/user-guide/tomes#tome-assets)

## Tome Metadata

The `metadata.yml` file specifies key information about a [Tome](/user-guide/terminology#tome):

| Name | Description | Required |
|------|-------------|----------|
| `name` | Display name of your [Tome](/user-guide/terminology#tome). | Yes |
| `description` | Provide a helpful description of functionality, for user's of your [Tome](/user-guide/terminology#tome). | Yes |
| `author` | Your name/handle, so you can get credit for your amazing work! | Yes |
| `tactic` | The relevant [MITRE ATT&CK tactic](https://attack.mitre.org/tactics/enterprise/) that best describes this [Tome](/user-guide/terminology#tome). Possible values include: `UNSPECIFIED`, `RECON`, `RESOURCE_DEVELOPMENT`, `INITIAL_ACCESS`, `EXECUTION`, `PERSISTENCE`, `PRIVILEGE_ESCALATION`, `DEFENSE_EVASION`, `CREDENTIAL_ACCESS`, `DISCOVERY`, `LATERAL_MOVEMENT`, `COLLECTION`,`COMMAND_AND_CONTROL`,`EXFILTRATION`, `IMPACT`. | Yes |
| `paramdefs` | A list of [parameters](/user-guide/tomes#tome-parameters) that users may provide to your [Tome](/user-guide/terminology#tome) when it is run. | No |

### Tome Parameters

Parameters are defined as a YAML list, but have their own additional properties:

| Name | Description | Required |
| `name` | Identifier used to access the parameter via the `input_params` global. | Yes |
| `label` | Display name of the parameter for users of the [Tome](/user-guide/terminology#tome). | Yes |
| `type` | Type of the parameter in [Eldritch](/user-guide/terminology#eldritch). Current values include: `string`. | Yes |
| `placeholder` | An example value displayed to users to help explain the parameter's purpose. | Yes |

#### Tome Parameter Example

```yaml
paramdefs:
  - name: path
    type: string
    label: File path
    placeholder: "/etc/open*"
```

#### Referencing Tome Parameters

If you've defined a parameter for your [Tome](/user-guide/terminology#tome), there's a good chance you'll want to use it. Luckily, [Eldritch](/user-guide/terminology#eldritch) makes this easy for you by providing a global `input_params` dictionary, which is populated with the parameter values provided to your [Tome](/user-guide/terminology#tome). To access a parameter, simply use the `paramdef` name (defined in `metadata.yml`). For example:

```python
def print_path_param():
  path = input_params["path"]
  print(path)
```

In the above example, our `metadata.yml` file would specify a value for `paramdefs` with `name: path` set. Then when accessing `input_params["path"]` the string value provided to the [Tome](/user-guide/terminology#tome) is returned.

### Tome Metadata Example

```yaml
name: List files
description: List the files and directories found at the path. Supports basic glob functionality. Does not glob more than one level.
author: hulto
tactic: RECON
paramdefs:
  - name: path
    type: string
    label: File path
    placeholder: "/etc/open*"
```

For more examples, please see Tavern's first party supported [Tomes](https://github.com/spellshift/realm/tree/main/tavern/tomes).

### Tome Assets

Assets are files that can be made available to your [Tome](/user-guide/terminology#tome) if required. These assets are lazy-loaded by [Agents](/user-guide/terminology#agent), so if they are unused they will not increase the on-the-wire size of the payload sent to an [Agent](/user-guide/terminology#agent). This means it's encouraged to include multiple versions of files, for example `myimplant-linux` and `myimplant.exe`. This enables [Tomes](/user-guide/terminology#tome) to be cross-platform, reducing the need for redundant [Tomes](/user-guide/terminology#tome).

#### Referencing Tome Assets

When using the [Eldritch Assets API](/user-guide/eldritch#assets), these assets are referenced based on the **name of the directory** your `main.eldritch` file is in (which may be the same as your [Tome's](/user-guide/terminology#tome) name). For example, with an asset `imix.exe` located in `/mytome/assets/imix.exe` (and where my `metadata.yml` might specify a name of "My Tome"), the correct identifier to reference this asset is `mytome/assets/imix.exe` (no leading `/`). On all platforms (even on Windows systems), use `/` as the path separator when referencing these assets.

Below is the directory structure used in this example:

```text
mytome/
      /main.eldritch
      /metadata.yml
      /assets/
             /imix.exe
             /imix-linux
```

## Importing Tomes from Git

### Create Git Repository

First, create a git repository and commit your [Tomes](/user-guide/terminology#tome) there. Realm primarily supports using GitHub private repositories (or public if suitable), but you may use any git hosting service at your own risk. If you forget to include `main.eldritch` or `metadata.yml` files, your [Tomes](/user-guide/terminology#tome) **will not be imported**. Be sure to include a `main.eldritch` and `metadata.yml` in your [Tome's](/user-guide/terminology#tome) root directory.

Additionally, copy the URL to the repository, which you will need in the next step. **For private repositories, only SSH is supported.**

![Git Repo Copy](/assets/img/user-guide/tomes/git-repo-copy.png)

### Import Tome Repository

Next, navigate to the "Tomes" page and select "Import tome repository"

![Tavern Tomes Page](/assets/img/user-guide/tomes/tomes-page.png)

Then, enter the repository URL copied from the previous step and click "Save Link".

![Import Tomes](/assets/img/user-guide/tomes/import-tomes.png)

### Git Repository Keys

For private repositories, Tavern will need permission to clone the repository. To enable this, copy the provided SSH public key and add it to your repository. For public repositories, you may skip this step.

![Import Tomes Pubkey](/assets/img/user-guide/tomes/import-tomes-pubkey.png)

On GitHub, you can easily accomplish this by adding Tavern's public key for the repository to the "Deploy Keys" setting of your repository.

![GitHub Deploy Keys](/assets/img/user-guide/tomes/github-deploy-keys.png)

### Import Tomes

Now, all that's left is to click "Import tomes". If all goes well, your [Tomes](/user-guide/terminology#tome) will be added to Tavern and will be displayed on the view. If [Tomes](/user-guide/terminology#tome) are missing, be sure each [Tome](/user-guide/terminology#tome)  has a valid `metadata.yml` and `main.eldritch` file in the [Tome](/user-guide/terminology#tome) root directory.

Anytime you need to re-import your [Tomes](/user-guide/terminology#tome) (for example, after an update), you may navigate to the "Tomes" page and click "Refetch tomes".
