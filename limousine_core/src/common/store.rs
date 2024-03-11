use marble::Marble;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;

pub type StoreId = u64;

pub const STORE_ID_NONE: StoreId = 0;
pub const GLOBAL_CATALOG_ID: StoreId = 1;

#[derive(Serialize, Deserialize)]
pub struct IndexCatalogPage {
    max_id: StoreId,
}

impl Default for IndexCatalogPage {
    fn default() -> Self {
        Self {
            max_id: GLOBAL_CATALOG_ID + 1,
        }
    }
}

pub struct IndexStoreInner {
    store: Marble,
    catalog: IndexCatalogPage,
}

#[derive(Clone)]
pub struct IndexStore {
    inner: Rc<RefCell<IndexStoreInner>>,
    local_catalog: StoreId,
}

impl IndexStore {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let store = marble::open(path.as_ref())?;

        // Load catalog
        let catalog = if let Some(catalog_data) = store.read(GLOBAL_CATALOG_ID)? {
            bincode::deserialize(&catalog_data).expect("Corrupted catalog!")
        } else {
            let catalog = IndexCatalogPage::default();
            let catalog_data = bincode::serialize(&catalog).unwrap();

            store.write_batch([(GLOBAL_CATALOG_ID, Some(&catalog_data))])?;
            catalog
        };

        let inner = IndexStoreInner { store, catalog };

        Ok(Self {
            inner: Rc::new(RefCell::new(inner)),
            local_catalog: GLOBAL_CATALOG_ID,
        })
    }

    pub fn load_new(path: impl AsRef<Path>) -> io::Result<Self> {
        std::fs::remove_dir_all(path.as_ref())?;

        Self::load(path)
    }

    pub fn shutdown(self) -> io::Result<()> {
        // Persist catalog
        let catalog_data = bincode::serialize(&self.inner.borrow().catalog).unwrap();
        self.inner
            .borrow_mut()
            .store
            .write_batch([(GLOBAL_CATALOG_ID, Some(&catalog_data))])?;

        Ok(())
    }

    pub fn new_local_store(&self) -> Self {
        // Make sure the allocator doesn't overwrite our local catalog
        {
            let catalog = &mut self.inner.borrow_mut().catalog;
            catalog.max_id = catalog.max_id.max(self.local_catalog + 1)
        }

        Self {
            inner: self.inner.clone(),
            local_catalog: self.local_catalog + 1,
        }
    }
}
