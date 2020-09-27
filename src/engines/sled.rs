use async_trait::async_trait;
use sled::Db;

use smol::channel::bounded;

use super::KvsEngine;
use crate::{KvsError, Result, ThreadPool};

/// Wrapper of `sled::Db`
#[derive(Clone)]
pub struct SledKvsEngine<P: ThreadPool> {
    pool: P,
    db: Db,
}

impl<P: ThreadPool> SledKvsEngine<P> {
    /// Creates a `SledKvsEngine` from `sled::Db`.
    ///
    /// Operations are run in the given thread pool. `concurrency` specifies the number of
    /// threads in the thread pool.
    pub fn new(db: Db, concurrency: u32) -> Result<Self> {
        let pool = P::new(concurrency)?;
        Ok(SledKvsEngine { pool, db })
    }
}

#[async_trait]
impl<P: ThreadPool> KvsEngine for SledKvsEngine<P> {
    async fn set(&self, key: String, value: String) -> Result<()> {
        let db = self.db.clone();
        let (tx, rx) = bounded(1);
        self.pool.spawn(move || {
            let res = (|| {
                db.insert(key, value.into_bytes())?;
                db.flush()?;
                Result::<()>::Ok(())
            })();

            smol::block_on(async {
                if tx.send(res).await.is_err() {
                    error!("Receiving end is dropped");
                }
            })
        });

        rx.recv().await?
    }

    async fn get(&self, key: String) -> Result<Option<String>> {
        let db = self.db.clone();
        let (tx, rx) = bounded(1);
        self.pool.spawn(move || {
            let res = (move || {
                Ok(db
                    .get(key)?
                    .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
                    .map(String::from_utf8)
                    .transpose()?) as Result<Option<String>>
            })();

            smol::block_on(async {
                if tx.send(res).await.is_err() {
                    error!("Receiving end is dropped");
                }
            })
        });

        rx.recv().await?
    }

    async fn remove(&self, key: String) -> Result<()> {
        let db = self.db.clone();
        let (tx, rx) = bounded(1);
        self.pool.spawn(move || {
            let res = (|| {
                match db.remove(key) {
                    Ok(o) => match o {
                        Some(_) => Ok(()),
                        None => Err(KvsError::KeyNotFound),
                    },
                    Err(e) => Err(KvsError::Sled(e)),
                }?;
                db.flush()?;
                Result::<()>::Ok(())
            })();

            smol::block_on(async {
                if tx.send(res).await.is_err() {
                    error!("Receiving end is dropped");
                }
            })
        });

        rx.recv().await?
    }
}
