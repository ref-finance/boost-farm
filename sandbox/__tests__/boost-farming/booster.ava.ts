import { NEAR, Gas, NearAccount } from 'near-workspaces-ava';
import { initWorkSpace, registerFungibleTokenUser, callWithMetrics, assertFailure, untill } from './helper';

const workspace = initWorkSpace();

workspace.test('booster_seed_first', async (test, { farming, ft, mft, owner, alice, bob }) => {
    // reward token
    await owner.call(ft, 'mint', {account_id: alice, amount: NEAR.parse('100000')});
    registerFungibleTokenUser(ft, farming);
    // token:0 is booster token, token:1 is affected seed
    await owner.call(mft, 'mint', {inner_id: '0', account_id: alice, amount: NEAR.parse('20000')});
    await owner.call(mft, 'mint', {inner_id: '1', account_id: alice, amount: NEAR.parse('10000')});
    await owner.call(mft, 'mint', {inner_id: '1', account_id: bob, amount: NEAR.parse('10000')});
    await owner.call(mft, 'mft_register', {token_id: ':0', account_id: farming}, { attachedDeposit: NEAR.parse("1250000 nN") });
    await owner.call(mft, 'mft_register', {token_id: ':1', account_id: farming}, { attachedDeposit: NEAR.parse("1250000 nN") });
    // register to farm
    await alice.call(farming, 'storage_deposit', {account_id: alice}, {attachedDeposit: NEAR.parse("100 mN")});
    await bob.call(farming, 'storage_deposit', {account_id: bob}, {attachedDeposit: NEAR.parse("100 mN")});
    // create seed
    await owner.call(farming, 'create_seed', {seed_id: mft.accountId + "@0", seed_decimal: 24}, {attachedDeposit: NEAR.from("1")});
    await owner.call(farming, 'create_seed', {seed_id: mft.accountId + "@1", seed_decimal: 24}, {attachedDeposit: NEAR.from("1")});
    // alice and bob stake free seed
    await alice.call(
        mft, 
        'mft_transfer_call', 
        {token_id: ':0', receiver_id: farming, amount: NEAR.parse('100'), msg: '\"Free\"'}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas')}
    );
    await alice.call(
        mft, 
        'mft_transfer_call', 
        {token_id: ':1', receiver_id: farming, amount: NEAR.parse('1'), msg: '\"Free\"'}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas')}
    );
    await bob.call(
        mft, 
        'mft_transfer_call', 
        {token_id: ':1', receiver_id: farming, amount: NEAR.parse('1'), msg: '\"Free\"'}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas')}
    );
    let farmer_seed = await farming.view('get_farmer_seed', {farmer_id: alice, seed_id: mft.accountId + "@1"});
    test.is((farmer_seed as any).free_amount, NEAR.parse('1').toString());
    test.is(Object.keys((farmer_seed as any).boost_ratios).length, 0);

    let as = {};
    as[mft.accountId+ "@1"] = 100;
    await owner.call(
        farming, 
        'modify_booster', 
        {
            booster_id: mft.accountId + "@0", booster_info: {
                booster_decimal: 24, 
                affected_seeds: as, 
            }
        }, 
        {attachedDeposit: NEAR.from("1")}
    );
    farmer_seed = await farming.view('get_farmer_seed', {farmer_id: alice, seed_id: mft.accountId + "@1"});
    test.is((farmer_seed as any).free_amount, NEAR.parse('1').toString());
    test.is(Object.keys((farmer_seed as any).boost_ratios).length, 0);

    // after claim_reward of affected seed, should update boost_ratios
    await alice.call(farming, 'claim_reward_by_seed', {seed_id: mft.accountId + "@1"}, {gas: Gas.parse('100 Tgas')});
    farmer_seed = await farming.view('get_farmer_seed', {farmer_id: alice, seed_id: mft.accountId + "@1"});
    test.is((farmer_seed as any).boost_ratios[mft.accountId + "@0"], 1);

    // after adding booster, should update boost_ratios of affected seed
    await alice.call(
        mft, 
        'mft_transfer_call', 
        {token_id: ':0', receiver_id: farming, amount: NEAR.parse('9900'), msg: '\"Free\"'}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas')}
    );
    farmer_seed = await farming.view('get_farmer_seed', {farmer_id: alice, seed_id: mft.accountId + "@1"});
    test.is((farmer_seed as any).boost_ratios[mft.accountId + "@0"], 2);
    
    // test.log((farmer_seed as any).boost_ratios);
    // test.log((await farming.view('get_config') as any).booster_seeds);
});