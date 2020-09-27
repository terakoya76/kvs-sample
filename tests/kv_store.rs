use rayon::prelude::*;
use smol::Executor;
use tempfile::TempDir;
use walkdir::WalkDir;

use kvs::{KvStore, KvsEngine, RayonThreadPool, Result};

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    smol::block_on(async {
        store.set("key1".to_owned(), "value1".to_owned()).await?;
        store.set("key2".to_owned(), "value2".to_owned()).await?;

        assert_eq!(
            store.get("key1".to_owned()).await?,
            Some("value1".to_owned())
        );
        assert_eq!(
            store.get("key2".to_owned()).await?,
            Some("value2".to_owned())
        );

        // Open from disk again and check persistent data
        drop(store);
        let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
        assert_eq!(
            store.get("key1".to_owned()).await?,
            Some("value1".to_owned())
        );
        assert_eq!(
            store.get("key2".to_owned()).await?,
            Some("value2".to_owned())
        );

        Ok(())
    })
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    smol::block_on(async {
        store.set("key1".to_owned(), "value1".to_owned()).await?;
        assert_eq!(
            store.get("key1".to_owned()).await?,
            Some("value1".to_owned())
        );
        store.set("key1".to_owned(), "value2".to_owned()).await?;
        assert_eq!(
            store.get("key1".to_owned()).await?,
            Some("value2".to_owned())
        );

        // Open from disk again and check persistent data
        drop(store);
        let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
        assert_eq!(
            store.get("key1".to_owned()).await?,
            Some("value2".to_owned())
        );
        store.set("key1".to_owned(), "value3".to_owned()).await?;
        assert_eq!(
            store.get("key1".to_owned()).await?,
            Some("value3".to_owned())
        );

        Ok(())
    })
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    smol::block_on(async {
        store.set("key1".to_owned(), "value1".to_owned()).await?;
        assert_eq!(store.get("key2".to_owned()).await?, None);

        // Open from disk again and check persistent data
        drop(store);
        let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
        assert_eq!(store.get("key2".to_owned()).await?, None);

        Ok(())
    })
}

#[test]
fn remove_non_existent_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    smol::block_on(async {
        assert!(store.remove("key1".to_owned()).await.is_err());

        Ok(())
    })
}

#[test]
fn remove_key() -> Result<()> {
    smol::block_on(async {
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");
        let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
        store.set("key1".to_owned(), "value1".to_owned()).await?;
        assert!(store.remove("key1".to_owned()).await.is_ok());
        assert_eq!(store.get("key1".to_owned()).await?, None);

        Ok(())
    })
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;

    let dir_size = || {
        let entries = WalkDir::new(temp_dir.path()).into_iter();
        let len: walkdir::Result<u64> = entries
            .map(|res| {
                res.and_then(|entry| entry.metadata())
                    .map(|metadata| metadata.len())
            })
            .sum();
        len.expect("fail to get directory size")
    };

    smol::block_on(async {
        let mut current_size = dir_size();
        for iter in (0..1000).into_iter() {
            for key_id in 0..1000 {
                let key = format!("key{}", key_id);
                let value = format!("{}", iter);
                store.set(key, value).await?;
            }

            let new_size = dir_size();
            if new_size > current_size {
                current_size = new_size;
                continue;
            }
            // Compaction triggered

            drop(store);
            // reopen and check content
            let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
            for key_id in 0..1000 {
                let key = format!("key{}", key_id);
                let val = store.get(key).await?;
                assert_eq!(val, Some(format!("{}", iter)));
            }

            return Result::<()>::Ok(());
        }

        panic!("No compaction detected");
    })
}

#[test]
fn concurrent_set() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8)?;

    let ex = Executor::new();
    (0..10000).into_par_iter().for_each(|i| {
        smol::block_on(
            ex.run(async { store.set(format!("key{}", i), format!("value{}", i)).await }),
        );
    });

    let ex = Executor::new();
    smol::block_on(ex.run(async {
        // We only check concurrent set in this test, so we check sequentially here
        let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 1)?;
        for i in 0..10000 {
            assert_eq!(
                store.get(format!("key{}", i)).await?,
                Some(format!("value{}", i))
            );
        }

        Ok(())
    }))
}

#[test]
fn concurrent_get() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8)?;

    let ex = Executor::new();
    smol::block_on(ex.run(async {
        // We only check concurrent get in this test, so we set sequentially here
        for i in 0..100 {
            store
                .set(format!("key{}", i), format!("value{}", i))
                .await
                .unwrap();
        }
    }));

    let ex = Executor::new();
    (0..100).into_par_iter().for_each(|thread_id| {
        (0..100).into_par_iter().for_each(|i| {
            let store = store.clone();
            let key_id = (i + thread_id) % 100;
            smol::block_on(ex.run(async move {
                assert_eq!(
                    store.get(format!("key{}", key_id)).await?,
                    Some(format!("value{}", key_id))
                );

                Result::<()>::Ok(())
            }));
        });
    });

    // Open from disk again and check persistent data
    let store = KvStore::<RayonThreadPool>::open(temp_dir.path(), 8)?;
    let ex = Executor::new();
    (0..100).into_par_iter().for_each(|thread_id| {
        (0..100).into_par_iter().for_each(|i| {
            let store = store.clone();
            let key_id = (i + thread_id) % 100;
            smol::block_on(ex.run(async move {
                assert_eq!(
                    store.clone().get(format!("key{}", key_id)).await?,
                    Some(format!("value{}", key_id))
                );

                Result::<()>::Ok(())
            }));
        });
    });

    Result::<()>::Ok(())
}
