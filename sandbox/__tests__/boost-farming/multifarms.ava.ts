import { NEAR, Gas, NearAccount } from 'near-workspaces-ava';
import { initWorkSpace, registerFungibleTokenUser, callWithMetrics, assertFailure } from './helper';

const workspace = initWorkSpace();

async function createThenStake (owner: NearAccount, farming: NearAccount, ft: NearAccount, mft: NearAccount, alice: NearAccount, id: number) {
    const terms = {
        reward_token: ft.accountId,
        start_at: 0,
        daily_reward: NEAR.parse('86400')
    };
    await owner.call(farming, 'create_farm', { seed_id: mft.accountId + "@0", terms }, { attachedDeposit: NEAR.from("1") });
    let msg = '{"Reward":{"farm_id":"' + mft.accountId + '@0#' + id.toString() + '"}}';
    await alice.call(
        ft, 
        'ft_transfer_call', 
        {receiver_id: farming, amount: NEAR.parse('3600'), msg}, 
        {attachedDeposit: NEAR.from("1"), gas: Gas.parse('100 Tgas') }
    );

    const rslt = await callWithMetrics(
        alice,
        mft, 
        'mft_transfer_call', 
        {token_id: ':0', receiver_id: farming, amount: NEAR.parse('1'), msg: '\"Free\"'}, 
        { attachedDeposit: NEAR.from("1"), gas: Gas.parse('200 Tgas') }
    );
    return rslt;
  }

workspace.test('multifarming', async (test, { farming, ft, mft, owner, alice, bob }) => {
    
    await owner.call(ft, 'mint', { account_id: alice, amount: NEAR.parse('100000') }, { attachedDeposit: NEAR.from("0"), gas: Gas.parse('30 Tgas') });
    await owner.call(mft, 'mint', { inner_id: '0', account_id: alice, amount: NEAR.parse('10000') }, { attachedDeposit: NEAR.from("0"), gas: Gas.parse('30 Tgas') });

    registerFungibleTokenUser(ft, farming);
    await alice.call(farming, 'storage_deposit', { account_id: alice }, { attachedDeposit: NEAR.parse("100 mN") });
    await alice.call(mft, 'mft_register', {token_id: ':0', account_id: farming}, { attachedDeposit: NEAR.parse("1250000 nN") });

    await owner.call(farming, 'create_seed', { seed_id: mft.accountId + "@0", seed_decimal: 24 }, { attachedDeposit: NEAR.from("1") });
    let rslt = await callWithMetrics(
        alice,
        mft, 
        'mft_transfer_call', 
        {token_id: ':0', receiver_id: farming, amount: NEAR.parse('1'), msg: '\"Free\"'}, 
        { attachedDeposit: NEAR.from("1"), gas: Gas.parse('200 Tgas') }
    );
    test.log('successValue:', rslt.successValue);
    test.log('Gas:', rslt.metrics.gasBurnt.toHuman());

    let tx;
    for (let i=0;i<4;i++) {
        tx = await createThenStake(owner, farming, ft, mft, alice, i);
        test.log('successValue:', tx.successValue);
        test.log('Gas:', tx.metrics.gasBurnt.toHuman());
    }
    
    // let farms = await farming.view('list_seed_farms', {seed_id: mft.accountId + "@0"});
    // test.log(farms);
    let farmer_seed = await farming.view('list_farmer_seeds', {farmer_id: alice});
    test.log(farmer_seed);

    let farmer_reward = await farming.view('list_farmer_rewards', {farmer_id: alice});
    test.log(farmer_reward);

});