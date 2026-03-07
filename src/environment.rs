#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Sandbox,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Endpoints {
    pub api_base_url: &'static str,
}

pub const SANDBOX_ENDPOINTS: Endpoints = Endpoints {
    api_base_url: "https://api-sandbox.asaas.com",
};

pub const PRODUCTION_ENDPOINTS: Endpoints = Endpoints {
    api_base_url: "https://api.asaas.com",
};

impl Environment {
    #[must_use]
    pub const fn endpoints(self) -> Endpoints {
        match self {
            Self::Sandbox => SANDBOX_ENDPOINTS,
            Self::Production => PRODUCTION_ENDPOINTS,
        }
    }
}
