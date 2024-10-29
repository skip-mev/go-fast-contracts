use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr,
};
use go_fast::gateway::InstantiateMsg;
use go_fast_transfer_cw::state::{CONFIG, LOCAL_DOMAIN, NONCE};

pub mod common;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();

    let info = mock_info("creator", &[]);

    let mut env = mock_env();
    env.contract.address = Addr::unchecked("fast_transfer_gateway");

    let instantiate_msg = InstantiateMsg {
        token_denom: "uusdc".to_string(),
        address_prefix: "osmo".to_string(),
        mailbox_addr: "mailbox_contract_address".into(),
        hook_addr: "hook_contract_address".into(),
        local_domain: 1,
    };

    go_fast_transfer_cw::contract::instantiate(deps.as_mut(), env, info, instantiate_msg.clone())
        .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();

    assert_eq!(config.token_denom, instantiate_msg.token_denom);
    assert_eq!(config.address_prefix, instantiate_msg.address_prefix);
    assert_eq!(config.mailbox_addr, instantiate_msg.mailbox_addr);
    assert_eq!(config.hook_addr, instantiate_msg.hook_addr);

    let local_domain = LOCAL_DOMAIN.load(deps.as_ref().storage).unwrap();
    assert_eq!(local_domain, 1);

    let nonce = NONCE.load(deps.as_ref().storage).unwrap();
    assert_eq!(nonce, 0);
}
