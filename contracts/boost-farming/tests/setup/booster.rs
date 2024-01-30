use crate::*;

impl Env {
    pub fn modify_booster(
        &self,
        operator: &UserAccount,
        booster_id: &SeedId, 
        booster_info: &BoosterInfo
    ) -> ExecutionResult {
        let booster_info = meme_farming::BoosterInfo{
            booster_decimal: booster_info.booster_decimal,
            affected_seeds: booster_info.affected_seeds.clone(),
        };
        operator
            .function_call(
                self.farming_contract.contract.modify_booster(
                    booster_id.clone(),
                    booster_info
                ),
                MAX_GAS.0,
                1,
            )
    }
}