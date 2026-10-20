import { chromium, request as pwRequest } from 'playwright';
import * as child_process from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../../../');
const screenshotsDir = path.resolve(__dirname, '../screenshots/admin-tabs');
const baseURL = 'http://127.0.0.1:8000';

fs.mkdirSync(screenshotsDir, { recursive: true });

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function main() {
  const apiCtx = await pwRequest.newContext({ baseURL });
  const browser = await chromium.launch({ headless: true });
  const pageCtx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await pageCtx.newPage();

  let tavernProcess = null;
  const results = { screenshots: [] };

  try {
    const tavernBin = process.env.TAVERN_BIN || '/tmp/tavern_bin';
    try {
      await apiCtx.get(`${baseURL}/assets`);
    } catch (_) {
      console.log('Starting local tavern server...');
      tavernProcess = child_process.spawn(
        tavernBin,
        [],
        { cwd: repoRoot, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, MYSQL_ADDR: '', HTTP_LISTEN_ADDR: '127.0.0.1:8000' } }
      );
      for (let i = 0; i < 30; i++) {
        try {
          const res = await apiCtx.get(`${baseURL}/assets`);
          if (res.ok()) break;
        } catch (_) {}
        await sleep(1000);
      }
    }

    // 1. Visit /admin -> default Users tab
    console.log('[1] Visiting /admin (Users tab)...');
    await page.goto(`${baseURL}/admin`, { waitUntil: 'networkidle' });
    await page.waitForSelector('text=Users');
    await sleep(1500);

    const usersTabScreenshot = path.join(screenshotsDir, '1-admin-users-tab.png');
    await page.screenshot({ path: usersTabScreenshot });
    results.screenshots.push({ step: 'Users tab on Admin portal', file: usersTabScreenshot });
    console.log(`    Captured: ${usersTabScreenshot}`);

    // Verify sidebar has Admin and no Agent Builder
    const sidebarText = await page.innerText('nav');
    console.log(`    Sidebar contains Admin: ${sidebarText.includes('Admin')}`);
    console.log(`    Sidebar contains Agent Builder: ${sidebarText.includes('Agent Builder')}`);

    // 2. Click Agents tab
    console.log('[2] Switching to Agents tab...');
    const agentsTab = page.locator('role=tab[name*="Agents"]');
    await agentsTab.click();
    await sleep(1500);

    const agentsTabScreenshot = path.join(screenshotsDir, '2-admin-agents-tab.png');
    await page.screenshot({ path: agentsTabScreenshot });
    results.screenshots.push({ step: 'Agents tab on Admin portal', file: agentsTabScreenshot });
    console.log(`    Captured: ${agentsTabScreenshot}`);

    // 3. Click Profiles tab
    console.log('[3] Switching to Profiles tab...');
    const profilesTab = page.locator('role=tab[name*="Profiles"]');
    await profilesTab.click();
    await sleep(1500);

    const profilesTabScreenshot = path.join(screenshotsDir, '3-admin-profiles-tab.png');
    await page.screenshot({ path: profilesTabScreenshot });
    results.screenshots.push({ step: 'Profiles tab on Admin portal', file: profilesTabScreenshot });
    console.log(`    Captured: ${profilesTabScreenshot}`);

    // Open Add Profile modal
    console.log('[3b] Opening Add Build Profile modal...');
    const addProfileBtn = page.locator('button:has-text("Add build profile")').first();
    if (await addProfileBtn.isVisible()) {
      await addProfileBtn.click();
      await sleep(1000);
      const modalScreenshot = path.join(screenshotsDir, '3b-admin-new-profile-modal.png');
      await page.screenshot({ path: modalScreenshot });
      results.screenshots.push({ step: 'New Build Profile modal on Admin', file: modalScreenshot });
      console.log(`    Captured: ${modalScreenshot}`);
      // Close modal
      const cancelBtn = page.locator('button:has-text("Cancel")');
      if (await cancelBtn.isVisible()) {
        await cancelBtn.click();
        await sleep(500);
      }
    }

    // 4. Click Builders tab
    console.log('[4] Switching to Builders tab...');
    const buildersTab = page.locator('role=tab[name*="Builders"]');
    await buildersTab.click();
    await sleep(1500);

    const buildersTabScreenshot = path.join(screenshotsDir, '4-admin-builders-tab.png');
    await page.screenshot({ path: buildersTabScreenshot });
    results.screenshots.push({ step: 'Builders tab on Admin portal', file: buildersTabScreenshot });
    console.log(`    Captured: ${buildersTabScreenshot}`);

    // Open Enroll Builder modal
    console.log('[4b] Opening Enroll Builder modal...');
    const enrollBuilderBtn = page.locator('button:has-text("Enroll builder")').first();
    if (await enrollBuilderBtn.isVisible()) {
      await enrollBuilderBtn.click();
      await sleep(1000);
      const enrollModalScreenshot = path.join(screenshotsDir, '4b-admin-enroll-builder-modal.png');
      await page.screenshot({ path: enrollModalScreenshot });
      results.screenshots.push({ step: 'Enroll Builder modal on Admin', file: enrollModalScreenshot });
      console.log(`    Captured: ${enrollModalScreenshot}`);
      // Close modal
      const cancelBtn = page.locator('button:has-text("Cancel")');
      if (await cancelBtn.isVisible()) {
        await cancelBtn.click();
        await sleep(500);
      }
    }

    // 5. Test /builder redirect to /admin?tab=agents
    console.log('[5] Testing /builder redirect...');
    await page.goto(`${baseURL}/builder`, { waitUntil: 'networkidle' });
    await sleep(1500);
    console.log(`    URL after navigating to /builder: ${page.url()}`);

    const redirectScreenshot = path.join(screenshotsDir, '5-builder-redirect-to-admin-agents.png');
    await page.screenshot({ path: redirectScreenshot });
    results.screenshots.push({ step: 'Redirect /builder to /admin?tab=agents', file: redirectScreenshot });
    console.log(`    Captured: ${redirectScreenshot}`);

    fs.writeFileSync(path.join(screenshotsDir, 'results.json'), JSON.stringify(results, null, 2));
    console.log('Admin tabs verification completed successfully!');
  } catch (err) {
    console.error('Error during admin tabs verification:', err);
    process.exit(1);
  } finally {
    await browser.close();
    if (tavernProcess) {
      tavernProcess.kill('SIGTERM');
    }
  }
}

main();
