const assert = require('node:assert/strict');
const { readFile } = require('node:fs/promises');
const path = require('node:path');
const { describe, it } = require('node:test');

const projectRoot = path.resolve(__dirname, '..');
const capabilityDirectory = path.join(projectRoot, 'src-tauri', 'capabilities');

const readCapability = async (name) =>
  JSON.parse(
    await readFile(path.join(capabilityDirectory, `${name}.json`), 'utf8'),
  );

describe('Tauri event capabilities', () => {
  for (const role of ['companion', 'preferences']) {
    it(`${role} can subscribe but cannot emit`, async () => {
      const capability = await readCapability(role);
      const permissions = new Set(capability.permissions);

      assert.equal(permissions.has('core:event:allow-listen'), true);
      assert.equal(permissions.has('core:event:allow-unlisten'), true);
      assert.equal(permissions.has('core:event:default'), false);
      assert.equal(permissions.has('core:event:allow-emit'), false);
      assert.equal(permissions.has('core:event:allow-emit-to'), false);
    });
  }
});
