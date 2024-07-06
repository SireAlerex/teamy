use bson::Bson;
use bson::Document;
use mongodb::bson::doc;
use mongodb::bson::to_document;
use mongodb::options::UpdateModifications;
use mongodb::results::DeleteResult;
use mongodb::results::UpdateResult;
use mongodb::{error::Error, options::ClientOptions, Client, Collection};
use serenity::futures::TryStreamExt;
use serenity::prelude::Context;

use crate::DatabaseUriContainer;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct User {
    _id: mongodb::bson::oid::ObjectId,
    user_id: String,
}

impl User {
    pub fn builder(user_id: String) -> Self {
        Self {
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
    pub fn builder(channel_id: String) -> Self {
        Self {
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
    pub fn builder(guild_id: String) -> Self {
        Self {
            _id: mongodb::bson::oid::ObjectId::new(),
            guild_id,
        }
    }
}

pub fn mongodb_error<T: Into<String>>(message: T) -> Error {
    Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        message.into(),
    ))
}

async fn get_client(ctx: &Context) -> Result<Client, Error> {
    let data = ctx.data.read().await;
    if let Some(db) = data.get::<DatabaseUriContainer>() {
        let options = ClientOptions::parse(&db.0).await?; // FIXME: very slow
        Client::with_options(options)
    } else {
        Err(mongodb_error("no db uri"))
    }
}

pub async fn get_coll<
    'a,
    T: core::fmt::Debug + serde::Deserialize<'a> + serde::Serialize + Send + Sync,
>(
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
    let o = coll.find_one(doc_filter).await?;
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
    if let Some(document) = filter.into() {
        get_coll::<T>(ctx, collection)
            .await?
            .find(document)
            .await?
            .try_collect::<Vec<T>>()
            .await
    } else {
        Err(mongodb_error("get_objects: can't filter into document"))
    }
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
) -> Result<Bson, Error> {
    let coll: Collection<T> = get_coll(ctx, collection).await?;
    if is_object_in_coll(ctx, collection, object).await? {
        Err(mongodb_error("l'objet à insérer existe déjà"))
    } else {
        Ok(coll.insert_one(object).await?.inserted_id)
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
                let _: UpdateResult = coll
                    .update_one(
                        doc! {"_id": to_document(&res)?.get("_id")},
                        (*update).clone(),
                    )
                    .await?;
                Ok(())
            }
            None => Err(mongodb_error("l'objet à modifier n'existe pas")),
        }
    } else {
        Err(mongodb_error(format!(
            "erreur pour accéder à l'objet : {object:?}"
        )))
    }
}

pub async fn update_query<
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
    update: impl Into<UpdateModifications>,
) -> Result<UpdateResult, Error> {
    let coll: Collection<T> = get_coll(ctx, collection).await?;
    coll.update_one(query, update).await
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
                let _: DeleteResult = coll
                    .delete_one(doc! {"_id": to_document(&res)?.get("_id")})
                    .await?;
                Ok(())
            }
            None => Err(mongodb_error("l'objet à supprimer n'existe pas")),
        }
    } else {
        Err(mongodb_error(format!(
            "erreur pour accéder à l'objet : {object:?}"
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
    if let Some(document) = filter.into() {
        get_coll::<T>(ctx, collection)
            .await?
            .find_one(document)
            .await
    } else {
        Err(mongodb_error("find_filter: can't filter into document"))
    }
}

pub async fn delete_query<
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
        .delete_one(query)
        .await?;
    Ok(())
}

pub async fn delete_multiple_query<
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
        .delete_many(query)
        .await?;
    Ok(())
}
