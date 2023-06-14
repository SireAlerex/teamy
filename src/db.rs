use bson::Document;
use mongodb::bson::doc;
use mongodb::bson::to_document;
use mongodb::{
    error::Error,
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};
use serenity::futures::TryStreamExt;
use serenity::prelude::Context;

use crate::DatabaseUriContainer;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct User {
    _id: mongodb::bson::oid::ObjectId,
    user_id: String,
}

impl User {
    pub fn builder(user_id: String) -> User {
        User {
            _id: mongodb::bson::oid::ObjectId::new(),
            user_id,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Chan {
    _id: mongodb::bson::oid::ObjectId,
    channel_id: String,
}

impl Chan {
    pub fn builder(channel_id: String) -> Chan {
        Chan {
            _id: mongodb::bson::oid::ObjectId::new(),
            channel_id,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Guild {
    _id: mongodb::bson::oid::ObjectId,
    guild_id: String,
}

impl Guild {
    pub fn builder(guild_id: String) -> Guild {
        Guild {
            _id: mongodb::bson::oid::ObjectId::new(),
            guild_id,
        }
    }
}

pub fn mongodb_error(message: impl ToString) -> Error {
    Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        message.to_string(),
    ))
}

async fn get_client(ctx: &Context) -> Result<Client, Error> {
    let data = ctx.data.read().await;
    let db = data.get::<DatabaseUriContainer>().unwrap().lock().await;
    let options =
        ClientOptions::parse_with_resolver_config(&db.db_uri, ResolverConfig::cloudflare()).await?;
    Client::with_options(options)
}

pub async fn get_coll<'a, T: core::fmt::Debug + serde::Deserialize<'a> + serde::Serialize>(
    ctx: &Context,
    collection: &str,
) -> Result<Collection<T>, Error> {
    Ok(get_client(ctx)
        .await?
        .database("teamy")
        .collection(collection))
}

pub async fn get_object<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    object: &T,
) -> Result<Option<T>, Error> {
    let coll: Collection<T> = get_coll(ctx, collection).await?;
    let mut doc_filter = to_document(&object)?;
    doc_filter.remove("_id");
    let o = coll.find_one(doc_filter, None).await?;
    Ok(o)
}

pub async fn get_objects<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    filter: impl Into<Option<Document>>,
) -> Result<Vec<T>, Error> {
    get_coll::<T>(ctx, collection)
        .await?
        .find(filter, None)
        .await?
        .try_collect::<Vec<T>>()
        .await
}

pub async fn is_object_in_coll<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    object: &T,
) -> Result<bool, Error> {
    match get_object(ctx, collection, object).await? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

pub async fn insert<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    object: &T,
) -> Result<(), Error> {
    let coll: Collection<T> = get_coll(ctx, collection).await?;
    if !is_object_in_coll(ctx, collection, object).await? {
        let _ = coll.insert_one(object, None).await?;
        Ok(())
    } else {
        Err(mongodb_error("l'objet à insérer existe déjà"))
    }
}

pub async fn update<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    object: &T,
    update: &bson::Document,
) -> Result<(), Error> {
    let coll: Collection<T> = get_coll(ctx, collection).await?;
    if let Ok(o) = get_object(ctx, collection, object).await {
        match o {
            Some(res) => {
                let _ = coll
                    .update_one(
                        doc! {"_id": to_document(&res).unwrap().get("_id")},
                        (*update).clone(),
                        None,
                    )
                    .await?;
                Ok(())
            }
            None => Err(mongodb_error("l'objet à modifier n'existe pas")),
        }
    } else {
        Err(mongodb_error(format!(
            "erreur pour accéder à l'objet : {:?}",
            object
        )))
    }
}

pub async fn delete<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    object: &T,
) -> Result<(), Error> {
    let coll: Collection<T> = get_coll(ctx, collection).await?;
    if let Ok(o) = get_object(ctx, collection, object).await {
        match o {
            Some(res) => {
                let _ = coll
                    .delete_one(doc! {"_id": to_document(&res).unwrap().get("_id")}, None)
                    .await?;
                Ok(())
            }
            None => Err(mongodb_error("l'objet à supprimer n'existe pas")),
        }
    } else {
        Err(mongodb_error(format!(
            "erreur pour accéder à l'objet : {:?}",
            object
        )))
    }
}

pub async fn find_filter<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    filter: impl Into<Option<Document>>,
) -> Result<Option<T>, Error> {
    get_coll::<T>(ctx, collection)
        .await?
        .find_one(filter, None)
        .await
}

pub async fn delete_filter<
    T: core::fmt::Debug
        + serde::de::DeserializeOwned
        + serde::Serialize
        + std::marker::Unpin
        + std::marker::Send
        + std::marker::Sync,
>(
    ctx: &Context,
    collection: &str,
    query: Document,
) -> Result<(), Error> {
    get_coll::<T>(ctx, collection)
        .await?
        .delete_one(query, None)
        .await?;
    Ok(())
}
