use near_sdk::{ext_contract};

pub const TGAS: u64 = 1_000_000_000_000;

#[ext_contract(this_contract)]
trait Callbacks {
  fn ft_resolve_transfer(&self, sender_id: String, receiver_id: String, amount: String) -> String;
}

#[ext_contract(cross_comunication)]
trait CrossComunication {
  fn ft_on_transfer(&self, sender_id: String, amount: String, memo: Option<String>, msg: String) -> String;
}