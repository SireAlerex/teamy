use std::str::FromStr;

use anyhow::anyhow;
use shuttle_runtime::Error;
use shuttle_secrets::SecretStore;

pub fn get(secret_store: &SecretStore, key: &str) -> Result<String, Error> {
    secret_store
        .get(key)
        .ok_or_else(|| anyhow!("'{key}' was not found").into())
}

pub fn parse<T: FromStr>(secret_store: &SecretStore, key: &str) -> Result<T, Error> {
    match secret_store.get(key) {
        Some(s) => s
            .parse::<T>()
            .map_err(|_err| anyhow!("'{key}' should be u64").into()),
        None => Err(anyhow!("'{key}' was not found").into()),
    }
}

pub fn parse_objects<T: FromStr, F: From<T>>(
    secret_store: &SecretStore,
    key: &str,
) -> Result<Vec<F>, Error> {
    match secret_store.get(key) {
        Some(s) => s
            .split(',')
            .map(str::parse::<T>)
            .map(|x| {
                x.map_err(|_err| anyhow!("'{key}' should be {}", std::any::type_name::<T>()).into())
                    .map(F::from)
            })
            .collect(),
        None => Err(anyhow!("'{key}' was not found").into()),
    }
}
