import { expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const workflow = Bun.YAML.parse(readFileSync(new URL('../../.github/workflows/website.yml', import.meta.url), 'utf8'));
const release = Bun.YAML.parse(readFileSync(new URL('../../.github/workflows/build.yml', import.meta.url), 'utf8'));

function allows(condition, github) {
  return runInNewContext(condition, { github });
}

function event(name, ref = 'refs/heads/main', run = {}) {
  return {
    event_name: name, ref, repository: 'WhoSowSee/mdv',
    event: { workflow_run: { conclusion: 'success', event: 'push', head_repository: { full_name: 'WhoSowSee/mdv' }, ...run } },
  };
}

test('checks main pushes and pull requests and follows the actual release workflow', () => {
  expect(workflow.on.push.branches).toEqual(['main']);
  expect(workflow.on.pull_request.branches).toEqual(['main']);
  expect(Object.hasOwn(workflow.on, 'workflow_dispatch')).toBe(true);
  expect(workflow.on.workflow_run).toEqual({ workflows: [release.name], types: ['completed'] });
  expect(workflow.on.release).toBeUndefined();
  expect(workflow.on.pull_request_target).toBeUndefined();
  expect(release.on.push.tags).toBeDefined();
  expect(release.on.push.branches).toBeUndefined();
});

test('skips unsuccessful releases, manual package checks and runs from other repositories', () => {
  const condition = workflow.jobs.build.if;
  for (const name of ['push', 'pull_request', 'workflow_dispatch', 'workflow_run']) {
    expect(allows(condition, event(name)), name).toBe(true);
  }
  for (const conclusion of ['failure', 'cancelled', 'skipped', 'timed_out']) {
    expect(allows(condition, event('workflow_run', undefined, { conclusion })), conclusion).toBe(false);
  }
  expect(allows(condition, event('workflow_run', undefined, { event: 'workflow_dispatch' }))).toBe(false);
  expect(allows(condition, event('workflow_run', undefined, { head_repository: { full_name: 'fork/mdv' } }))).toBe(false);
});

test('publishes only successful main builds with write permissions isolated to deployment', () => {
  const { build, deploy } = workflow.jobs;
  const upload = build.steps.find((step) => step.uses?.startsWith('actions/upload-pages-artifact@'));
  expect(upload.if).toBe(deploy.if);
  for (const name of ['push', 'workflow_dispatch', 'workflow_run']) {
    expect(allows(deploy.if, event(name)), name).toBe(true);
  }
  expect(allows(deploy.if, event('pull_request'))).toBe(false);
  expect(allows(deploy.if, event('workflow_dispatch', 'refs/heads/feature'))).toBe(false);
  expect(allows(deploy.if, event('workflow_dispatch', 'refs/tags/v6.1.0'))).toBe(false);
  expect(deploy.needs).toBe('build');
  expect(workflow.permissions).toEqual({ contents: 'read' });
  expect(build.permissions).toBeUndefined();
  expect(deploy.permissions).toEqual({ pages: 'write', 'id-token': 'write' });
  expect(deploy.environment.name).toBe('github-pages');
  const checkout = build.steps.find((step) => step.uses?.startsWith('actions/checkout@'));
  expect(checkout.with).toEqual({ 'persist-credentials': false });
});

test('uses the website lockfile, manifest and build script and uploads the generated directory', () => {
  const steps = workflow.jobs.build.steps;
  expect(workflow.defaults.run).toEqual({ shell: 'bash', 'working-directory': 'website' });
  expect(steps.find((step) => step.uses?.startsWith('oven-sh/setup-bun@')).with['bun-version-file']).toBe('website/package.json');
  expect(steps.filter((step) => step.run).map((step) => step.run)).toEqual([
    'bun install --frozen-lockfile', 'bun run test', 'bun run build',
  ]);
  expect(steps.find((step) => step.uses?.startsWith('actions/upload-pages-artifact@')).with.path).toBe('website/dist');
  expect(workflow.concurrency['cancel-in-progress']).toBe("${{ github.event_name == 'pull_request' }}");
});
