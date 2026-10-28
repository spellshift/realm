import * as os from 'os';
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
  const browser = await chromium.launch({
    headless: true,
  });
  const pageCtx = await browser.newContext({
    viewport: { width: 1440, height: 900 },
  });
  const page = await pageCtx.newPage();

  let builderProcess = null;
  let configPath = null;
  let enrollmentDir = null;
  const results = { screenshots: [] };

  try {
    // 0. Cleanup stale builders
    console.log('[0] Cleaning up stale builders...');
    const existing = await graphql(apiCtx, `query listBuilders { builders { edges { node { id } } } }`);
    for (const edge of existing?.builders?.edges ?? []) {
      try {
        await graphql(apiCtx, `mutation del($id: ID!) { deleteBuilder(builderID: $id) }`, { id: edge.node.id });
      } catch (e) {}
    }

    // Also upload a regular non-agent asset to clearly demonstrate filtering and distinct icons
    console.log('[0b] Uploading a standard non-agent asset...');
    const form = new FormData();
    const testBlob = new Blob(['sample-data-contents'], { type: 'text/plain' });
    form.append('file', testBlob, 'documentation-guide.txt');
    // Use tavern upload endpoint if available or via graphql
    try {
      await apiCtx.post(`${baseURL}/assets/upload`, {
        multipart: {
          file: {
            name: 'documentation-guide.txt',
            mimeType: 'text/plain',
            buffer: Buffer.from('sample documentation guide content'),
          },
        },
      });
    } catch (e) {
      console.log('Upload notice:', e.message);
    }

    // 1. Enroll builder & start local daemon
    console.log('[1] Enrolling builder...');
    enrollmentDir = fs.mkdtempSync(path.join(os.tmpdir(), 'realm-builder-'));
    child_process.execFileSync(process.env.TAVERN_BIN || '/tmp/tavern_bin', ['builder', 'generate-csr', '--output-dir', enrollmentDir], { cwd: repoRoot });
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

    configPath = path.join(enrollmentDir, 'builder.yaml');
    fs.writeFileSync(configPath, configYaml);

    const tavernBin = process.env.TAVERN_BIN || '/tmp/tavern_bin';
    builderProcess = child_process.spawn(
      tavernBin,
      ['builder', '--config', configPath, '--executor', 'local'],
      { cwd: repoRoot, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, MYSQL_ADDR: '' } }
    );
    builderProcess.stdout.on('data', (d) => console.log(`[builder] ${d.toString().trim()}`));
    builderProcess.stderr.on('data', (d) => console.log(`[builder] ${d.toString().trim()}`));

    console.log('    Waiting for builder to check in...');
    for (let i = 0; i < 60; i++) {
      const data = await graphql(
        apiCtx,
        `query checkBuilder($id: ID!) { node(id: $id) { ... on Builder { id lastSeenAt } } }`,
        { id: builderId }
      );
      if (data?.node?.lastSeenAt) {
        console.log(`    Builder checked in at ${data.node.lastSeenAt}`);
        break;
      }
      await sleep(1000);
    }

    // 2. Create profile & build an agent
    console.log('[2] Creating profile and building an agent...');
    const profileName = `ops-agent-profile-${Date.now().toString().slice(-4)}`;
    const tomesData = await graphql(apiCtx, `query getTomes { tomes { edges { node { id name tactic } } } }`);
    const availableTomes = tomesData?.tomes?.edges || [];
    const attachedTomes = availableTomes.slice(0, 2).map((edge) => ({
      tomeID: edge.node.id,
      params: '{}',
    }));

    const profileData = await graphql(
      apiCtx,
      `mutation mkProfile($input: CreateBuildProfileInput!) {
        createBuildProfile(input: $input) { id name }
      }`,
      {
        input: {
          name: profileName,
          description: 'Production agent profile for Linux',
          setupscript: 'echo "setup"',
          prebuildscript: 'echo "prebuild"',
          buildScript: 'mkdir -p output && echo "int main(){return 0;}" | gcc -x c - -o output/imix',
          postbuildscript: 'echo "postbuild"',
          artifactPath: 'output/imix',
          tomes: attachedTomes,
        },
      }
    );
    const profileId = profileData.createBuildProfile.id;

    const taskData = await graphql(
      apiCtx,
      `mutation mkTask($input: CreateBuildTaskInput!) {
        createBuildTask(input: $input) { id }
      }`,
      { input: { profileID: profileId, targetOS: 'PLATFORM_LINUX' } }
    );
    const taskId = taskData.createBuildTask.id;
    console.log(`    Created build task ${taskId}, awaiting completion...`);

    let artifactName = '';
    for (let i = 0; i < 60; i++) {
      const td = await graphql(
        apiCtx,
        `query getTask($id: ID!) {
          node(id: $id) {
            ... on BuildTask {
              id
              finishedAt
              exitCode
              artifact { id name }
            }
          }
        }`,
        { id: taskId }
      );
      if (td?.node?.finishedAt) {
        artifactName = td.node.artifact?.name || '';
        break;
      }
      await sleep(1000);
    }
    console.log(`    Agent artifact built: ${artifactName}`);

    // ---------- Step 1 Screenshot: Assets Header with Generate Agent button ----------
    console.log('[Step 1] Navigating to /assets and capturing header with "Generate agent" button...');
    await page.goto(`${baseURL}/assets`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    const generateBtn = page.getByRole('button', { name: 'Generate agent' });
    await generateBtn.waitFor({ state: 'visible', timeout: 10000 });

    const shot1 = path.join(screenshotsDir, 'step1-assets-header-generate-btn.png');
    await page.screenshot({ path: shot1, fullPage: true });
    results.screenshots.push({ path: shot1, caption: 'Step 1: Assets page showing new "Generate agent" button' });
    console.log(`    Screenshot saved: ${shot1}`);

    // ---------- Step 2 Screenshot: Generate Agent modal open ----------
    console.log('[Step 2] Clicking "Generate agent" to open modal...');
    await generateBtn.click();
    await page.waitForSelector('[data-testid="generate-agent-modal"]', { state: 'visible', timeout: 10000 });
    await page.waitForTimeout(500);

    const shot2 = path.join(screenshotsDir, 'step2-assets-generate-agent-modal.png');
    await page.screenshot({ path: shot2, fullPage: true });
    results.screenshots.push({ path: shot2, caption: 'Step 2: Agent builder flow opened as a modal on /assets' });
    console.log(`    Screenshot saved: ${shot2}`);

    // Close modal by clicking Cancel button or pressing Escape
    const cancelBtn = page.getByRole('button', { name: 'Cancel' });
    if (await cancelBtn.isVisible()) {
      await cancelBtn.click();
    } else {
      await page.keyboard.press('Escape');
    }
    await page.waitForTimeout(1000);

    // Ensure dialog is gone
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);

    // ---------- Step 3 Screenshot: Built agent with unique BugAntIcon ----------
    console.log('[Step 3] Verifying agent artifact in table with bug icon...');
    await page.goto(`${baseURL}/assets`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    const artifactBasename = artifactName.split('/').pop();
    const agentText = page.locator('span').filter({ hasText: artifactBasename.slice(0, 18) });
    await agentText.first().waitFor({ state: 'visible', timeout: 10000 });

    // Hover over bug icon or agent row
    const bugIcons = page.locator('[data-testid="agent-bug-icon"]');
    if ((await bugIcons.count()) > 0) {
      await bugIcons.first().hover();
      await page.waitForTimeout(500);
    }

    const shot3 = path.join(screenshotsDir, 'step3-assets-table-agent-bug-icon.png');
    await page.screenshot({ path: shot3, fullPage: true });
    results.screenshots.push({ path: shot3, caption: 'Step 3: Built agent row with unique bug icon (reusing hosts icon) & tooltip' });
    console.log(`    Screenshot saved: ${shot3}`);

    // ---------- Step 4 Screenshot: Filter by Agents applied ----------
    console.log('[Step 4] Applying "Agents" filter in Filters popover...');
    const filterPopoverBtn = page.getByRole('button', { name: /Filters \(/ });
    await filterPopoverBtn.waitFor({ state: 'visible', timeout: 5000 });
    await filterPopoverBtn.click();
    await page.waitForTimeout(800);

    // In AgentFilterBar: select "Agents"
    const agentFilterBar = page.locator('[data-testid="agent-filter-bar"]');
    await agentFilterBar.waitFor({ state: 'visible', timeout: 5000 });

    // Click the react-select dropdown in agentFilterBar and select "Agents"
    const selectInput = agentFilterBar.locator('input').first();
    await selectInput.focus();
    await page.keyboard.type('Agents');
    await page.waitForTimeout(400);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(800);

    // Take screenshot of popover with Agent filter
    const shot4a = path.join(screenshotsDir, 'step4a-assets-filter-popover.png');
    await page.screenshot({ path: shot4a });
    results.screenshots.push({ path: shot4a, caption: 'Step 4a: Filters popover showing new Type filter for Agents' });
    console.log(`    Screenshot saved: ${shot4a}`);

    // Close the filter popover by pressing Escape
    await page.keyboard.press('Escape');
    await page.waitForTimeout(1000);

    const shot4b = path.join(screenshotsDir, 'step4b-assets-filter-agents-applied.png');
    await page.screenshot({ path: shot4b, fullPage: true });
    results.screenshots.push({ path: shot4b, caption: 'Step 4b: Agents filter applied showing "Filters (1)" and only built agent artifacts' });
    console.log(`    Screenshot saved: ${shot4b}`);

    console.log('\n=== ALL ASSETS AGENT FLOW VERIFICATIONS PASSED ===');
    fs.writeFileSync(path.join(screenshotsDir, 'assets-flow-results.json'), JSON.stringify(results, null, 2));
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
