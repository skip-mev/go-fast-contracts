use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Deps, HexBinary, StdResult};

#[cw_serde]
pub struct DispatchMsg {
    pub dest_domain: u32,
    pub recipient_addr: HexBinary,
    pub msg_body: HexBinary,
    pub hook: Option<String>,
    pub metadata: Option<HexBinary>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Dispatch(DispatchMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Hook(MailboxHookQueryMsg),
    Mailbox(MailboxQueryMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum MailboxHookQueryMsg {
    #[returns(QuoteDispatchResponse)]
    QuoteDispatch { sender: String, msg: DispatchMsg },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum MailboxQueryMsg {
    #[returns(DefaultHookResponse)]
    DefaultHook {},

    #[returns(RequiredHookResponse)]
    RequiredHook {},
}

#[cw_serde]
pub struct DefaultHookResponse {
    pub default_hook: String,
}

#[cw_serde]
pub struct RequiredHookResponse {
    pub required_hook: String,
}

#[cw_serde]
pub struct QuoteDispatchResponse {
    pub fees: Vec<Coin>,
}

pub fn get_default_hook(deps: Deps, mailbox: impl Into<String>) -> StdResult<String> {
    let response: DefaultHookResponse = deps
        .querier
        .query_wasm_smart(mailbox, &QueryMsg::Mailbox(MailboxQueryMsg::DefaultHook {}))?;

    Ok(response.default_hook)
}

pub fn get_required_hook(deps: Deps, mailbox: impl Into<String>) -> StdResult<String> {
    let response: RequiredHookResponse = deps.querier.query_wasm_smart(
        mailbox,
        &QueryMsg::Mailbox(MailboxQueryMsg::RequiredHook {}),
    )?;

    Ok(response.required_hook)
}

pub fn quote_dispatch(
    deps: Deps,
    mailbox: impl Into<String>,
    msg: DispatchMsg,
) -> StdResult<Vec<Coin>> {
    let mailbox: String = mailbox.into();

    let response: QuoteDispatchResponse = deps.querier.query_wasm_smart(
        mailbox.clone(),
        &QueryMsg::Hook(MailboxHookQueryMsg::QuoteDispatch {
            sender: mailbox,
            msg,
        }),
    )?;

    Ok(response.fees)
}
