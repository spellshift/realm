import { test, expect } from '@playwright/test';

// Set timeout for this specific test file to 60 seconds
test.setTimeout(60000);

test('End-to-end reverse shell repl test', async ({ page }) => {
  // 1. Connect to tavern's UI using playwright at http://127.0.0.1:8000/createQuest
  console.log('Navigating to /createQuest');
  await page.goto('/createQuest');

  // 2. Select the only visible beacon and click "continue"
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

  // Select the checkbox. Using force: true because Chakra UI hides the actual input.
  // We use .locator('.chakra-card input[type="checkbox"]') to avoid selecting filter switches.
  console.log('Selecting beacon');
  await page.locator('.chakra-card input[type="checkbox"]').first().check({ force: true });

  // Click Continue
  console.log('Clicking Continue (Beacon)');
  await page.getByRole('button', { name: 'Continue' }).click();

  // 3. Select the "Reverse Shell Repl" tome and click "continue"
  console.log('Selecting Tome');
  await expect(page.getByText('Loading tomes...')).toBeHidden();
  await page.getByText('Reverse Shell Repl').click();

  console.log('Clicking Continue (Tome)');
  await page.getByRole('button', { name: 'Continue' }).click();

  // 4. Select "Submit"
  console.log('Submitting Quest');
  await page.getByRole('button', { name: 'Submit' }).click();

  // 5. Wait for agent execution. We poll by reloading the page until the "Shells" tab appears.
  console.log('Waiting for execution and Shells tab...');

  await expect(async () => {
    console.log('Reloading page to check for Shells tab...');
    await page.reload();
    // Check if the shells tab exists. We use a short timeout here because we want to fail fast and retry reloading.
    await expect(page.getByRole('tab', { name: 'Shells' })).toBeVisible({ timeout: 2000 });
  }).toPass({
    // We wait up to 45 seconds for the shell to appear.
    // The agent interval is 5s, plus execution time, plus network latency.
    timeout: 45000,
    intervals: [2000, 5000] // Retry after 2s, then every 5s
  });

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
  await page.locator('.xterm-cursor-layer').click();

  console.log('Sending command');
  // Type something.
  await page.waitForTimeout(1000); // Wait a bit for connection to be fully established
  await page.keyboard.type('print("Hello E2E")');
  await page.keyboard.press('Enter');

  // Verify output.
  console.log('Verifying output');
  // xterm rows usually contain the text.
  await expect(page.locator('.xterm-rows')).toContainText('Hello E2E', { timeout: 10000 });

  console.log('Test Complete');
});
