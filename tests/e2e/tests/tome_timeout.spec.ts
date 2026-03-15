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
      // The tomes should finish within 10 seconds. We'll wait 12s to be safe.
      console.log('Waiting for execution (up to 15s)');
      await page.waitForTimeout(15000);

      // Reload to refresh output
      await page.reload();

      // 6. Check output
      const outputPanel = page.locator('[aria-label="task output"]');
      await expect(outputPanel).toBeVisible({ timeout: 10000 });

      // If it doesn't finish, it won't have the typical successful task output structure,
      // or it might still be in a pending/running state.
      // We check that the output panel has content indicating completion.
      // In the Tavern UI, a completed task usually shows its textual output.
      // We expect it to NOT be empty or stuck in "Running...".

      const text = await outputPanel.innerText();
      console.log(`Output for ${tome}:`, text.substring(0, 200) + '...');

      // We expect the script to actually return something (like a table, IP, or process names).
      expect(text.trim().length).toBeGreaterThan(0);

      // Specific checks per tome can be added here if needed, but a non-empty output
      // after 15 seconds strongly implies it didn't hang indefinitely (which would
      // result in no output or a timeout error from the backend if it was killed).
      if (tome === 'Process list') {
          expect(text).toContain('PID');
      } else if (tome === 'Netstat') {
          expect(text).toContain('PROTO');
      } else if (tome === 'Get network info') {
          expect(text).toContain('IFACE');
      }

      console.log(`Test for ${tome} Complete`);
    });
  }
});
