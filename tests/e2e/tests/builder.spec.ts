import { test, expect } from '@playwright/test';
import * as child_process from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Builder E2E Test', () => {
  let builderProcess: child_process.ChildProcess | null = null;
  let configPath: string | null = null;
  const repoRoot = path.resolve(__dirname, '../../../');

  test.beforeAll(async ({ request }) => {
    // 1. Register builder via GraphQL mutation
    console.log('Registering builder via GraphQL...');
    const registerMutation = `
      mutation registerNewBuilder($input: CreateBuilderInput!) {
        registerBuilder(input: $input) {
          builder {
            id
          }
          config
        }
      }
    `;

    const registerResp = await request.post('/graphql', {
      data: {
        query: registerMutation,
        variables: {
          input: {
            supportedTargets: ['PLATFORM_LINUX'],
            upstream: 'http://127.0.0.1:8000',
          },
        },
      },
    });

    expect(registerResp.ok()).toBeTruthy();
    const registerJson = await registerResp.json();
    expect(registerJson.errors).toBeUndefined();

    const builderData = registerJson.data.registerBuilder;
    const builderId = builderData.builder.id;
    const configYaml = builderData.config;
    console.log(`Registered builder with ID ${builderId}`);

    // 2. Write builder config to a temporary file
    configPath = path.join('/tmp', `builder-e2e-config-${Date.now()}.yaml`);
    fs.writeFileSync(configPath, configYaml);

    // 3. Spawn builder process with LocalExecutor
    console.log(`Starting local builder daemon with config ${configPath}...`);
    builderProcess = child_process.spawn(
      'go',
      ['run', './tavern', 'builder', '--config', configPath, '--executor', 'local'],
      {
        cwd: repoRoot,
        stdio: ['ignore', 'pipe', 'pipe'],
      }
    );

    builderProcess.stdout?.on('data', (data) => {
      console.log(`[builder stdout] ${data.toString().trim()}`);
    });
    builderProcess.stderr?.on('data', (data) => {
      console.log(`[builder stderr] ${data.toString().trim()}`);
    });

    // 4. Poll until the builder has checked in (lastSeenAt is set)
    console.log(`Waiting for builder ${builderId} to check in...`);
    const checkQuery = `
      query checkBuilder($id: ID!) {
        node(id: $id) {
          ... on Builder {
            id
            lastSeenAt
          }
        }
      }
    `;

    let checkedIn = false;
    for (let i = 0; i < 30; i++) {
      const resp = await request.post('/graphql', {
        data: {
          query: checkQuery,
          variables: { id: builderId },
        },
      });
      if (resp.ok()) {
        const json = await resp.json();
        if (json.data?.node?.lastSeenAt) {
          checkedIn = true;
          console.log(`Builder ${builderId} checked in at ${json.data.node.lastSeenAt}`);
          break;
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    expect(checkedIn).toBeTruthy();
  });

  test.afterAll(async () => {
    if (builderProcess) {
      console.log('Stopping builder process...');
      builderProcess.kill('SIGTERM');
      setTimeout(() => {
        try {
          builderProcess?.kill('SIGKILL');
        } catch (_) {}
      }, 2000);
    }
    if (configPath && fs.existsSync(configPath)) {
      try {
        fs.unlinkSync(configPath);
      } catch (_) {}
    }
  });

  test('Builds agent task locally and displays artifact in assets UI', async ({ page, request }, testInfo) => {
    test.setTimeout(120000);

    // 1. Create a build profile with mock recipe
    const profileName = `e2e-${Date.now().toString().slice(-4)}`;
    console.log(`Creating build profile ${profileName}...`);
    const createProfileMutation = `
      mutation createProfile($input: CreateBuildProfileInput!) {
        createBuildProfile(input: $input) {
          id
          name
        }
      }
    `;

    const profileResp = await request.post('/graphql', {
      data: {
        query: createProfileMutation,
        variables: {
          input: {
            name: profileName,
            description: 'Profile for builder e2e test',
            setupscript: 'echo "setup stage running"',
            prebuildscript: 'echo "prebuild stage running"',
            buildScript: 'mkdir -p output && echo "mock compiled agent binary" > output/imix',
            postbuildscript: 'echo "postbuild stage running"',
            artifactPath: 'output/imix',
          },
        },
      },
    });

    expect(profileResp.ok()).toBeTruthy();
    const profileJson = await profileResp.json();
    expect(profileJson.errors).toBeUndefined();
    const profileId = profileJson.data.createBuildProfile.id;
    console.log(`Created build profile ${profileId}`);

    // 2. Create build task
    console.log(`Creating build task for profile ${profileId}...`);
    const createTaskMutation = `
      mutation createTask($input: CreateBuildTaskInput!) {
        createBuildTask(input: $input) {
          id
        }
      }
    `;

    const taskResp = await request.post('/graphql', {
      data: {
        query: createTaskMutation,
        variables: {
          input: {
            targetOS: 'PLATFORM_LINUX',
            profileID: profileId,
          },
        },
      },
    });

    expect(taskResp.ok()).toBeTruthy();
    const taskJson = await taskResp.json();
    expect(taskJson.errors).toBeUndefined();
    const taskId = taskJson.data.createBuildTask.id;
    console.log(`Created build task ${taskId}`);

    // 3. Wait for the build task to complete and artifact to be uploaded
    console.log(`Waiting for build task ${taskId} to finish and upload artifact...`);
    const checkTaskQuery = `
      query checkTask($id: ID!) {
        node(id: $id) {
          ... on BuildTask {
            id
            finishedAt
            exitCode
            artifact {
              id
              name
            }
          }
        }
      }
    `;

    let completed = false;
    let artifactName = '';
    for (let i = 0; i < 60; i++) {
      const resp = await request.post('/graphql', {
        data: {
          query: checkTaskQuery,
          variables: { id: taskId },
        },
      });
      if (resp.ok()) {
        const json = await resp.json();
        const task = json.data?.node;
        if (task?.finishedAt) {
          completed = true;
          artifactName = task.artifact?.name || '';
          console.log(`Build task ${taskId} completed with exitCode ${task.exitCode}, artifact: ${artifactName}`);
          break;
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    expect(completed).toBeTruthy();
    expect(artifactName).toBeTruthy();

    // 4. Navigate to /assets page
    console.log('Navigating to /assets...');
    await page.goto('/assets');

    // 5. Wait for the artifact row to appear in the table
    // Match either the full name or the profile identifier
    console.log(`Waiting for asset matching "${profileName}" in table...`);
    const assetRow = page.locator('div').filter({ hasText: `imix-${profileName}` });
    await expect(assetRow.first()).toBeVisible({ timeout: 15000 });

    console.log('Artifact verified in Assets UI table');

    // 6. Screenshot final screen showing uploaded asset in Assets UI
    await page.waitForTimeout(1000);
    const screenshotPath = testInfo.outputPath('builder-e2e-asset.png');
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`Saved screenshot to ${screenshotPath}`);
  });
});
