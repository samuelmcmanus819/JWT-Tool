#[cfg(test)]
mod tests {
  use crate::execute::{instantiate, execute};
  use crate::msg::{
    InstantiateMsg, ExecuteMsg, QueryMsg, ValidateResponse, PubKeyResponse
  };
use crate::query::query;
use crate::state::{Header, Payload, privkey_read};

use base64::Engine;
use base64::engine::general_purpose;
use cosmwasm_std::{
    testing::*, 
    Response, 
    MessageInfo, 
    DepsMut, from_binary, Api, Binary, to_vec,
  };
  use cosmwasm_std::{ Coin, Uint128 };
use secret_toolkit_crypto::secp256k1::PrivateKey;

  fn execute_instantiate(deps: DepsMut, info: MessageInfo) {
    let hours_until_token_expiration: u8 = 24;
    let init_msg: InstantiateMsg = InstantiateMsg { hours_until_token_expiration };

    // we can just call .unwrap() to assert this was a success
    let _res: Response = instantiate(deps, mock_env(), info, init_msg).unwrap();
  }


  #[test]
  fn proper_initialization() {
    //Set up dependencies and single user's wallet
    let mut deps = mock_dependencies();
    let info = mock_info(
      "creator",
      &[Coin {
        denom: "earth".to_string(),
        amount: Uint128::new(1000),
      }],
    );
    //Instantiate the contract
    execute_instantiate(deps.as_mut(), info.clone());
  }

  #[test]
  fn provision_success() {
    //Set up dependencies and single user's wallet
    let mut deps = mock_dependencies();
    let info = mock_info(
      "creator",
      &[Coin {
        denom: "earth".to_string(),
        amount: Uint128::new(1000),
      }],
    );
    execute_instantiate(deps.as_mut(), info.clone());

    //Provision a JWT
    let provision: ExecuteMsg = ExecuteMsg::Provision{  };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), provision).unwrap();
    let jwt = (&res.attributes[0].value).clone();

    //Verify that the JWT is valid
    let msg = QueryMsg::ValidateJwt { jwt };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let valid_response: ValidateResponse = from_binary(&res).unwrap();
    assert_eq!(valid_response.valid, true);
  }

  #[test]
  fn validate_bad_payload() {
    //Set up dependencies and single user's wallet
    let mut deps = mock_dependencies();
    let info = mock_info(
      "creator",
      &[Coin {
        denom: "earth".to_string(),
        amount: Uint128::new(1000),
      }],
    );
    execute_instantiate(deps.as_mut(), info.clone());

    //Provision a JWT
    let provision: ExecuteMsg = ExecuteMsg::Provision{  };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), provision).unwrap();
    let mut jwt = (&res.attributes[0].value).clone();

    //Modify the payload
    let index = jwt.find('.').unwrap();
    jwt.replace_range(index+2..index+3, "X");

    //Verify that the JWT is invalid
    let msg = QueryMsg::ValidateJwt { jwt };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let valid_response: ValidateResponse = from_binary(&res).unwrap();
    assert_ne!(valid_response.valid, true);

  }

  #[test]
  fn validate_expired_token(){
  //Set up dependencies and single user's wallet
  let mut deps = mock_dependencies();
  let info = mock_info(
    "creator",
    &[Coin {
      denom: "earth".to_string(),
      amount: Uint128::new(1000),
    }],
  );
  execute_instantiate(deps.as_mut(), info.clone());

  let env = mock_env();
    //Set expiration time to current time MINUS 1 day
  let exp = env.block.time.minus_seconds(86400);
  //Generate the header for the token
  let header = to_vec(&Header { typ: String::from("JWT"), alg: String::from("HS256") }).unwrap();
  //Generate the token payload, consisting of the user's address and the expiration time
  let payload = to_vec(&Payload { address: info.sender.clone().into_string(), exp }).unwrap();
  //Encode the header and payload as URL-safe base64
  let encoded_header: String = general_purpose::URL_SAFE_NO_PAD.encode(header.clone());
  let encoded_payload: String = general_purpose::URL_SAFE_NO_PAD.encode(payload.clone());

  //Convert the header and payload to bytes so that they can be signed
  let mut msg_to_sign: Vec<u8> = vec![];
  msg_to_sign.append(&mut header.clone().to_vec());
  msg_to_sign.append(&mut payload.clone().to_vec());
  //Get the private key from contract state
  let private_key = PrivateKey::parse(&privkey_read(&deps.storage).load().unwrap()).unwrap();

  //Sign the hashed header and payload
  let signature = Binary::from(deps.api.secp256k1_sign(&msg_to_sign, &private_key.serialize()).unwrap());
  let encoded_signature: String = signature.to_base64();

  //Return the JWT to the user
  let jwt = encoded_header + "." + &encoded_payload + "." + &encoded_signature;

  //Verify that the JWT is invalid
  let msg = QueryMsg::ValidateJwt { jwt };
  let res = query(deps.as_ref(), mock_env(), msg).unwrap();
  let valid_response: ValidateResponse = from_binary(&res).unwrap();
  assert_ne!(valid_response.valid, true);
  }

  #[test]
  fn get_public_key() {
    //Set up dependencies and single user's wallet
    let mut deps = mock_dependencies();
    let info = mock_info(
      "creator",
      &[Coin {
        denom: "earth".to_string(),
        amount: Uint128::new(1000),
      }],
    );
    execute_instantiate(deps.as_mut(), info.clone());

    let msg = QueryMsg::GetPubKey {  };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let _pubkey_response: PubKeyResponse = from_binary(&res).unwrap();
  }
}