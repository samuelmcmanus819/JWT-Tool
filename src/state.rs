use schemars::JsonSchema;
use secret_toolkit::{
  storage::{
    Item 
  }
};
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
  Storage, 
  Timestamp, 
};
use cosmwasm_storage::{
  singleton, 
  singleton_read, 
  ReadonlySingleton, 
  Singleton
};

pub const PRIVKEY_KEY: &[u8] = b"privkey";
pub const EXPIRY_KEY: &[u8] = b"expiry";


#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct Header {
  pub typ: String,
  pub alg: String
}


#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct Payload {
  pub address: String,
  pub exp: Timestamp
}

pub static EXPIRY: Item<u16> = Item::new(EXPIRY_KEY);

pub fn expiry(storage: &mut dyn Storage) -> Singleton<u64> {
  singleton(storage, EXPIRY_KEY)
}

pub fn expiry_read(storage: &dyn Storage) -> ReadonlySingleton<u64>{
  singleton_read(storage, EXPIRY_KEY)
}

pub static PRIVKEY: Item<[u8; 32]> = Item::new(PRIVKEY_KEY);

pub fn privkey(storage: &mut dyn Storage) -> Singleton<[u8; 32]> {
  singleton(storage, PRIVKEY_KEY)
}

pub fn privkey_read(storage: &dyn Storage) -> ReadonlySingleton<[u8; 32]> {
  singleton_read(storage, PRIVKEY_KEY)
}
