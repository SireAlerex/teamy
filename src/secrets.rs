use std::str::FromStr;

use anyhow::anyhow;
use shuttle_runtime::Error;
use shuttle_runtime::SecretStore;

pub fn get(secret_store: &SecretStore, key: &str) -> Result<String, Error> {
    secret_store
        .get(key)
        .ok_or_else(|| anyhow!("'{key}' was not found").into())
}

pub fn parse<T: FromStr>(secret_store: &SecretStore, key: &str) -> Result<T, Error> {
    secret_store.get(key).map_or_else(
        || Err(anyhow!("'{key}' was not found").into()),
        |s| {
            s.parse::<T>()
                .map_err(|_err| anyhow!("'{key}' should be u64").into())
        },
    )
}

pub fn parse_objects<T: FromStr, F: From<T>>(
    secret_store: &SecretStore,
    key: &str,
) -> Result<Vec<F>, Error> {
    secret_store.get(key).map_or_else(
        || Err(anyhow!("'{key}' was not found").into()),
        |s| {
            s.split(',')
                .map(str::parse::<T>)
                .map(|x| {
                    x.map_err(|_err| {
                        anyhow!("'{key}' should be {}", std::any::type_name::<T>()).into()
                    })
                    .map(F::from)
                })
                .collect()
        },
    )
}
