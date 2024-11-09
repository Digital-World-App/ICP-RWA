use crate::digital_world;
use crate::utils;
use candid::{CandidType, Principal};
use ic_cdk_macros::*;
use ic_ledger_types::{
    AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, MAINNET_LEDGER_CANISTER_ID,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::hash::Hash;

const ICP_FEE: u64 = 10_000;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash)]
pub struct TransferArgs {
    amount: Tokens,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
}

/// Mapeia um ID do mundo digital para se ele foi comprado
pub type PurchaseStore = BTreeMap<i128, Trade>;

thread_local! {
    pub static PURCHASE_STORE: RefCell<PurchaseStore> = RefCell::default();
}

/// Estrutura para armazenar dados de transações
#[derive(Clone, Copy, CandidType, Deserialize)]
pub struct Trade {
    seller: Principal,
    amount: u64,
}

// Função auxiliar para buscar transações específicas por ID
fn get_trade_for_digital_world_id(id: i128) -> Option<Trade> {
    PURCHASE_STORE.with(|event_store| event_store.borrow().get(&id).cloned())
}

/// Função para reivindicar uma venda
#[update]
async fn claim_sale(digital_world_id: i128) -> Result<BlockIndex, String> {
    let caller_principal = utils::caller();
    let canister_id = ic_cdk::api::id();

    // Busca o registro de venda
    let trade = get_trade_for_digital_world_id(digital_world_id)
        .ok_or_else(|| "Item não encontrado".to_string())?;

    // Verifica se o solicitante é o vendedor
    if trade.seller != caller_principal {
        return Err("Você não é o vendedor deste item".to_string());
    }

    ic_cdk::println!(
        "Transferindo {} ICP para o principal {}",
        &trade.amount,
        caller_principal,
    );

    let transfer_args = ic_ledger_types::TransferArgs {
        memo: Memo(0),
        amount: Tokens::from_e8s(trade.amount),
        fee: Tokens::from_e8s(ICP_FEE),
        from_subaccount: Some(principal_to_subaccount(&canister_id)),
        to: AccountIdentifier::new(
            &caller_principal,
            &principal_to_subaccount(&caller_principal),
        ),
        created_at_time: None,
    };

    let transfer_result = ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(|e| format!("Erro ao chamar o ledger: {:?}", e))?
        .map_err(|e| format!("Erro na transferência do ledger: {:?}", e));

    PURCHASE_STORE.with(|purchase_store| purchase_store.borrow_mut().remove(&digital_world_id));
    transfer_result
}

#[derive(CandidType)]
pub enum DepositErr {
    BalanceLow,
    TransferFailure,
}

pub type DepositReceipt = Result<(), DepositErr>;

/// Função para comprar um item, executada pelo comprador
#[update]
pub async fn buy_item(digital_world_id: i128, amount: u64) -> DepositReceipt {
    let canister_id = ic_cdk::api::id();
    let caller_principal = utils::caller();

    // Obtém detalhes do mundo digital pelo ID
    let digital_world = digital_world::get_digital_world_by_id(digital_world_id);
    let trade = Trade {
        seller: digital_world.user,
        amount,
    };

    PURCHASE_STORE
        .with(|purchase_store| purchase_store.borrow_mut().insert(digital_world_id, trade));

    let this_canister_account =
        AccountIdentifier::new(&canister_id, &principal_to_subaccount(&canister_id));

    let caller_account = AccountIdentifier::new(
        &caller_principal,
        &principal_to_subaccount(&caller_principal),
    );

    let caller_balance_args = ic_ledger_types::AccountBalanceArgs {
        account: caller_account,
    };
    let caller_balance =
        ic_ledger_types::account_balance(MAINNET_LEDGER_CANISTER_ID, caller_balance_args)
            .await
            .map_err(|_| DepositErr::TransferFailure)?;

    if caller_balance.e8s() < ICP_FEE {
        return Err(DepositErr::BalanceLow);
    }

    let transfer_args = ic_ledger_types::TransferArgs {
        memo: Memo(0),
        amount: caller_balance - Tokens::from_e8s(ICP_FEE),
        fee: Tokens::from_e8s(ICP_FEE),
        from_subaccount: Some(principal_to_subaccount(&caller_principal)),
        to: this_canister_account,
        created_at_time: None,
    };

    ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
        .await
        .map_err(|_| DepositErr::TransferFailure)?
        .map_err(|_| DepositErr::TransferFailure)?;

    ic_cdk::println!(
        "Depósito de {} ICP na conta {:?}",
        caller_balance - Tokens::from_e8s(ICP_FEE),
        &caller_account
    );

    Ok(())
}

/// Função auxiliar para converter o principal em uma subconta
pub fn principal_to_subaccount(principal_id: &Principal) -> Subaccount {
    let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
    let principal_id_slice = principal_id.as_slice();
    subaccount[0] = principal_id_slice.len() as u8;
    subaccount[1..1 + principal_id_slice.len()].copy_from_slice(principal_id_slice);

    Subaccount(subaccount)
}
