import { test, expect } from '@playwright/test';

test.describe('Tome Execution Timeout Tests', () => {
  const tomesToTest = ['Process list', 'Netstat', 'Get network info'];

  for (const tome of tomesToTest) {
    test(`End-to-end execution of ${tome} tome`, async ({ page }) => {
      // 1. Navigate to /createQuest
      console.log(`Navigating to /createQuest for ${tome}`);
      await page.goto('/createQuest');

      // 2. Wait for beacons to load
      console.log('Waiting for beacons to load');
      await expect(page.getByText('Loading beacons...')).toBeHidden({ timeout: 15000 });

      // Select the first beacon
      const beacons = page.locator('.chakra-card input[type="checkbox"]');
      await expect(beacons.first()).toBeVisible({ timeout: 10000 });
      console.log('Selecting beacon');
      await beacons.first().check({ force: true });

      // Click Continue (Beacon Step)
      console.log('Clicking Continue (Beacon)');
      await page.getByRole('button', { name: 'Continue' }).click();

      // 3. Select the target tome
      console.log(`Selecting Tome: ${tome}`);
      await expect(page.getByText('Loading tomes...')).toBeHidden();
      await expect(page.getByText(tome)).toBeVisible({ timeout: 10000 });
      await page.getByText(tome).click();

      // Click Continue (Tome Step)
      console.log('Clicking Continue (Tome)');
      await page.getByRole('button', { name: 'Continue' }).click();

      // 4. Submit Quest
      console.log('Submitting Quest');
      await page.getByRole('button', { name: 'Submit' }).click();

      // 5. Wait for execution
      // The tomes hang due to a bug, so we want to wait the full timeout (15s) and check they have NOT finished.
      console.log('Waiting for execution (up to 15s)');
      await page.waitForTimeout(15000);

      // Reload to refresh the task status
      await page.reload();

      // 6. Check that the task has NOT finished
      console.log('Checking that task did NOT finish');

      // The TaskTimeStamp component renders "Finished at <time> on <date>" when execFinishedAt is set.
      // We expect it NOT to be visible because the tome execution should have timed out/hung.
      // This assertion validates the bug is correctly caught by the test.
      const finishedAtText = page.getByText(/Finished at/);
      await expect(finishedAtText).not.toBeVisible();

      console.log(`Test for ${tome} Complete`);
    });
  }
});
