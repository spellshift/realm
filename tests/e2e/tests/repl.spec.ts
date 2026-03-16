import { test, expect } from '@playwright/test';

test('End-to-end reverse shell repl test', async ({ page }) => {
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

  // 3. Select the "Reverse Shell REPL" tome and click "continue"
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();

  // Search for the tome first (virtualized table doesn't show all items)
  const dialog = page.locator('[role="dialog"]');
  const searchInput = dialog.getByPlaceholder('Tome name, description & params');
  await searchInput.fill('Reverse Shell REPL');
  await page.waitForTimeout(500); // Wait for search results to filter

  // Click the tome row (role="button") containing the tome name - scoped to dialog
  const tomeRow = dialog.locator('[role="button"]').filter({ hasText: 'Reverse Shell REPL' });
  await expect(tomeRow).toBeVisible();
  await tomeRow.click();

  console.log('Clicking Continue (Tome)');
  await page.locator('[aria-label="continue tome step"]').click();

  // 4. Select "Submit"
  console.log('Submitting Quest');
  await page.locator('[aria-label="submit quest"]').click();

  // 5. Wait at least 11 seconds for agent execution
  console.log('Waiting 12s for execution');
  await page.waitForTimeout(12000);

  // 6. Refresh the page
  console.log('Reloading page');
  await page.reload();

  // 7. See a "Shells" tab in the output, select it
  console.log('Clicking Shells tab');
  await page.getByRole('tab', { name: 'Shells' }).click();

  // 8. Select "Join shell instance"
  console.log('Joining shell instance');
  await page.getByRole('button', { name: 'Join shell instance' }).click();

  // 9. Perform tests to ensure it functions as expected
  console.log('Verifying shell connection');

  // Wait for terminal to be visible
  await expect(page.locator('#terminal')).toBeVisible({ timeout: 15000 });

  // Focus the terminal (clicking it helps ensure focus)
  await page.locator('#terminal').click();

  console.log('Sending command');
  // Type something.
  // Wait for the prompt to ensure the session is ready
  await expect(page.locator('.xterm-rows')).toContainText('>>>', { timeout: 20000 });
  await page.keyboard.type('print("Hello E2E")', { delay: 100 });
  await page.waitForTimeout(100);
  await page.keyboard.press('Enter');

  // Verify output.
  console.log('Verifying output');
  // xterm rows usually contain the text.
  // We expect the command echoed, output, and a new prompt.
  // The prompt might contain trailing spaces or non-breaking spaces depending on rendering.
  await expect(page.locator('.xterm-rows')).toContainText('Hello E2E');
  await expect(page.locator('.xterm-rows')).toContainText('>>>');

  console.log('Test Complete');
});
