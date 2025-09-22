use crate::handlers::common::{get_error, get_opts};
use async_trait::async_trait;
use custom_logger as log;
use hyper::body::Bytes;
use serde_derive::{Deserialize, Serialize};

#[async_trait]
pub trait DocumentformInterface {
    async fn save_formdata(
        db: String,
        original_key: String,
        gemini_document: String,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_formdata(
        db_path: String,
        key: String,
    ) -> Result<FormData, Box<dyn std::error::Error>>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormData {
    pub key: Option<String>,
    pub title: String,
    pub file: String,
    pub category: String,
    pub prompt: String,
    pub credentials: String,
    pub run_once: String,
    pub db: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {}

#[async_trait]
impl DocumentformInterface for Document {
    async fn save_formdata(
        db_path: String,
        original_key: String,
        gemini_document: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut fd = db_read(format!("{}/queue", db_path), original_key.clone()).await?;
        fd.key = Some(original_key.clone());
        let json_data = serde_json::to_string(&fd)?;
        db_upsert(
            format!("{}/archive", db_path.clone()),
            original_key.clone(),
            json_data,
        )
        .await?;
        log::debug!(
            "[save_formdata] saving gemini document with key {} ",
            fd.file,
        );
        let result = db_upsert(format!("{}/documents", db_path), fd.file, gemini_document).await?;
        db_delete(format!("{}/queue", db_path), original_key).await?;
        Ok(result)
    }

    async fn get_formdata(db: String, key: String) -> Result<FormData, Box<dyn std::error::Error>> {
        let fd = db_read(db, key).await?;
        Ok(fd)
    }
}

async fn db_upsert(
    db: String,
    id: String,
    data: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let tree = get_opts(db)?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    txn.set_durability(surrealkv::Durability::Immediate);
    let key = Bytes::from(id.clone());
    let value = Bytes::from(data);
    txn.set(&key, &value)
        .map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    let msg = format!("form data {} created/updated successfully", id);
    Ok(msg)
}

async fn db_read(db: String, key: String) -> Result<FormData, Box<dyn std::error::Error>> {
    let tree = get_opts(db)?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    let b_key = Bytes::from(key.clone());
    log::debug!("[db_read] key {}", key);
    let result = txn.get(&b_key).map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    match result {
        Some(value) => {
            let fd = serde_json::from_slice(&value).map_err(|e| get_error(e.to_string()))?;
            log::trace!("[db_read] {:?}", fd);
            Ok(fd)
        }
        None => {
            let fd = FormData {
                key: None,
                title: "".to_string(),
                file: "".to_string(),
                category: "".to_string(),
                prompt: "".to_string(),
                credentials: "".to_string(),
                run_once: "on".to_string(),
                db: "formdata".to_string(),
            };
            Ok(fd)
        }
    }
}

async fn db_delete(db: String, key: String) -> Result<(), Box<dyn std::error::Error>> {
    let tree = get_opts(db)?;
    // start transaction
    let mut txn = tree.begin().map_err(|e| get_error(e.to_string()))?;
    let b_key = Bytes::from(key.clone());
    log::debug!("[db_read] key {}", key);
    txn.delete(&b_key).map_err(|e| get_error(e.to_string()))?;
    // commit transaction
    txn.commit().await?;
    tree.close().await?;
    Ok(())
}
