import { test, expect } from '@playwright/test';

test('End-to-end reverse shell repl test', async ({ page }) => {
  // Increase test timeout to 120 seconds to accommodate slow CI environments and agent startup
  test.setTimeout(120000);

  // 1. Connect to tavern's UI using playwright at http://127.0.0.1:8000/createQuest
  console.log('Navigating to /createQuest');
  await page.goto('/createQuest');

  // 2. Select the only visible beacon and click "continue"
  console.log('Waiting for beacons to load');
  await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

  // Select the checkbox. Using force: true because Chakra UI hides the actual input.
  console.log('Selecting beacon');
  // Avoid selecting the filter switches by targeting the option container
  await page.locator('.option-container input[type="checkbox"]').first().check({ force: true });

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

  // 5. Wait for agent execution and polling for Shells tab
  console.log('Waiting for agent execution and checking for Shells tab...');

  // Polling loop to wait for Shells tab
  // Agent check-in and task execution might take time, especially cold start
  const maxRetries = 20;
  const retryInterval = 5000; // 5 seconds
  let shellTabFound = false;

  for (let i = 0; i < maxRetries; i++) {
    console.log(`Attempt ${i + 1}/${maxRetries}: Waiting ${retryInterval}ms...`);
    await page.waitForTimeout(retryInterval);

    console.log('Reloading page');
    await page.reload({ waitUntil: 'networkidle' });

    // Check if we are still on the same quest details page or redirected
    // The "Shells" tab appears in the Task results area
    const shellsTab = page.getByRole('tab', { name: 'Shells' });

    // Use a short timeout for the visibility check to avoid hanging the loop
    if (await shellsTab.isVisible({ timeout: 1000 }).catch(() => false)) {
      console.log('Shells tab found!');
      shellTabFound = true;
      await shellsTab.click();
      break;
    } else {
      console.log('Shells tab not found yet.');
    }
  }

  if (!shellTabFound) {
    throw new Error('Timed out waiting for Shells tab to appear.');
  }

  // 8. Select "Join shell instance"
  console.log('Joining shell instance');
  await page.getByRole('button', { name: 'Join shell instance' }).click();

  // 9. Perform tests to ensure it functions as expected
  console.log('Verifying shell connection');

  // Wait for terminal to be visible
  await expect(page.locator('#terminal')).toBeVisible({ timeout: 30000 });

  // Focus the terminal (clicking it helps ensure focus)
  await page.locator('.xterm-cursor-layer').click();

  console.log('Sending command');
  // Type something.
  await page.waitForTimeout(5000); // Wait a bit more for connection to be fully established
  await page.keyboard.type('print("Hello E2E")');
  await page.keyboard.press('Enter');

  // Verify output.
  console.log('Verifying output');
  // xterm rows usually contain the text.
  await expect(page.locator('.xterm-rows')).toContainText('Hello E2E', { timeout: 30000 });

  console.log('Test Complete');
});
