use candid::{CandidType, Deserialize};
use ic_stable_memory::derive::{CandidAsDynSizeBytes, StableType};
use lazy_static::lazy_static;
use network::nw::SupportedNetwork;

#[derive(
    Default,
    CandidType,
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Deserialize,
    StableType,
    CandidAsDynSizeBytes,
)]
pub enum Token {
    #[default]
    DAI,
}

lazy_static! {
    static ref DAI_MAINNET: TokenConfig = TokenConfig::new(SupportedNetwork::Mainnet, Token::DAI);
    static ref DAI_OPTIMISM: TokenConfig = TokenConfig::new(SupportedNetwork::Optimism, Token::DAI);
}

#[derive(
    CandidType, Debug, Clone, PartialEq, PartialOrd, Deserialize, StableType, CandidAsDynSizeBytes,
)]
pub struct TokenConfig {
    pub network: SupportedNetwork,
    token: Token,
}

impl TokenConfig {
    pub fn new(network: SupportedNetwork, token: Token) -> Self {
        Self { network, token }
    }
    pub fn address(&self) -> &str {
        self.token.address(&self.network)
    }
    pub fn contract_craeted_block_number(&self) -> u64 {
        self.token.contract_craeted_block_number(&self.network)
    }
}

impl Token {
    fn address(&self, network: &SupportedNetwork) -> &str {
        match (self, network) {
            (Token::DAI, SupportedNetwork::Mainnet) => "0x6b175474e89094c44da98b954eedeac495271d0f",
            (Token::DAI, SupportedNetwork::Optimism) => {
                "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"
            }
            (Token::DAI, SupportedNetwork::PolygonMumbai) => "",
        }
    }
    fn contract_craeted_block_number(&self, network: &SupportedNetwork) -> u64 {
        match (self, network) {
            (Token::DAI, SupportedNetwork::Mainnet) => 8928158,
            (Token::DAI, SupportedNetwork::Optimism) => 0,
            (Token::DAI, SupportedNetwork::PolygonMumbai) => 0,
        }
    }
}
