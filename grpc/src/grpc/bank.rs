use celestia_proto::cosmos::bank::v1beta1::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QuerySpendableBalancesRequest, QuerySpendableBalancesResponse, QueryTotalSupplyRequest,
    QueryTotalSupplyResponse,
};
use celestia_types::state::{Address, Coin};

use crate::grpc::{FromGrpcResponse, IntoGrpcParam};
use crate::{Error, Result};

impl<I> IntoGrpcParam<QueryBalanceRequest> for (&Address, I)
where
    I: Into<String>,
{
    fn into_parameter(self) -> QueryBalanceRequest {
        QueryBalanceRequest {
            address: self.0.to_string(),
            denom: self.1.into(),
        }
    }
}

impl FromGrpcResponse<Coin> for QueryBalanceResponse {
    fn try_from_response(self) -> Result<Coin> {
        Ok(self
            .balance
            .ok_or(Error::FailedToParseResponse)?
            .try_into()?)
    }
}

impl IntoGrpcParam<QueryAllBalancesRequest> for &Address {
    fn into_parameter(self) -> QueryAllBalancesRequest {
        QueryAllBalancesRequest {
            address: self.to_string(),
            pagination: None,
        }
    }
}

impl FromGrpcResponse<Vec<Coin>> for QueryAllBalancesResponse {
    fn try_from_response(self) -> Result<Vec<Coin>> {
        Ok(self
            .balances
            .into_iter()
            .map(|coin| coin.try_into())
            .collect::<Result<_, _>>()?)
    }
}

impl IntoGrpcParam<QuerySpendableBalancesRequest> for &Address {
    fn into_parameter(self) -> QuerySpendableBalancesRequest {
        QuerySpendableBalancesRequest {
            address: self.to_string(),
            pagination: None,
        }
    }
}

impl FromGrpcResponse<Vec<Coin>> for QuerySpendableBalancesResponse {
    fn try_from_response(self) -> Result<Vec<Coin>> {
        Ok(self
            .balances
            .into_iter()
            .map(|coin| coin.try_into())
            .collect::<Result<_, _>>()?)
    }
}

impl IntoGrpcParam<QueryTotalSupplyRequest> for () {
    fn into_parameter(self) -> QueryTotalSupplyRequest {
        QueryTotalSupplyRequest { pagination: None }
    }
}

impl FromGrpcResponse<Vec<Coin>> for QueryTotalSupplyResponse {
    fn try_from_response(self) -> Result<Vec<Coin>> {
        Ok(self
            .supply
            .into_iter()
            .map(|coin| coin.try_into())
            .collect::<Result<_, _>>()?)
    }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
pub use wbg::*;

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
mod wbg {
    use celestia_types::state::Coin;
    use js_sys::BigInt;
    use wasm_bindgen::{prelude::*, JsCast};

    use crate::utils::make_object;

    #[wasm_bindgen(typescript_custom_section)]
    const _: &str = "
    /**
     * Coin
     */
    export interface Coin {
      denom: string,
      amount: bigint
    }
    ";

    #[wasm_bindgen]
    extern "C" {
        /// Coin exposed to javascript
        #[wasm_bindgen(typescript_type = "Coin")]
        pub type JsCoin;
    }

    impl From<Coin> for JsCoin {
        fn from(value: Coin) -> JsCoin {
            let obj = make_object!(
                "denom" => value.denom.into(),
                "amount" => BigInt::from(value.amount)
            );

            obj.unchecked_into()
        }
    }
}
