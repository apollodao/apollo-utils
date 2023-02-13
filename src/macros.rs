#[macro_export]
macro_rules! contract_helper {
    ($contract_helper:ident, $contract_helper_unchecked:ident, $execute_msg:ident) => {
        #[cosmwasm_schema::cw_serde]
        pub struct $contract_helper(cosmwasm_std::Addr);

        #[cosmwasm_schema::cw_serde]
        pub struct $contract_helper_unchecked(String);

        impl $contract_helper_unchecked {
            pub fn new(addr: String) -> Self {
                Self(addr)
            }

            pub fn check(
                &self,
                api: &dyn cosmwasm_std::Api,
            ) -> cosmwasm_std::StdResult<$contract_helper> {
                let addr = api.addr_validate(&self.0)?;
                Ok($contract_helper(addr))
            }
        }

        impl $contract_helper {
            pub fn addr(&self) -> cosmwasm_std::Addr {
                self.0.clone()
            }

            pub fn call(
                &self,
                msg: impl Into<$execute_msg>,
                funds: Vec<cosmwasm_std::Coin>,
            ) -> cosmwasm_std::StdResult<cosmwasm_std::CosmosMsg> {
                let msg = cosmwasm_std::to_binary(&msg.into())?;
                Ok(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: self.addr().into(),
                    msg,
                    funds,
                }
                .into())
            }
        }

        impl From<$contract_helper> for $contract_helper_unchecked {
            fn from(h: $contract_helper) -> Self {
                Self(h.0.to_string())
            }
        }
    };
}

#[cfg(test)]
pub mod tests {
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::Empty;

    contract_helper!(TestHelper, TestHelperUnchecked, Empty);

    const HELPER_ADDR: &str = "test_helper";

    #[test]
    fn test_new_and_check_for_unchecked() {
        let api = MockApi::default();
        let helper = TestHelperUnchecked::new(HELPER_ADDR.to_string());

        let checked_helper = helper.check(&api).unwrap();

        assert_eq!(helper.0, checked_helper.0.to_string())
    }

    #[test]
    fn test_from_checked_for_unchecked() {
        let api = MockApi::default();
        let helper = TestHelperUnchecked::new(HELPER_ADDR.to_string());

        let checked_helper = helper.check(&api).unwrap();

        assert_eq!(helper, TestHelperUnchecked::from(checked_helper))
    }

    #[test]
    fn test_addr_for_checked() {
        let api = MockApi::default();
        let helper = TestHelperUnchecked::new(HELPER_ADDR.to_string())
            .check(&api)
            .unwrap();

        assert_eq!(helper.addr().to_string(), HELPER_ADDR)
    }
}
