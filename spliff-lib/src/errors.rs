pub enum SpliffError {
    SolanaAPIError(String),
    SolanaProgramError(String),
    InputError(String),
}
