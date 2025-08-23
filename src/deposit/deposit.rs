use ic_cdk::{api::msg_caller, update};

use crate::{
    deposit::deposit_params::DepositParams, house_settings::get_house_asset_ledger,
    user::balance_utils::update_user_balance,
};

#[update]
pub async fn deposit(params: DepositParams) {
    let user = msg_caller();

    let house_asset_ledger = get_house_asset_ledger();

    let tx_result = house_asset_ledger
        ._send_in(params.amount, user, params.block_index, None)
        .await;

    if tx_result {
        update_user_balance(user, params.amount, true);
    }
}
