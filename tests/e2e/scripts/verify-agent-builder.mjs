import * as os from 'os';
// Standalone e2e verification script for the Agent Builder flow.
// Not a Playwright test (no test runner) -- a driver script that:
//   1. Enrolls a builder via GraphQL and starts a local builder daemon.
//   2. Creates a build profile whose buildScript compiles a real ELF via gcc.
//   3. Creates a build task and waits for it to complete.
//   4. Verifies the uploaded artifact is a valid ELF and screenshots each step.
//
// Usage: node scripts/verify-agent-builder.mjs
import { chromium, request as pwRequest } from 'playwright';
import * as child_process from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../../../');
const screenshotsDir = path.resolve(__dirname, '../screenshots');
const baseURL = 'http://127.0.0.1:8000';

fs.mkdirSync(screenshotsDir, { recursive: true });

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function graphql(apiCtx, query, variables) {
  const resp = await apiCtx.post(`${baseURL}/graphql`, {
    data: { query, variables },
  });
  const json = await resp.json();
  if (json.errors) {
    throw new Error(`GraphQL error: ${JSON.stringify(json.errors)}`);
  }
  return json.data;
}

async function main() {
  const apiCtx = await pwRequest.newContext({ baseURL });
  const browser = await chromium.launch();
  const pageCtx = await browser.newContext();
  const page = await pageCtx.newPage();

  let builderProcess = null;
  let configPath = null;
  let enrollmentDir = null;
  const results = { screenshots: [] };

  try {
    // ---------- Cleanup: remove any stale builders from prior runs ----------
    // CreateBuildTask randomly selects among ALL "healthy" (recently-seen) builders
    // that support the target OS. Leftover builders from earlier runs of this script
    // (now dead) can still look healthy for a few seconds and steal the task, so we
    // remove them first to guarantee our freshly-registered builder gets the task.
    console.log('[0] Cleaning up stale builders from previous runs...');
    const existing = await graphql(
      apiCtx,
      `query listBuilders { builders { edges { node { id } } } }`
    );
    for (const edge of existing?.builders?.edges ?? []) {
      try {
        await graphql(apiCtx, `mutation del($id: ID!) { deleteBuilder(builderID: $id) }`, { id: edge.node.id });
        console.log(`    Deleted stale builder ${edge.node.id}`);
      } catch (e) {
        console.log(`    Could not delete builder ${edge.node.id}: ${e.message}`);
      }
    }

    // ---------- Step 1: Enroll Builder ----------
    console.log('[1] Registering builder via GraphQL...');
    enrollmentDir = fs.mkdtempSync(path.join(os.tmpdir(), 'realm-builder-'));
    const tavernCli = process.env.TAVERN_BIN || '/tmp/tavern_bin';
    child_process.execFileSync(tavernCli, ['builder', 'generate-csr', '--output-dir', enrollmentDir], { cwd: repoRoot });
    const csr = fs.readFileSync(path.join(enrollmentDir, 'builder.csr'), 'utf8');

    const registerData = await graphql(
      apiCtx,
      `mutation registerNewBuilder($input: CreateBuilderInput!, $csr: String!) {
        registerBuilder(input: $input, csr: $csr) {
          builder { id }
          config
        }
      }`,
      { csr, input: { supportedTargets: ['PLATFORM_LINUX'], upstream: baseURL } }
    );
    const builderId = registerData.registerBuilder.builder.id;
    const configYaml = registerData.registerBuilder.config;
    console.log(`    Registered builder ${builderId}`);

    configPath = path.join(enrollmentDir, 'builder.yaml');
    fs.writeFileSync(configPath, configYaml);

    console.log('    Starting local builder daemon...');
    // Use the prebuilt binary (not `go run`) so the spawned process IS the
    // builder daemon -- `go run` forks a child that survives SIGTERM to the
    // `go run` wrapper, leaking orphaned builders across runs.
    const tavernBin = process.env.TAVERN_BIN || '/tmp/tavern_bin';
    builderProcess = child_process.spawn(
      tavernBin,
      ['builder', '--config', configPath, '--executor', 'local'],
      { cwd: repoRoot, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, MYSQL_ADDR: '' } }
    );
    builderProcess.stdout.on('data', (d) => console.log(`[builder] ${d.toString().trim()}`));
    builderProcess.stderr.on('data', (d) => console.log(`[builder] ${d.toString().trim()}`));

    console.log('    Waiting for builder to check in...');
    let checkedIn = false;
    for (let i = 0; i < 60; i++) {
      const data = await graphql(
        apiCtx,
        `query checkBuilder($id: ID!) { node(id: $id) { ... on Builder { id lastSeenAt } } }`,
        { id: builderId }
      );
      if (data.node?.lastSeenAt) {
        checkedIn = true;
        console.log(`    Builder checked in at ${data.node.lastSeenAt}`);
        break;
      }
      await sleep(1000);
    }
    if (!checkedIn) throw new Error('Builder never checked in');

    await page.goto(`${baseURL}/builder?tab=builders`);
    await page.waitForTimeout(1000);
    const shot1 = path.join(screenshotsDir, '1-builder-enrolled.png');
    await page.screenshot({ path: shot1, fullPage: true });
    results.screenshots.push({ path: shot1, caption: 'Step 1: Builder enrolled and online on /builder?tab=builders' });
    console.log(`    Screenshot saved: ${shot1}`);

    // ---------- Step 2: Create Profile ----------
    console.log('[2] Creating build profile with real ELF buildScript...');
    const profileName = `e2e-elf-${Date.now().toString().slice(-6)}`;
    const buildScript =
      "mkdir -p output && echo 'int main(){return 0;}' | gcc -x c - -o output/imix";
    const tomesData = await graphql(apiCtx, `query getTomes { tomes { edges { node { id name tactic } } } }`);
    const availableTomes = tomesData?.tomes?.edges || [];
    const attachedTomes = availableTomes.slice(0, 2).map((edge) => ({
      tomeID: edge.node.id,
      params: '{}',
    }));
    const profileData = await graphql(
      apiCtx,
      `mutation createProfile($input: CreateBuildProfileInput!) {
        createBuildProfile(input: $input) { id name }
      }`,
      {
        input: {
          name: profileName,
          description: 'E2E verification profile producing a real ELF binary',
          setupscript: 'echo "setup stage running"',
          prebuildscript: 'echo "prebuild stage running"',
          buildScript,
          postbuildscript: 'echo "postbuild stage running"',
          artifactPath: 'output/imix',
          tomes: attachedTomes,
        },
      }
    );
    const profileId = profileData.createBuildProfile.id;
    console.log(`    Created profile ${profileId} (${profileName})`);

    await page.goto(`${baseURL}/builder?tab=profiles`);
    await page.waitForTimeout(1000);
    const shot2 = path.join(screenshotsDir, '2-profile-created.png');
    await page.screenshot({ path: shot2, fullPage: true });
    results.screenshots.push({ path: shot2, caption: 'Step 2: Build profile created on /builder?tab=profiles' });
    console.log(`    Screenshot saved: ${shot2}`);

    // ---------- Step 3: Create Build Task ----------
    console.log('[3] Creating build task...');
    const taskData = await graphql(
      apiCtx,
      `mutation createTask($input: CreateBuildTaskInput!) {
        createBuildTask(input: $input) { id }
      }`,
      { input: { targetOS: 'PLATFORM_LINUX', profileID: profileId } }
    );
    const taskId = taskData.createBuildTask.id;
    console.log(`    Created build task ${taskId}`);

    await page.goto(`${baseURL}/builder?tab=agents`);
    await page.waitForTimeout(1000);
    const shot3a = path.join(screenshotsDir, '3a-build-in-progress.png');
    await page.screenshot({ path: shot3a, fullPage: true });
    results.screenshots.push({ path: shot3a, caption: 'Step 3: Build task in progress on /builder?tab=agents' });

    console.log('    Waiting for build task to complete...');
    let completed = false;
    let artifactName = '';
    let artifactId = '';
    let exitCode = null;
    for (let i = 0; i < 90; i++) {
      const data = await graphql(
        apiCtx,
        `query checkTask($id: ID!) {
          node(id: $id) {
            ... on BuildTask {
              id finishedAt exitCode
              artifact { id name }
            }
          }
        }`,
        { id: taskId }
      );
      const task = data.node;
      if (task?.finishedAt) {
        completed = true;
        exitCode = task.exitCode;
        artifactName = task.artifact?.name || '';
        artifactId = task.artifact?.id || '';
        console.log(`    Build task completed: exitCode=${exitCode}, artifact=${artifactName}`);
        break;
      }
      await sleep(1000);
    }
    if (!completed) throw new Error('Build task never finished');
    if (exitCode !== 0) throw new Error(`Build task failed with exitCode ${exitCode}`);
    if (!artifactName) throw new Error('Build task finished without an artifact');

    await page.goto(`${baseURL}/builder?tab=agents`);
    await page.waitForTimeout(1000);
    const shot3b = path.join(screenshotsDir, '3b-build-completed.png');
    await page.screenshot({ path: shot3b, fullPage: true });
    results.screenshots.push({ path: shot3b, caption: 'Step 3: Build task completed on /builder?tab=agents' });
    console.log(`    Screenshot saved: ${shot3b}`);

    // ---------- Step 4: Verify ELF in Assets & Built Agents ----------
    console.log('[4] Verifying artifact in /assets...');
    await page.goto(`${baseURL}/assets`);
    await page.waitForTimeout(1000);
    // The UI truncates long asset names with an ellipsis, so match on a
    // leading substring rather than the full basename.
    const artifactBasename = artifactName.split('/').pop();
    const assetRow = page.locator('div').filter({ hasText: artifactBasename.slice(0, 20) });
    await assetRow.first().waitFor({ state: 'visible', timeout: 15000 });
    const shot4a = path.join(screenshotsDir, '4a-assets-page.png');
    await page.screenshot({ path: shot4a, fullPage: true });
    results.screenshots.push({ path: shot4a, caption: 'Step 4: Uploaded artifact visible on /assets' });
    console.log(`    Screenshot saved: ${shot4a}`);

    console.log('    Downloading artifact to verify ELF magic bytes...');
    const downloadResp = await apiCtx.get(`${baseURL}/assets/download/${encodeURIComponent(artifactName)}`);
    if (!downloadResp.ok()) throw new Error(`Failed to download artifact, status ${downloadResp.status()}`);
    const artifactBuf = await downloadResp.body();
    const artifactLocalPath = `/tmp/verified-artifact-${artifactId}`;
    fs.writeFileSync(artifactLocalPath, artifactBuf);

    const magic = artifactBuf.subarray(0, 4);
    const isElf = magic[0] === 0x7f && magic[1] === 0x45 && magic[2] === 0x4c && magic[3] === 0x46; // \x7fELF
    let fileOutput;
    try {
      fileOutput = child_process.execSync(`file ${artifactLocalPath}`).toString().trim();
    } catch (_) {
      // `file` utility isn't installed in this environment; fall back to manually
      // decoding the ELF header (class/data/type/machine) so we still report detail.
      const ei_class = artifactBuf[4] === 2 ? '64-bit' : '32-bit';
      const ei_data = artifactBuf[5] === 1 ? 'LSB' : 'MSB';
      const e_type = artifactBuf.readUInt16LE(16);
      const typeName = { 1: 'relocatable', 2: 'executable', 3: 'pie/shared object', 4: 'core' }[e_type] || `type ${e_type}`;
      fileOutput = `ELF ${ei_class} ${ei_data} ${typeName} (decoded from header; 'file' utility unavailable)`;
    }
    console.log(`    Magic bytes: ${Array.from(magic).map((b) => b.toString(16).padStart(2, '0')).join(' ')}`);
    console.log(`    file output: ${fileOutput}`);
    if (!isElf) throw new Error('Downloaded artifact is NOT a valid ELF file!');

    results.elfMagicBytes = Array.from(magic).map((b) => '0x' + b.toString(16)).join(' ');
    results.elfFileOutput = fileOutput;
    results.artifactName = artifactName;
    results.artifactId = artifactId;

    console.log('    Verifying /builder?tab=agents built agent row...');
    await page.goto(`${baseURL}/builder?tab=agents`);
    const buildRow = page.getByRole('row').filter({ hasText: profileName });
    await buildRow.waitFor({ state: 'visible', timeout: 15000 });
    // Status is now a pure CircleCheck icon prepended to the agent name
    // (no "Built" text), so assert on the agent name + "Profile: <name>"
    // subtext plus the icon, rather than a status label.
    const artifactBasenameShort = artifactName.split('/').pop();
    const nameVisible = await buildRow.getByText(artifactBasenameShort, { exact: false }).first().isVisible();
    const profileLabelVisible = await buildRow.getByText(`Profile: ${profileName}`, { exact: false }).isVisible();
    const statusIconVisible = (await buildRow.locator('.lucide-circle-check').count()) > 0;
    const downloadBtnVisible = await buildRow.getByRole('button', { name: 'Download' }).isVisible();
    const createLinkBtnVisible = await buildRow.getByRole('button', { name: 'Create Link' }).isVisible();
    const deleteBtnVisible = await buildRow.getByRole('button', { name: 'Delete' }).isVisible();
    console.log(`    Name=${nameVisible} ProfileLabel=${profileLabelVisible} StatusIcon=${statusIconVisible} Download=${downloadBtnVisible} CreateLink=${createLinkBtnVisible} Delete=${deleteBtnVisible}`);
    if (!nameVisible || !profileLabelVisible || !statusIconVisible || !downloadBtnVisible || !createLinkBtnVisible || !deleteBtnVisible) {
      throw new Error('Built agent row missing expected status/actions');
    }

    const shot4b = path.join(screenshotsDir, '4b-builder-agents-built.png');
    await page.screenshot({ path: shot4b, fullPage: true });
    results.screenshots.push({ path: shot4b, caption: 'Step 4: Built agent row on /builder?tab=agents (Status: Built, Target: Linux, Download/Create Link actions)' });
    console.log(`    Screenshot saved: ${shot4b}`);

    console.log('\n=== VERIFICATION SUCCEEDED ===');
    fs.writeFileSync(path.join(screenshotsDir, 'results.json'), JSON.stringify(results, null, 2));
    console.log(JSON.stringify(results, null, 2));
  } finally {
    await browser.close();
    if (builderProcess) {
      builderProcess.kill('SIGTERM');
      await sleep(1000);
      try { builderProcess.kill('SIGKILL'); } catch (_) {}
    }
    if (enrollmentDir) fs.rmSync(enrollmentDir, { recursive: true, force: true });
    await apiCtx.dispose();
  }
}

main().catch((err) => {
  console.error('VERIFICATION FAILED:', err);
  process.exit(1);
});
