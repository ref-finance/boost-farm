import { NEAR, Gas, NearAccount } from 'near-workspaces-ava';
import { initWorkSpace, registerFungibleTokenUser, callWithMetrics, assertFailure, untill } from './helper';

const workspace = initWorkSpace();

workspace.test('farm_lifecycle', async (test, { farming, ft, mft, owner, alice, bob }) => {
    await owner.call(ft, 'mint', { account_id: bob, amount: NEAR.parse('100000') }, { attachedDeposit: NEAR.from("0"), gas: Gas.parse('30 Tgas') });
    registerFungibleTokenUser(ft, farming);

    await owner.call(farming, 'extend_operators', { operators: [alice] }, { attachedDeposit: NEAR.from("1") });

    test.is((await farming.view('list_seeds_info') as any).length, 0);
    await alice.call(farming, 'create_seed', { seed_id: mft.accountId + "@0", seed_decimal: 24 }, { attachedDeposit: NEAR.from("1") });
    test.is((await farming.view('list_seeds_info') as any).length, 1);
    test.deepEqual((await farming.view('get_seed', {seed_id: mft.accountId + "@0"}) as any), {
        seed_id: mft.accountId + "@0",
        seed_decimal: 24,
        next_index: 0,
        total_seed_amount: NEAR.parse('0').toString(),
        total_seed_power: NEAR.parse('0').toString(),
        min_deposit: '1000000000000000000',
        min_locking_duration_sec: 2592000,
        slash_rate: 200,
    });
    test.is((await farming.view('list_seed_farms', {seed_id: mft.accountId + "@0"}) as any).length, 0);

    // create a farm start at 10 secs later
    let ts = new Date().getTime() + 10000;
    const terms = {
        reward_token: ft.accountId,
        start_at: Math.floor(ts/1000),
        daily_reward: NEAR.parse('86400')
    };
    await alice.call(farming, 'create_farm', { seed_id: mft.accountId + "@0", terms }, { attachedDeposit: NEAR.from("1") });
    test.is((await farming.view('list_seed_farms', {seed_id: mft.accountId + "@0"}) as any).length, 1);
    test.is((await farming.view('get_seed', {seed_id: mft.accountId + "@0"}) as any).next_index, 1);
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).status, 'Created');
    test.deepEqual((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).terms, {
        reward_token: ft.accountId,
        start_at: Math.floor(ts/1000),
        daily_reward: NEAR.parse('86400').toString(),
    });
    test.log('[' + Date.now() + ']', 'Operator created a farm start at', ts);

    await untill(ts+1000);
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).status, 'Pending');

    let msg = '{"Reward":{"farm_id":"' + mft.accountId + '@0#0"}}';
    await bob.call(
        ft, 
        'ft_transfer_call', 
        {receiver_id: farming, amount: NEAR.parse('8'), msg}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas') }
    );
    test.log('[' + Date.now() + ']', 'bob deposited 8 secs reward');
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).status, 'Running');

    await untill(ts+9000);
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).status, 'Ended');

    await bob.call(
        ft, 
        'ft_transfer_call', 
        {receiver_id: farming, amount: NEAR.parse('5'), msg}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas') }
    );
    test.log('[' + Date.now() + ']', 'bob re-deposited 5 secs reward');
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).status, 'Running');
    let end_ts2 = Math.floor((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).distributed_at / 1000000) + 5000;

    await untill(end_ts2);
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).status, 'Ended');

    await owner.call(farming, 'remove_farm_from_seed', {farm_id: mft.accountId + "@0#0"}, { attachedDeposit: NEAR.from("1") });
    test.log('[' + Date.now() + ']', 'Owner removed farm');
    test.is((await farming.view('list_seed_farms', {seed_id: mft.accountId + "@0"}) as any).length, 0);
    test.is((await farming.view('list_outdated_farms') as any).length, 1);
    let outdated_farm = await farming.view('get_outdated_farm', {farm_id: mft.accountId + "@0#0"}) as any;
    test.is(outdated_farm.amount_of_beneficiary, NEAR.parse('13').toString());
    test.is(outdated_farm.claimed_reward, NEAR.parse('0').toString());
    test.is(outdated_farm.distributed_reward, NEAR.parse('13').toString());
    test.is(outdated_farm.total_reward, NEAR.parse('13').toString());
});

workspace.test('farm_cancel', async (test, { farming, ft, mft, owner, alice, bob }) => {
    await owner.call(farming, 'extend_operators', { operators: [alice] }, { attachedDeposit: NEAR.from("1") });

    await alice.call(farming, 'create_seed', { seed_id: mft.accountId + "@0", seed_decimal: 24 }, { attachedDeposit: NEAR.from("1") });

    const terms = {
        reward_token: ft.accountId,
        start_at: 0,
        daily_reward: NEAR.parse('86400')
    };
    await alice.call(farming, 'create_farm', { seed_id: mft.accountId + "@0", terms }, { attachedDeposit: NEAR.from("1") });
    test.is((await farming.view('list_seed_farms', {seed_id: mft.accountId + "@0"}) as any).length, 1);

    await assertFailure(
        test,
        bob.call(farming, 'cancel_farm', {farm_id: mft.accountId + "@0#0"}, {attachedDeposit: NEAR.from("1")}),
        'E002'
    );
    
    await alice.call(farming, 'cancel_farm', {farm_id: mft.accountId + "@0#0"}, {attachedDeposit: NEAR.from("1")});
    test.is((await farming.view('list_seed_farms', {seed_id: mft.accountId + "@0"}) as any).length, 0);
});

workspace.test('farm_modify_daily_reward', async (test, { farming, ft, mft, owner, alice, bob }) => {
    await owner.call(farming, 'extend_operators', { operators: [alice] }, { attachedDeposit: NEAR.from("1") });

    await alice.call(farming, 'create_seed', { seed_id: mft.accountId + "@0", seed_decimal: 24 }, { attachedDeposit: NEAR.from("1") });

    const terms = {
        reward_token: ft.accountId,
        start_at: 0,
        daily_reward: NEAR.parse('86400')
    };
    await alice.call(farming, 'create_farm', { seed_id: mft.accountId + "@0", terms }, { attachedDeposit: NEAR.from("1") });
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).terms.daily_reward, NEAR.parse('86400').toString());
 
    await assertFailure(
        test,
        bob.call(farming, 'modify_daily_reward', {farm_id: mft.accountId + "@0#0", daily_reward: NEAR.parse('8640')}, {attachedDeposit: NEAR.from("1")}),
        'E002'
    );
    
    await alice.call(farming, 'modify_daily_reward', {farm_id: mft.accountId + "@0#0", daily_reward: NEAR.parse('8640')}, {attachedDeposit: NEAR.from("1")});
    test.is((await farming.view('get_farm', {farm_id: mft.accountId + "@0#0"}) as any).terms.daily_reward, NEAR.parse('8640').toString());
});