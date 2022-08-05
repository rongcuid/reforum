use eyre::*;

pub fn to_eyre<E: ToString>(e: E) -> Report {
    eyre!(e.to_string())
}