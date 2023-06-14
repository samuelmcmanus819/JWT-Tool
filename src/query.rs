use base64::{engine::general_purpose, Engine};
use cosmwasm_std::{
  Deps,
  Env, 
  StdResult, 
  entry_point, 
  Binary, 
  to_binary, StdError, from_slice
};
use secret_toolkit_crypto::{
  secp256k1::{PrivateKey}, 
  sha_256
};

use crate::{
  msg::{
    ValidateResponse,
    QueryMsg, PubKeyResponse
  }, 
  state::{privkey_read, Payload}, 
};

//Route the user's query to the appropriate function
#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
  match msg {
    QueryMsg::ValidateJwt { jwt } => to_binary(&validate_jwt(deps, env, jwt)?),
    QueryMsg::GetPubKey {  } => to_binary(&get_public_key(deps)?)
  }
}

//Validate the user's JWT
fn validate_jwt(deps: Deps, env: Env, jwt: String) -> StdResult<ValidateResponse> {
  let jwt_sections: Vec<&str> = jwt.split(".").collect::<Vec<&str>>();

  //First check the expiration time of the JWT
  let payload = match general_purpose::URL_SAFE_NO_PAD.decode(jwt_sections[1]){
    Ok(payload) => payload,
    Err(_) => { return Ok(ValidateResponse { valid: false }); }
  };
  let payload: Payload = match from_slice(payload.as_slice()) {
    Ok(payload) => payload,
    Err(_) => { return Ok(ValidateResponse { valid: false }) }
  };
  if payload.exp < env.block.time {
    return Ok(ValidateResponse { valid: false });
  }

  //Hash the header and payload of the token and decode them from base64
  let msg_hash: &[u8] = &sha_256(&match general_purpose::URL_SAFE_NO_PAD.decode(&(String::from(jwt_sections[0]) + jwt_sections[1])){
    Ok(payload) => payload,
    Err(_) => { return Ok(ValidateResponse { valid: false }); }
  });
  //Grab the signature on the token and decode it from base64
  let signature = match general_purpose::URL_SAFE_NO_PAD.decode(jwt_sections[2]){
    Ok(signature) => signature,
    Err(_) => { return Ok(ValidateResponse { valid: false }); }
  };
  //Derive the public key from our private key
  let private_key: PrivateKey = match PrivateKey::parse(&privkey_read(deps.storage).load()?) {
    Ok(key) => key,
    Err(_) => { return StdResult::Err(StdError::NotFound { kind: String::from("Private key not found") }); }
  };
  let public_key: &[u8] = &private_key.pubkey().serialize();

  //Validate the integrity of the token
  match deps.api.secp256k1_verify(msg_hash, signature.as_slice(), public_key){
    Ok(valid) => match valid {
      true => {  return Ok(ValidateResponse { valid: true }); }
      false => { return Ok(ValidateResponse { valid: false }); }
    }
    Err(e) => { return StdResult::Err(StdError::VerificationErr { source: e }) }
  }
}

fn get_public_key(deps: Deps) -> StdResult<PubKeyResponse> {
  //Pull the private key from storage
  let private_key: PrivateKey = match PrivateKey::parse(&privkey_read(deps.storage).load()?) {
    Ok(key) => key,
    Err(_) => { return StdResult::Err(StdError::NotFound { kind: String::from("Private key not found") }); }
  };

  //Derive the public key from the private key, encode it to base64, and return it
  let public_key: [u8; 65] = private_key.pubkey().serialize();
  Ok(PubKeyResponse { pubkey: general_purpose::URL_SAFE_NO_PAD.encode(public_key) })
}