#[cfg(test)]
mod test {
    use soroban_sdk::{Env};

    use crate::FeeContract;

    #[test]
    fn test_get_fee_config_default() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FeeContract);
        let client = FeeContractClient::new(&env, &contract_id);

        let (base_fee, max_fee) = client.get_fee_config();

        assert_eq!(base_fee, 100);
        assert_eq!(max_fee, 1_000_000);
    }
}