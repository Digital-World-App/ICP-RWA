use candid::{candid_method, Nat};
use ic_cdk::api::caller;
use ic_cdk_macros::{query, update};
use std::collections::HashMap;

// Estrutura de dados para armazenar informações sobre itens e vendas
#[derive(Clone, Debug, Default)]
struct Item {
    owner: String,
    price: Nat,
}

#[derive(Clone, Debug, Default)]
struct Sale {
    buyer: Option<String>,
    amount: Nat,
}

// Banco de dados para itens e vendas
static mut ITEMS: Option<HashMap<u64, Item>> = None;
static mut SALES: Option<HashMap<u64, Sale>> = None;

// Função auxiliar para inicializar o banco de dados
fn initialize_data() {
    unsafe {
        if ITEMS.is_none() {
            ITEMS = Some(HashMap::new());
        }
        if SALES.is_none() {
            SALES = Some(HashMap::new());
        }
    }
}

// Função de saudação
#[update]
#[candid_method(update)]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to the Digital World.", name)
}

// Função para comprar um item
#[update]
#[candid_method(update)]
fn buy_item(item_id: u64, amount: Nat) -> Result<String, String> {
    initialize_data();

    unsafe {
        if let Some(items) = &mut ITEMS {
            if let Some(item) = items.get(&item_id) {
                if amount >= item.price {
                    let buyer = caller().to_string();
                    SALES.as_mut().unwrap().insert(
                        item_id,
                        Sale {
                            buyer: Some(buyer.clone()),
                            amount: amount.clone(),
                        },
                    );
                    return Ok(format!("Item {} comprado por {}", item_id, buyer));
                } else {
                    return Err("Valor insuficiente para comprar o item".to_string());
                }
            } else {
                return Err("Item não encontrado".to_string());
            }
        }
    }

    Err("Erro ao processar a compra".to_string())
}

// Função para reivindicar uma venda
#[update]
#[candid_method(update)]
fn claim_sale(item_id: u64) -> Result<String, String> {
    initialize_data();

    unsafe {
        if let Some(sales) = &mut SALES {
            if let Some(sale) = sales.get(&item_id) {
                if sale.buyer.is_some() {
                    let buyer = sale.buyer.clone().unwrap();
                    return Ok(format!(
                        "Venda do item {} reivindicada por {}",
                        item_id, buyer
                    ));
                } else {
                    return Err("Nenhum comprador registrado para este item".to_string());
                }
            } else {
                return Err("Item não encontrado nas vendas".to_string());
            }
        }
    }

    Err("Erro ao processar a reivindicação de venda".to_string())
}

// Função de inicialização de Candid
candid::export_service!();

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Nat;

    #[test]
    fn test_greet() {
        assert_eq!(
            greet("Alice".to_string()),
            "Hello, Alice! Welcome to the Digital World."
        );
    }

    #[test]
    fn test_buy_item_and_claim_sale() {
        let item_id = 1;
        let price = Nat::from(100);
        let buyer_amount = Nat::from(100);

        initialize_data();
        unsafe {
            ITEMS.as_mut().unwrap().insert(
                item_id,
                Item {
                    owner: "Owner".to_string(),
                    price: price.clone(),
                },
            );
        }

        let buy_result = buy_item(item_id, buyer_amount);
        assert!(buy_result.is_ok());

        let claim_result = claim_sale(item_id);
        assert!(claim_result.is_ok());
    }
}

// Exporta o serviço Candid para o arquivo `.did`
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    std::println!("{}", __export_service());
}
