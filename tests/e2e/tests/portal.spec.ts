import { test, expect } from '@playwright/test';

test('End-to-end portal provisioning test', async ({ page }) => {
  // Navigate to quests page and open the Create Quest modal
  console.log('Navigating to /quests');
  await page.goto('/quests');

  // Click "Create a quest" button to open the modal
  console.log('Opening Create Quest modal');
  await page.getByRole('button', { name: 'Create a quest' }).click();

  // Select the only visible beacon and click "continue"
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

  // Select the first beacon checkbox using aria-label (Chakra Checkbox with aria-label="Select beacon {name}")
  const beaconCheckbox = page.getByLabel(/Select beacon/).first();
  await expect(beaconCheckbox).toBeVisible();

  // Select the beacon
  console.log('Selecting beacon');
  await beaconCheckbox.check({ force: true });

  // Click Continue
  console.log('Clicking Continue (Beacon)');
  await page.locator('[aria-label="continue beacon step"]').click();

  // 3. Select the "SOCKS5 Relay" tome and click "continue"
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();
  // Click the tome row (role="button") containing the tome name
  const tomeRow = page.locator('[role="button"]').filter({ hasText: 'SOCKS5 Relay' });
  await expect(tomeRow).toBeVisible();
  await tomeRow.click();

  console.log('Clicking Continue (Tome)');
  await page.locator('[aria-label="continue tome step"]').click();

  // 4. Select "Submit"
  console.log('Submitting Quest');
  await page.locator('[aria-label="submit quest"]').click();

  // 5. Wait for execution.
  // We expect "Portal created" to appear in the output.
  console.log('Waiting for "Portal created" message');
  await page.waitForTimeout(12000);
  await page.reload({timeout: 30000});

  // Use aria-label to find the specific output panel and check for "Portal created" within it
  const outputPanel = page.locator('[aria-label="task output"]');
  await expect(outputPanel.getByText('Portal created')).toBeVisible({ timeout: 30000 });

  console.log('Portal created successfully');
});
