import { NEAR } from 'near-workspaces-ava';
import { initWorkSpace, assertFailure } from './helper';

const workspace = initWorkSpace();

workspace.test('set_owner', async (test, { farming, ft, mft, owner, alice, bob }) => {
  let md = await farming.view('get_metadata');
  test.log(md);

  test.deepEqual(md, {
    version: '0.1.3',
    owner_id: 'ref_owner.test.near',
    operators: [],
    farmer_count: '0',
    farm_count: '0',
    outdated_farm_count: '0',
    seed_count: '0',
  });

  await owner.call(farming, 'set_owner', { owner_id: alice }, { attachedDeposit: NEAR.from("1") });

  test.is(
    (await farming.view('get_metadata') as any).owner_id,
    alice.accountId,
  );

  await alice.call(farming, 'set_owner', { owner_id: bob }, { attachedDeposit: NEAR.from("1") });

  test.is(
    (await farming.view('get_metadata') as any).owner_id,
    bob.accountId,
  );

  await bob.call(farming, 'set_owner', { owner_id: owner }, { attachedDeposit: NEAR.from("1") });

  test.is(
    (await farming.view('get_metadata') as any).owner_id,
    owner.accountId,
  );
});

workspace.test('manage_operators', async (test, { farming, ft, mft, owner, alice, bob }) => {

  await owner.call(farming, 'extend_operators', { operators: [alice, bob] }, { attachedDeposit: NEAR.from("1") });
  test.deepEqual(
    (await farming.view('get_metadata') as any).operators,
    [alice.accountId, bob.accountId],
  );

  await owner.call(farming, 'remove_operators', { operators: [bob] }, { attachedDeposit: NEAR.from("1") });
  test.deepEqual(
    (await farming.view('get_metadata') as any).operators,
    [alice.accountId],
  );
});
