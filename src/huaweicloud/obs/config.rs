/// Config carries all the configuration for Huawei Cloud OBS services.
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Config {
    /// `access_key_id` will be loaded from
    ///
    /// - this field if it's `is_some`
    pub access_key_id: Option<String>,
    /// `secret_access_key` will be loaded from
    ///
    /// - this field if it's `is_some`
    pub secret_access_key: Option<String>,
    /// `security_token` will be loaded from
    ///
    /// - this field if it's `is_some`
    pub security_token: Option<String>,
}
