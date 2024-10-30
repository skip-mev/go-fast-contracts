use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env, MockApi, MockQuerier},
    to_json_binary, Addr, ContractResult, DepsMut, Env, HexBinary, MemoryStorage, MessageInfo,
    OwnedDeps, QuerierResult, SystemResult, WasmQuery,
};
use go_fast::{
    gateway::{Config, ExecuteMsg},
    helpers::keccak256_hash,
    FastTransferOrder,
};
use go_fast_transfer_cw::{
    error::ContractResponse,
    helpers::bech32_encode,
    state::{CONFIG, LOCAL_DOMAIN, NONCE, REMOTE_DOMAINS},
};
use hyperlane::mailbox::{DefaultHookResponse, QueryMsg as HplQueryMsg, RequiredHookResponse};

pub fn default_instantiate() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
    let mut deps = mock_dependencies();

    let mut env = mock_env();
    env.contract.address = Addr::unchecked("fast_transfer_gateway");

    NONCE.save(deps.as_mut().storage, &0).unwrap();

    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                token_denom: "uusdc".to_string(),
                address_prefix: "osmo".to_string(),
                mailbox_addr: bech32_encode(
                    "osmo",
                    &keccak256_hash("mailbox_contract_address".as_bytes()),
                )
                .unwrap()
                .into_string(),
                hook_addr: "hook_contract_address".into(),
            },
        )
        .unwrap();

    LOCAL_DOMAIN.save(deps.as_mut().storage, &1).unwrap();

    REMOTE_DOMAINS
        .save(
            deps.as_mut().storage,
            2,
            &HexBinary::from_hex(
                "0000000000000000000000005B16CfB4Fa672d351760a189278406013a61B231",
            )
            .unwrap(),
        )
        .unwrap();

    let wasm_handler = |query: &WasmQuery| -> QuerierResult {
        match query {
            WasmQuery::Smart { contract_addr, msg } => {
                if contract_addr
                    == &bech32_encode(
                        "osmo",
                        &keccak256_hash("mailbox_contract_address".as_bytes()),
                    )
                    .unwrap()
                    .into_string()
                {
                    let msg: HplQueryMsg = from_json(msg).unwrap();
                    match msg {
                        HplQueryMsg::Hook(_) => todo!(),
                        HplQueryMsg::Mailbox(msg) => match msg {
                            hyperlane::mailbox::MailboxQueryMsg::DefaultHook {} => {
                                return SystemResult::Ok(ContractResult::Ok(
                                        to_json_binary(&DefaultHookResponse {
                                            default_hook: "osmo12pvc4v625ewl34uqqgm3ezw76durxlky5j4guz8kvhal7em3e5wqz7cnla".into(),
                                        })
                                            .unwrap(),
                                        ));
                            }
                            hyperlane::mailbox::MailboxQueryMsg::RequiredHook {} => {
                                return SystemResult::Ok(ContractResult::Ok(
                                            to_json_binary(&RequiredHookResponse {
                                                required_hook: "osmo1hsztuzngm4skzjejqxw8kwg4dg39nr3jzwwp38638pqe8kg03nyqtzuw0l".into(),
                                            })
                                            .unwrap(),
                                        ));
                            }
                        },
                    }
                }

                panic!("Unsupported query: {:?}", query)
            }
            _ => panic!("Unsupported query: {:?}", query),
        }
    };

    deps.querier.update_wasm(wasm_handler);

    (deps, env)
}

pub fn submit_order(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    order: &FastTransferOrder,
) -> ContractResponse {
    let execute_msg = ExecuteMsg::SubmitOrder {
        sender: order.sender.clone(),
        recipient: order.recipient.clone(),
        amount_in: order.amount_in,
        amount_out: order.amount_out,
        destination_domain: order.destination_domain,
        timeout_timestamp: order.timeout_timestamp,
        data: order.data.clone(),
    };

    go_fast_transfer_cw::contract::execute(deps, env.clone(), info.clone(), execute_msg)
}
