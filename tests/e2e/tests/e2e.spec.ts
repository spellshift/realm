import { test, expect } from '@playwright/test';

// Set timeout for this specific test file to 120 seconds to allow for retries
test.setTimeout(120000);

test('End-to-end reverse shell repl test', async ({ page }) => {
  // Define the quest creation and waiting flow
  const createQuestAndWaitForShell = async (attempt: number) => {
    console.log(`Attempt ${attempt}: Navigating to /createQuest`);
    await page.goto('/createQuest');

    // Select the only visible beacon and click "continue"
    console.log(`Attempt ${attempt}: Waiting for beacons to load`);
    await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

    // Select the checkbox. Using force: true because Chakra UI hides the actual input.
    // We use .locator('.chakra-card input[type="checkbox"]') to avoid selecting filter switches.
    console.log(`Attempt ${attempt}: Selecting beacon`);
    await page.locator('.chakra-card input[type="checkbox"]').first().check({ force: true });

    // Click Continue
    console.log(`Attempt ${attempt}: Clicking Continue (Beacon)`);
    await page.getByRole('button', { name: 'Continue' }).click();

    // Select the "Reverse Shell Repl" tome and click "continue"
    console.log(`Attempt ${attempt}: Selecting Tome`);
    await expect(page.getByText('Loading tomes...')).toBeHidden();
    await page.getByText('Reverse Shell Repl').click();

    console.log(`Attempt ${attempt}: Clicking Continue (Tome)`);
    await page.getByRole('button', { name: 'Continue' }).click();

    // Select "Submit"
    console.log(`Attempt ${attempt}: Submitting Quest`);
    await page.getByRole('button', { name: 'Submit' }).click();

    // Wait for agent execution. We poll by reloading the page until the "Shells" tab appears.
    console.log(`Attempt ${attempt}: Waiting for execution and Shells tab...`);

    await expect(async () => {
      console.log(`Attempt ${attempt}: Reloading page to check for Shells tab...`);
      await page.reload();
      // Check if the shells tab exists. We use a short timeout here because we want to fail fast and retry reloading.
      await expect(page.getByRole('tab', { name: 'Shells' })).toBeVisible({ timeout: 2000 });
    }).toPass({
      // We wait up to 30 seconds for the shell to appear in this attempt.
      timeout: 30000,
      intervals: [2000, 5000] // Retry after 2s, then every 5s
    });
  };

  // Main retry loop
  let success = false;
  // Try up to 3 times (initial + 2 retries)
  for (let attempt = 1; attempt <= 3; attempt++) {
    try {
      await createQuestAndWaitForShell(attempt);
      success = true;
      break; // Success!
    } catch (e) {
      console.log(`Attempt ${attempt} failed: ${e}`);
      if (attempt === 3) {
        // If the last attempt fails, re-throw the error to fail the test
        throw e;
      }
      console.log('Retrying quest creation...');
    }
  }

  // See a "Shells" tab in the output, select it
  console.log('Clicking Shells tab');
  await page.getByRole('tab', { name: 'Shells' }).click();

  // Select "Join shell instance"
  console.log('Joining shell instance');
  await page.getByRole('button', { name: 'Join shell instance' }).click();

  // Perform tests to ensure it functions as expected
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
