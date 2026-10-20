import { chromium, request as pwRequest } from 'playwright';
import * as child_process from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../../../');
const screenshotsDir = path.resolve(__dirname, '../screenshots/full-flow');
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
  const results = { screenshots: [] };

  try {
    // 0. Clean up stale builders
    console.log('[0] Cleaning up stale builders...');
    const existing = await graphql(apiCtx, `query listBuilders { builders { edges { node { id } } } }`);
    for (const edge of existing?.builders?.edges ?? []) {
      try {
        await graphql(apiCtx, `mutation del($id: ID!) { deleteBuilder(builderID: $id) }`, { id: edge.node.id });
      } catch (e) {}
    }

    // Upload a sample asset so non-agent row is visible
    console.log('[0b] Uploading a standard non-agent asset...');
    try {
      await apiCtx.post(`${baseURL}/assets/upload`, {
        multipart: {
          file: {
            name: 'deployment-manifest.json',
            mimeType: 'application/json',
            buffer: Buffer.from('{"version":"1.0","service":"ops"}'),
          },
        },
      });
    } catch (e) {
      console.log('Upload notice:', e.message);
    }

    // 1. Enroll builder & start local daemon
    console.log('[1] Enrolling builder...');
    const registerData = await graphql(
      apiCtx,
      `mutation registerNewBuilder($input: CreateBuilderInput!) {
        registerBuilder(input: $input) {
          builder { id }
          config
        }
      }`,
      { input: { supportedTargets: ['PLATFORM_LINUX'], upstream: baseURL } }
    );
    const builderId = registerData.registerBuilder.builder.id;
    const configYaml = registerData.registerBuilder.config;

    configPath = `/tmp/builder-full-flow-${Date.now()}.yaml`;
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

    // ---------- Step 1: Assets page showing "Generate agent" button ----------
    console.log('[Step 1] Navigating to /assets...');
    await page.goto(`${baseURL}/assets`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    const generateBtn = page.getByRole('button', { name: 'Generate agent' });
    await generateBtn.waitFor({ state: 'visible', timeout: 10000 });

    const shot1 = path.join(screenshotsDir, '1-assets-header.png');
    await page.screenshot({ path: shot1, fullPage: true });
    results.screenshots.push({ path: shot1, caption: 'Step 1: Assets page with "Generate agent" action button in the header' });
    console.log(`    Saved: ${shot1}`);

    // ---------- Step 2: Open Generate Agent Modal (Step 1: Choose Profile) ----------
    console.log('[Step 2] Clicking "Generate agent"...');
    await generateBtn.click();
    await page.waitForSelector('[data-testid="generate-agent-modal"]', { state: 'visible', timeout: 10000 });
    await page.waitForTimeout(600);

    const shot2 = path.join(screenshotsDir, '2-modal-choose-profile.png');
    await page.screenshot({ path: shot2, fullPage: true });
    results.screenshots.push({ path: shot2, caption: 'Step 2: Generate agent modal opened to Step 1 (Choose profile) with "+ New profile" button' });
    console.log(`    Saved: ${shot2}`);

    // ---------- Step 3: Click "New profile" inside modal ----------
    console.log('[Step 3] Clicking "New profile"...');
    const newProfileBtn = page.getByRole('button', { name: 'New profile' });
    await newProfileBtn.waitFor({ state: 'visible', timeout: 5000 });
    await newProfileBtn.click();
    await page.waitForTimeout(600);

    const shot3 = path.join(screenshotsDir, '3-modal-new-profile-form.png');
    await page.screenshot({ path: shot3, fullPage: true });
    results.screenshots.push({ path: shot3, caption: 'Step 3: In-modal build profile authoring form rendered directly inside the wizard' });
    console.log(`    Saved: ${shot3}`);

    // ---------- Step 4: Fill in Profile Fields ----------
    console.log('[Step 4] Filling in profile form...');
    const nameInput = page.locator('input[name="profileName"]');
    await nameInput.waitFor({ state: 'visible', timeout: 10000 });
    await nameInput.fill('Stealth Operations Agent');

    const descInput = page.locator('textarea[placeholder="What is this profile for?"]');
    await descInput.fill('Modular Linux beacon profile created directly from the Assets modal workflow.');

    // Expand Advanced
    const advancedBtn = page.locator('button:has-text("Advanced")');
    await advancedBtn.click();
    await page.waitForTimeout(400);

    // Fill Build script, Setup script, and Artifact path for local compile
    const setupScriptArea = page.getByText('cd /home/vscode');
    await setupScriptArea.fill('echo "setup ready"');

    const buildScriptArea = page.getByText('{{.BuildCommand}}');
    await buildScriptArea.fill('cd "$REALM_WORKSPACE_DIR" && mkdir -p output && echo "int main(){return 0;}" | gcc -x c - -o output/imix');

    const artifactPathArea = page.getByText('{{.ArtifactPath}}');
    await artifactPathArea.fill('output/imix');

    const shot4 = path.join(screenshotsDir, '4-modal-profile-filled.png');
    await page.screenshot({ path: shot4, fullPage: true });
    results.screenshots.push({ path: shot4, caption: 'Step 4: Profile details configured (Name, Description, and build scripts under Advanced)' });
    console.log(`    Saved: ${shot4}`);

    // ---------- Step 5: Save Profile ----------
    console.log('[Step 5] Saving profile...');
    const saveProfileBtn = page.getByRole('button', { name: 'Save profile' });
    await saveProfileBtn.click();

    // Wait for the modal to return to Step 1 with the new profile selected
    await page.waitForSelector('text=Stealth Operations Agent', { timeout: 10000 });
    await page.waitForTimeout(600);

    const shot5 = path.join(screenshotsDir, '5-modal-profile-selected.png');
    await page.screenshot({ path: shot5, fullPage: true });
    results.screenshots.push({ path: shot5, caption: 'Step 5: Profile saved and automatically selected in Step 1' });
    console.log(`    Saved: ${shot5}`);

    // ---------- Step 6: Step 2 Compilation Target ----------
    console.log('[Step 6] Clicking Continue to Step 2 (Target)...');
    const continueBtn1 = page.getByRole('button', { name: 'Continue' });
    await continueBtn1.click();
    await page.waitForTimeout(600);

    const shot6 = path.join(screenshotsDir, '6-modal-choose-target.png');
    await page.screenshot({ path: shot6, fullPage: true });
    results.screenshots.push({ path: shot6, caption: 'Step 2: Choose compilation target (Linux x86_64 Executable)' });
    console.log(`    Saved: ${shot6}`);

    // ---------- Step 7: Step 3 Confirm & Build ----------
    console.log('[Step 7] Clicking Continue to Step 3 (Confirm)...');
    const continueBtn2 = page.getByRole('button', { name: 'Continue' });
    await continueBtn2.click();
    await page.waitForTimeout(600);

    const shot7 = path.join(screenshotsDir, '7-modal-confirm-build.png');
    await page.screenshot({ path: shot7, fullPage: true });
    results.screenshots.push({ path: shot7, caption: 'Step 3: Review configuration summary and submit "Build agent"' });
    console.log(`    Saved: ${shot7}`);

    // ---------- Step 8: Build Agent & Toast ----------
    console.log('[Step 8] Clicking "Start build"...');
    const buildBtn = page.getByRole('button', { name: 'Start build' });
    await buildBtn.click();

    // Wait for toast notification
    await page.waitForSelector('text=Agent build started', { timeout: 10000 });
    await page.waitForTimeout(400);

    const shot8 = path.join(screenshotsDir, '8-build-queued-toast.png');
    await page.screenshot({ path: shot8, fullPage: true });
    results.screenshots.push({ path: shot8, caption: 'Step 8: Build task queued with success toast notification and modal auto-closing' });
    console.log(`    Saved: ${shot8}`);

    // Wait for modal to dismiss
    await page.waitForTimeout(1500);

    // ---------- Step 9: Wait for build completion & show artifact with black bug icon ----------
    console.log('[Step 9] Waiting for build task to complete...');
    for (let i = 0; i < 40; i++) {
      const taskList = await graphql(
        apiCtx,
        `query listTasks {
          buildTasks(first: 5) {
            edges {
              node {
                id
                finishedAt
                artifact { id name }
              }
            }
          }
        }`
      );
      const edge = taskList?.buildTasks?.edges?.[0];
      if (edge?.node?.finishedAt && edge?.node?.artifact?.name) {
        console.log(`    Build finished! Artifact: ${edge.node.artifact.name}`);
        break;
      }
      await sleep(1000);
    }

    // Refresh Assets page
    await page.goto(`${baseURL}/assets`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    // Hover over the black bug icon in the table
    const bugIcons = page.locator('[data-testid="agent-bug-icon"]');
    await bugIcons.first().waitFor({ state: 'visible', timeout: 10000 });
    await bugIcons.first().hover();
    await page.waitForTimeout(500);

    const shot9 = path.join(screenshotsDir, '9-assets-built-agent-icon.png');
    await page.screenshot({ path: shot9, fullPage: true });
    results.screenshots.push({ path: shot9, caption: 'Step 9: Newly built agent artifact displayed in Assets table with black bug icon and tooltip' });
    console.log(`    Saved: ${shot9}`);

    // ---------- Step 10: Open Filter Popover & Select "Agents" ----------
    console.log('[Step 10] Filtering by Type: Agents...');
    const filterPopoverBtn = page.getByRole('button', { name: /Filters \(/ });
    await filterPopoverBtn.waitFor({ state: 'visible', timeout: 5000 });
    await filterPopoverBtn.click();
    await page.waitForTimeout(800);

    const agentFilterBar = page.locator('[data-testid="agent-filter-bar"]');
    await agentFilterBar.waitFor({ state: 'visible', timeout: 5000 });

    const selectInput = agentFilterBar.locator('input').first();
    await selectInput.focus();
    await page.keyboard.type('Agents');
    await page.waitForTimeout(400);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(800);

    const shot10 = path.join(screenshotsDir, '10-filter-popover-agents.png');
    await page.screenshot({ path: shot10 });
    results.screenshots.push({ path: shot10, caption: 'Step 10: Assets filter popover showing "Type" with black bug icon and "Agents" selected' });
    console.log(`    Saved: ${shot10}`);

    // ---------- Step 11: Filtered Assets Table View ----------
    console.log('[Step 11] Dismissing popover to show filtered assets table...');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(1000);

    const shot11 = path.join(screenshotsDir, '11-assets-table-filtered.png');
    await page.screenshot({ path: shot11, fullPage: true });
    results.screenshots.push({ path: shot11, caption: 'Step 11: Filtered Assets view showing "Filters (1)" and only built agent artifacts' });
    console.log(`    Saved: ${shot11}`);

    console.log('\n=== FULL ASSETS AGENT & PROFILE CREATION FLOW COMPLETED SUCCESSFULLY ===');
    fs.writeFileSync(path.join(screenshotsDir, 'results.json'), JSON.stringify(results, null, 2));
  } finally {
    await browser.close();
    if (builderProcess) {
      builderProcess.kill('SIGTERM');
      await sleep(1000);
      try { builderProcess.kill('SIGKILL'); } catch (_) {}
    }
    if (configPath && fs.existsSync(configPath)) {
      try { fs.unlinkSync(configPath); } catch (_) {}
    }
    await apiCtx.dispose();
  }
}

main().catch((err) => {
  console.error('FLOW FAILED:', err);
  process.exit(1);
});
