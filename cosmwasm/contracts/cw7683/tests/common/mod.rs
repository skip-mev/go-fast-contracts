use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, MockApi, MockQuerier},
    Addr, Env, MemoryStorage, OwnedDeps,
};
use cw_7683::state::GATEWAY_ADDRESS;

pub fn default_instantiate() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
    let mut deps = mock_dependencies();

    let mut env = mock_env();
    env.contract.address = Addr::unchecked("cw7683-settler");

    GATEWAY_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("go-fast-gateway"))
        .unwrap();

    (deps, env)
}
