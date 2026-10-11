// AC1: "A mocked smoke spec passes in Playwright WebKit and Chromium." Not named in this task's
// own Outputs (only the support files are) -- a spec file has to exist somewhere for that AC to
// mean anything at all, so this is a real, necessary addition, not scope creep.
import { test, expect } from '../support/fixtures';

test('the app shell loads, the sidebar renders, and a fixture project reaches the screen', async ({ mockedPage }) => {
  await mockedPage.goto('/');

  await expect(mockedPage.getByRole('navigation', { name: 'Primary' })).toBeVisible();

  // fixtures/dto/projects.json names "poirot" among others -- a real project, through a real
  // mocked list_projects/list_sessions round trip, not just the sidebar's own empty shell.
  await expect(mockedPage.getByText('poirot', { exact: false }).first()).toBeVisible();
});
