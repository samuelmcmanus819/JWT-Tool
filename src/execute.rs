use base64::{engine::general_purpose, Engine};
use cosmwasm_std::{
  DepsMut, 
  Env, 
  MessageInfo, 
  Response, 
  entry_point, 
  StdError, 
  to_vec, 
};

use crate::{error::ContractError, state::{expiry, expiry_read}};
use secret_toolkit_crypto::{
  secp256k1::{
    PrivateKey, 
  }
};

use crate::{
  state::{
    Header, 
    Payload, 
    privkey, 
    privkey_read, 
  }, 
  msg::{
    ExecuteMsg, 
    InstantiateMsg
  }
};

//Instantiate the smart contract
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> { 
  let expire_period: u64 = msg.hours_until_token_expiration as u64 * 3600;
  expiry(deps.storage).save(&expire_period)?;

  //Generate a private key based on the environment block random variable
  let rng: [u8; 32] = match env.block.random {
    Some(random) => random.to_array()?,
    None => { return Err(ContractError::KeygenError{ failed_value: String::from("Random number generation") }); }
  };
  let private_key = match PrivateKey::parse(&rng){
    Ok(key) => key,
    Err(_) => { return Err(ContractError::KeygenError{ failed_value: String::from("Privkey generation") }); }
  };
  privkey(deps.storage).save(&private_key.serialize())?;

  Ok(Response::default())
}

//Route execute messages to their appropriate destination
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::Provision { } => try_provision(deps, env, info),
  }
}

//Provision a JWT to the user which expires in a user-defined expiration period
fn try_provision(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
  //Set expiration time to current time + user-defined expiration period
  let expiration_period = expiry_read(deps.storage).load()?;
  let exp = env.block.time.plus_seconds(expiration_period as u64);
  //Generate the header for the token
  let header = match to_vec(&Header { typ: String::from("JWT"), alg: String::from("HS256") }.clone()){
    Ok(header) => header,
    Err(_) => { return Err(ContractError::ProvisionError {  }); }
  };
  //Generate the token payload, consisting of the user's address and the expiration time
  let payload = match to_vec(&Payload { address: info.sender.clone().into_string(), exp }.clone()){
    Ok(payload) => payload,
    Err(_) => { return Err(ContractError::ProvisionError {  }); }
  };
  //Encode the header and payload as URL-safe base64
  let encoded_header: String = general_purpose::URL_SAFE_NO_PAD.encode(header.clone());
  let encoded_payload: String = general_purpose::URL_SAFE_NO_PAD.encode(payload.clone());

  //Convert the header and payload to bytes so that they can be signed
  let mut msg_to_sign: Vec<u8> = vec![];
  msg_to_sign.append(&mut header.clone().to_vec());
  msg_to_sign.append(&mut payload.clone().to_vec());
  //Get the private key from contract state
  let private_key = match PrivateKey::parse( &privkey_read(deps.storage).load()?) {
    Ok(key) => key,
    Err(_) => { return Err(ContractError::Std(StdError::NotFound { kind: String::from("Private key not found") })) }
  };

  //Sign the hashed header and payload
  let signature = match deps.api.secp256k1_sign(&msg_to_sign, &private_key.serialize()){
    Ok(signature) => signature,
    Err(e) => { return Err(ContractError::Std(StdError::SigningErr { source: e })) }
  };
  let encoded_signature: String = general_purpose::URL_SAFE_NO_PAD.encode(signature);

  //Return the JWT to the user
  let jwt = encoded_header + "." + &encoded_payload + "." + &encoded_signature;
  Ok(Response::new().add_attribute("jwt", jwt))
}

