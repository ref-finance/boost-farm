# API for REF Boost-Farming
## What is Boost-Farming
This is the next generation of `ref-farming` contract.  
Compare to `ref-farming`, it has following feature upgrade:
* Support more seed power by locking seed for a period,
* Farm reward speed could be adjusted runtime,
* Ended farm could be restarted by deposit more reward,
* Simplified storage fee strategy to improve user experience,
* Support global booster tokens,

## Logic
### User Roles
There are three user roles:
* Farmer
    * Can stake/unstake seed to participate in farming,
    * Can claim farming reward to inner account,
    * Can withdraw assets from inner account to user wallet,
* Operator (also could be a farmer)
    * create new seeds and farms,
    * adjust locking policy,
    * adjust full slash per seed rate and default rate,
    * adjust minimum deposit per seed,
    * withdraw slashed seed to owner account,
* Owner (mostly is a DAO)
    * set owner to another account,
    * manage operators,
    * refund from seed lostfound,
    * upgrade the contract,
### Seed and Farm Logic
Farmer stakes `seed` to get multiple rewards from all `farm` reside in this seed.  
For `seed`:
* managed by operators,
* could be empty (without any farm in it),  
```rust
pub struct Seed {
    ...
    #[serde(with = "u128_dec_format")]
    pub total_seed_amount: Balance,
    #[serde(with = "u128_dec_format")]
    pub total_seed_power: Balance,
    ...
    /// full pre-unlock slash_rate,
    /// set to 0 to enable free pre-unlock
    pub slash_rate: u32,
    /// if min_lock_duration == 0, means forbid locking
    pub min_locking_duration_sec: DurationSec,
    /// the active farmers count 
    pub farmer_count: u32,
}
```
**Compute APY**:  
$$
\frac{\sum (daily\_reward * 360 * reward\_price)}{seed\_amount * seed\_price} * 100\%
$$

For `farm`:
```rust
pub struct FarmTerms {
    pub reward_token: AccountId,
    /// TimeStamp in sec
    /// set to 0 means to start as soon as reward deposited
    pub start_at: u32,
    #[serde(with = "u128_dec_format")]
    pub daily_reward: Balance,
}

pub enum FarmStatus {
    /// Just create and waiting for the start time
    Created,
    /// Past the start time but no reward deposited
    Pending,
    /// Past the start time and have reward to distribute
    Running,
    /// Past the start time and reward has been dry out
    Ended,
}

pub struct SeedFarm {
    ...
    pub terms: FarmTerms,
    /// total reward send into this farm by far,
    /// every time reward deposited in, add to this field
    #[serde(with = "u128_dec_format")]
    pub total_reward: Balance,
    #[serde(with = "u64_dec_format")]
    pub distributed_at: Timestamp,
    #[serde(with = "u128_dec_format")]
    pub distributed_reward: Balance,
    #[serde(with = "u128_dec_format")]
    pub claimed_reward: Balance,
    /// when there is no seed token staked, reward goes to beneficiary
    #[serde(with = "u128_dec_format")]
    pub amount_of_beneficiary: Balance,
    ...
}
```
**Amount Equations**:
$$
total\_reward = remaining\_reward + distributed\_reward
$$
$$
distributed\_reward = claimed\_reward + amount\_of\_beneficiary + unclaimed\_reward
$$
**Compute Expected End Date**:  
$$
end\_date = to\_sec(distributed\_at) + \frac{remaining\_reward * 3600 * 24}{daily\_reward} 
$$
### Locking Policy
Every farmer can lock their seed for a period to gain more power in reward distribution in this seed.
```rust
pub struct FarmerSeed {
    #[serde(with = "u128_dec_format")]
    pub free_amount: Balance,
    /// The amount of locked token.
    #[serde(with = "u128_dec_format")]
    pub locked_amount: Balance,
    /// The amount of power for those locked amount.
    #[serde(with = "u128_dec_format")]
    pub x_locked_amount: Balance,
    /// When the locking token can be unlocked without slash in nanoseconds.
    #[serde(with = "u64_dec_format")]
    pub unlock_timestamp: u64,
    /// The duration of current locking in seconds.
    pub duration_sec: u32,
    ...
}
```
$$
X = A + A \frac{D(M_{max} - M_{min})}{D_{max} M_{min}}
$$
$$
P = F + X
$$
Where,  
$P$ is total basic seed power,  
$F$ is free seed amount,
$X$ is locking seed power,  
$A$ is locking seed amount,  
$D$ is farmer requested locking period,  
$D_{max}$ is the maximum locking duration, such as 360 days,  
$M_{min}$ is base BP ratio litterally is 10000,  
$M_{max}$ is multiple BP ratio related to $D_{max}$, say 20000,  

**Example:**  
Alice staked 100 seed in free and locking another 30 seed for 60 days,  
$$
X = 30 + 30* \frac{60*(2.0 - 1.0)}{360*1.0} = 35
$$
$$
P = 100 + 35 = 135
$$

**Append to Current Locking**  
Require:
- The new unlock timestamp must be later than current one;

Algorithm:  
1. Using the new duration to re-lock current locking balance, if new $X$ is larger, then update it, $X = Max(X, X_{new})$
2. Lock the append amount, got the extra $X_{append}$
3. $X = X + X_{append}$

### Slash Policy
Slash policy is calculate per seed by `slash_rate` on `seed` level.
$$
slash\_amount = pre\_unlock\_amount * slash\_rate * \frac{unlock\_ts - current\_ts}{unlock\_ts-lock\_ts}
$$
$$
remain\_X = X * \frac{locking\_amount-pre\_unlock\_amount}{locking\_amount}
$$
### Global Booster Tokens Logic
A booster is a specific seed. Farmer would benefit from booster token with more multiple seed farming rewards.  
```rust
pub struct BoosterInfo {
    pub booster_decimal: u32,
    /// <affected_seed_id, log_base>
    pub affected_seeds: HashMap<SeedId, u32>,
}

pub struct Config {
    ...
    /// Key is booster_id, support multiple booster
    pub booster_seeds: HashMap<SeedId, BoosterInfo>,
    ...
}

pub struct FarmerSeed {
    ...
    /// <booster_id, booster-ratio>
    pub boost_ratios: HashMap<SeedId, f64>,
    ...
}
```

For each affected seed, we have:  
$$
booster\_ratio = log_{base}(\frac{booster\_power}{booster\_unit})
$$
$$
extra\_seed\_power = original\_seed\_power * booster\_ratio
$$
Where:  
$base$ determine the relative power of booster to the affected seed;  

Example:  
Say booster is a seed which decimal is 18.  
If $base = 10$,  
Then $100*10^{18}$ amount of booster_locking_power would got double extra seed power, make the farmer's total seed power goes to 3X.  

## Interface
### User Register
This contract obeys NEP-145 to manage storage, but choose a fixed storage fee policy in this contract. Each user only needs deposit to lock a fixed 0.1 NEAR as storage cost.

Detailed interface description could be found at [NEP-145](https://nomicon.io/Standards/StorageManagement.html).

Here we only list some common-use interfaces:

* `storage_deposit`, to register a user,
* `storage_unregister`, to unregister caller self and get 0.1 NEAR back,
* `storage_balance_of`, to get given user storage balance,
* `storage_balance_bounds`, to get storage policy.

Note: 
- To sucessfully unregister, farmer should withdraw all his seed and reward tokens before calling `storage_unregister`.
- Support having a sponsor to deposit storage for farmer, in that case, when `storage_unregister`, the fixed 0.1 near would transfer back to that sponsor. Can use `get_farmer_sponsor(farmer_id)` to check it.

### Farmer Stake/Unstake/Lock/Unlock
**Stake and Lock**  
are executed by calling seed token's `(m)ft_transfer_call ` with the following msg:
```rust
enum TokenReceiverMessage {
    Free,
    Lock { duration_sec: u32 },
    ...
}
```
Eg:
```bash
near call $MFT mft_transfer_call '{"receiver_id": "'$FARM'", "token_id": ":0", "amount": "1'$ZERO24'", "msg": "\"Free\""}' --account_id=u1.testnet --depositYocto=1 --gas=250$TGAS

near call $MFT mft_transfer_call '{"receiver_id": "'$FARM'", "token_id": ":0", "amount": "1'$ZERO24'", "msg": "{\"Lock\":{\"duration_sec\":5184000}}"}' --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```

**Unlock and Unstake**  
are unified into one interface `unlock_and_unstake_seed`:
```rust
pub fn unlock_and_unstake_seed(
    &mut self,
    seed_id: SeedId,
    unlock_amount: U128,
    unstake_amount: U128,
) 
```
First, check if `unlock_amount` > 0, do unlock and unlocked seed amount goes to Free,  
Then check if `unstake_amount` > 0, do unstake seed from Free.  

Eg:
```bash
near call $FARM unlock_and_unstake_seed '{"seed_id": "'$MFT'@0", "unlock_amount": "0", "unstake_amount": "1'$ZERO24'"}' --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```

**Withdraw seed**  
Withdraw those unstaked seed:
```rust
pub fn withdraw_seed(
    &mut self,
    seed_id: SeedId,
    amount: Option<U128>,
) 
```

Eg:
```bash
near call $FARM withdraw_seed '{"seed_id": "'$MFT'@0"}' --account_id=u1.testnet --gas=150$TGAS
```

**Free to Lock**  
Let farmer convert his Free seed to Locking without transfer from outside:
```rust
pub fn lock_free_seed(&mut self, seed_id: SeedId, duration_sec: u32, amount: Option<U128>)
```
Note: 
- if `amount` is not given, would take all free seed into locking.  
- if `amount` is zero, it is sort of extend locking period usage.

Eg:
```bash
near call $FARM lock_free_seed '{"seed_id": "'$MFT'@0", "duration_sec": 5184000}' --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```
**Pre-Unlock**  
Let farmer can unlock his undue locking seed with a little slashed seed pro rata to undue time-span. 

```rust
pub fn force_unlock(&mut self, seed_id: SeedId, unlock_amount: U128)
```
Eg:
```bash
near call $FARM force_unlock '{"seed_id": "'$MFT'@0", "unlock_amount": "1'$ZERO24'"}' --account_id=u1.testnet --depositYocto=1 --gas=150$TGAS
```
### Farmer Claim/Withdraw Reward
```rust
pub fn claim_reward_by_seed(&mut self, seed_id: SeedId)
```
Note: No deposit request, could be sign with access-key.  

Eg:
```bash
near call $FARM claim_reward_by_seed '{"seed_id": "'$MFT'@0"}' --account_id=u1.testnet --gas=150$TGAS
```
```rust
pub fn withdraw_reward(&mut self, token_id: AccountId, amount: Option<U128>)
```
Note: 
1. No deposit request, could be sign with access-key.  
2. If amount is not given, withdraw all balance.

Eg:
```bash
near call $FARM withdraw_reward '{"token_id": "ref.'$FT'"}' --account_id=u1.testnet --gas=150$TGAS
```
### Operator Manage Seed/Farm
```rust
pub fn create_seed(&mut self, seed_id: SeedId, seed_decimal: u32, min_deposit: Option<U128>);
pub fn create_farm(&mut self, seed_id: SeedId, terms: FarmTerms) -> FarmId;
pub fn modify_daily_reward(&mut self, farm_id: FarmId, daily_reward: U128);
pub fn cancel_farm(&mut self, farm_id: String);
pub fn remove_farm_from_seed(&mut self, farm_id: String);
```
Note:
1. `modify_daily_reward` can only be called by operator or owner,    
2. What does a cancel mean?  
    A farm without any deposited reward could be cancel;
3. What does a removal mean?  
    An ended farm could be finalized and remove out of seed;

Eg:
```bash
# create seed, shutdown locking
near call $FARM create_seed '{"seed_id": "'$MFT'@1", "seed_decimal": 24, "min_locking_duration_sec": 0}' --accountId $ROOT --depositYocto=1 
# create farm, start when deposit reward
near call $FARM create_farm '{"seed_id": "'$MFT'@0", "terms": {"reward_token": "usdt.'$FT'", "start_at": 0, "daily_reward": "8640'$ZERO6'"}}' --account_id=$ROOT --depositYocto=1 
```

### Deposit Reward to Farm
are executed by calling reward token's `ft_transfer_call ` with the following msg:
```rust
enum TokenReceiverMessage {
    ...
    Reward { farm_id: FarmId },
}
```
Eg:
```bash
near call ref.$FT ft_transfer_call '{"receiver_id": "'$FARM'", "amount": "36'$ZERO18'", "msg": "{\"Reward\":{\"farm_id\":\"'$MFT'@0#0\"}}"}' --account_id=u1.testnet --depositYocto=1 --gas=100$TGAS || true
```

### Seed Slashed/Lostfound Related
```rust
pub fn modify_locking_policy(&mut self, max_duration: DurationSec, max_ratio: u32);
pub fn modify_seed_min_locking_duration(&mut self, seed_id: String, min_locking_duration_sec: DurationSec);

pub fn modify_default_slash_rate(&mut self, slash_rate: u32);
pub fn modify_seed_slash_rate(&mut self, seed_id: String, slash_rate: u32);

pub fn withdraw_seed_slashed(&mut self, seed_id: SeedId) -> Promise;
pub fn return_seed_lostfound(&mut self, farmer_id: AccountId, seed_id: SeedId, amount: U128) -> Promise;
```

### Booster Related
```rust
pub struct BoosterInfo {
    pub booster_decimal: u32,
    /// <affected_seed, log_base>
    pub affected_seeds: HashMap<SeedId, u32>,
}
pub fn modify_booster(&mut self, booster_id: SeedId, booster_info: BoosterInfo);
```


### All Views
**Contract Info**
```bash
near view $FARM get_metadata
{
  version: '0.0.1',
  owner_id: 'ref-dev.testnet',
  operators: [],
  farmer_count: '1',
  farm_count: '16',
  outdated_farm_count: '0',
  seed_count: '3'
}

near view $FARM get_config
{
  seed_slash_rate: 200,
  booster_seeds: {
    'mft.ref-dev.testnet@1': {
      booster_decimal: 24,
      affected_seeds: { 'mft.ref-dev.testnet@0': 10 }
    }
  },
  max_num_farms_per_booster: 64,
  max_num_farms_per_seed: 16,
  maximum_locking_duration_sec: 31104000,
  max_locking_multiplier: 20000
}

near view $FARM get_contract_storage_report
{ storage: '559993', locking_near: '5599930000000000000000000' }

near view $FARM list_outdated_farms
near view $FARM get_outdated_farm '{"farm_id": "xxx"}'
near view $FARM list_slashed
near view $FARM list_lostfound
```
**Storage**
```bash
near view $FARM storage_balance_bounds
{ min: '100000000000000000000000', max: '100000000000000000000000' }

near view $FARM storage_balance_of '{"account_id": "xxx"}'
{ total: '100000000000000000000000', available: '0' }
```
**Seed Related**
```bash
near view $FARM list_seeds_info
[
  {
    seed_id: 'mft.ref-dev.testnet@1',
    seed_decimal: 24,
    next_index: 0,
    total_seed_amount: '0',
    total_seed_power: '0',
    min_deposit: '1000000000000000000',
    slash_rate: 200,
    min_locking_duration_sec: 0
  },
  {
    seed_id: 'mft.ref-dev.testnet@0',
    seed_decimal: 24,
    next_index: 0,
    total_seed_amount: '0',
    total_seed_power: '0',
    min_deposit: '1000000000000000000',
    slash_rate: 200,
    min_locking_duration_sec: 2592000
  }
]

near view $FARM get_seed '{"seed_id": "xxx"}'

near view $FARM list_seed_farms '{"seed_id": "'$MFT'@0"}'
[
  {
  farm_id: 'mft.ref-dev.testnet@0#2',
  terms: {
    reward_token: 'wbtc.fakes.testnet',
    start_at: 1650157060,
    daily_reward: '864000000000'
  },
  total_reward: '6048000000000',
  distributed_at: '1650161993305070047',
  distributed_reward: '49330936399',
  claimed_reward: '11344112909',
  amount_of_beneficiary: '0',
  status: 'Running'
},
  ...
]
near view $FARM get_farm '{"farm_id": "xxx"}'
```

**Farmer Concerned**
```bash
near view $FARM get_unclaimed_rewards '{"farmer_id": "u1.testnet", "seed_id": "'$MFT'@0"}'
{ 'ref.fakes.testnet': '508586041258490000000' }

near view $FARM list_farmer_seeds '{"farmer_id": "u1.testnet"}'
{
  'mft.ref-dev.testnet@1': {
    free_amount: '100000000000000000000000000',
    locked_amount: '0',
    x_locked_amount: '0',
    unlock_timestamp: '0',
    duration_sec: 0,
    boost_ratios: {}
  },
  'mft.ref-dev.testnet@0': {
    free_amount: '1000000000000000000000000',
    locked_amount: '0',
    x_locked_amount: '0',
    unlock_timestamp: '0',
    duration_sec: 0,
    boost_ratios: { 'mft.ref-dev.testnet@1': 2 }
  }
}

near view $FARM get_farmer_seed '{"farmer_id": "u1.testnet", "seed_id": "'$MFT'@0"}'

near view $FARM list_farmer_rewards '{"farmer_id": "u1.testnet"}'
{ 'ref.fakes.testnet': '67413958741510000000' }

near view $FARM get_farmer_reward '{"farmer_id": "u1.testnet", "token_id": "usdt.'$FT'"}'
'0'

near view $FARM get_farmer_sponsor '{"farmer_id": "u1.testnet"}'
'u1.testnet'
```
