import { test, expect } from '@playwright/test';

test('End-to-end shellv2 repl test', async ({ page, context }) => {
  // 1. Navigate to /hosts and wait for the host list to load
  console.log('Navigating to /hosts');
  await page.goto('/hosts');

  // Wait for a host row to appear (the agent should have registered by now)
  console.log('Waiting for hosts to load');
  const hostRow = page.locator('[role="button"]').first();
  await expect(hostRow).toBeVisible({ timeout: 15000 });

  // 2. Click on the host to navigate to host-details
  console.log('Navigating to host details');
  await hostRow.click();

  // 3. Click "Create Shell" button — this calls a mutation and opens /shellv2/:id in a new tab
  console.log('Creating shell');
  const createShellButton = page.getByLabel('Create Shell');
  await expect(createShellButton).toBeVisible({ timeout: 15000 });

  // Listen for the new page (tab) that will be opened by window.open
  const shellPagePromise = context.waitForEvent('page');
  await createShellButton.click();

  // 4. Switch to the new shellv2 tab
  const shellPage = await shellPagePromise;
  await shellPage.waitForLoadState();
  console.log('Switched to shellv2 tab');

  // 5. Wait for the terminal to render and the connection to be established
  console.log('Waiting for terminal');
  await expect(shellPage.locator('.xterm-rows')).toBeVisible({ timeout: 20000 });

  // Wait for "Connected to Tavern" which indicates the WebSocket is ready
  await expect(shellPage.locator('.xterm-rows')).toContainText('Connected to Tavern', { timeout: 20000 });

  // Wait for the initial prompt
  await expect(shellPage.locator('.xterm-rows')).toContainText('>>>', { timeout: 10000 });

  // 6. Focus the terminal and send a command
  console.log('Sending command: print("Hello E2E")');
  await shellPage.locator('.xterm-rows').click();
  await shellPage.keyboard.type('print("Hello E2E")', { delay: 100 });
  await shellPage.waitForTimeout(100);
  await shellPage.keyboard.press('Enter');

  // 7. Verify the output — non-interactive shell queues and returns output
  // Expected output pattern:
  //   [*] Task Queued for <beacon-name>
  //   [+] print("Hello E2E")
  //   Hello E2E
  //   >>>
  console.log('Verifying output');
  await expect(shellPage.locator('.xterm-rows')).toContainText('Hello E2E', { timeout: 30000 });
  // Verify a new prompt appeared after the output (command completed)
  // The prompt after output confirms the round-trip worked
  await expect(shellPage.locator('.xterm-rows')).toContainText('>>>', { timeout: 10000 });

  console.log('Test Complete');
});
