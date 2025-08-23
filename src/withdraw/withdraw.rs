use ic_cdk::{api::msg_caller, update};

use crate::{
    house_settings::get_house_asset_ledger,
    user::balance_utils::{get_user_balance, set_user_balance},
    withdraw::withdraw_params::WithdrawParams,
};

#[update]
pub async fn withdraw(params: WithdrawParams) {
    let user = msg_caller();

    let house_asset_ledger = get_house_asset_ledger();

    let user_balance = get_user_balance(user);

    if user_balance > params.amount {
        set_user_balance(user, user_balance - params.amount);
        let tx_result = house_asset_ledger
            ._send_out(params.amount, user, None)
            .await;
        if tx_result == false {
            //refund
            set_user_balance(user, user_balance + params.amount);
        }
    }
}
