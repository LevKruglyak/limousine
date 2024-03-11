use marble::Marble;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell, RefMut};
use std::io::ErrorKind;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::rc::Rc;

pub type StoreId = u64;

pub const STORE_ID_NONE: StoreId = 0;
pub const GLOBAL_CATALOG_ID: StoreId = 1;

#[derive(Serialize, Deserialize, Clone)]
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
    local_catalog_id: StoreId,
}

pub struct TypedIndexStore<C> {
    inner: IndexStore,
    catalog: C,
}

impl<C> Deref for TypedIndexStore<C> {
    type Target = IndexStore;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> DerefMut for TypedIndexStore<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C> TypedIndexStore<C>
where
    for<'de> C: Serialize + Deserialize<'de>,
    C: Clone,
{
    pub fn initialize(store: IndexStore) -> crate::Result<Self> {
        let catalog = (store.read_page(store.local_catalog_id)?).ok_or(std::io::Error::new(
            ErrorKind::Other,
            "Failed to deserialize local catalog!",
        ))?;

        Ok(Self {
            inner: store,
            catalog,
        })
    }

    pub fn shutdown(mut self) -> crate::Result<()> {
        let catalog = self.catalog.clone();
        let local_catalog_id = self.local_catalog_id;
        self.write_page(catalog, local_catalog_id)
    }
}

impl IndexStore {
    pub fn load(path: impl AsRef<Path>) -> crate::Result<Self> {
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
            local_catalog_id: GLOBAL_CATALOG_ID,
        })
    }

    pub fn load_new(path: impl AsRef<Path>) -> crate::Result<Self> {
        std::fs::remove_dir_all(path.as_ref())?;

        Self::load(path)
    }

    fn inner_ref(&self) -> Ref<'_, IndexStoreInner> {
        self.inner.as_ref().borrow()
    }

    fn inner_ref_mut(&mut self) -> RefMut<'_, IndexStoreInner> {
        self.inner.as_ref().borrow_mut()
    }

    pub fn write_page<P>(&mut self, page: P, id: StoreId) -> crate::Result<()>
    where
        P: Serialize,
    {
        let data = bincode::serialize(&page)?;
        self.inner_ref_mut()
            .store
            .write_batch([(id, Some(&data))])?;

        Ok(())
    }

    pub fn read_page<P>(&self, id: StoreId) -> crate::Result<Option<P>>
    where
        for<'de> P: Deserialize<'de>,
    {
        if let Some(data) = self.inner_ref().store.read(id)? {
            return Ok(Some(bincode::deserialize(data.as_ref())?));
        }

        Ok(None)
    }

    /// Only should be called on the global store
    pub fn shutdown_global(mut self) -> crate::Result<()> {
        assert_eq!(self.local_catalog_id, GLOBAL_CATALOG_ID);
        let catalog = self.inner_ref().catalog.clone();
        self.write_page(catalog, GLOBAL_CATALOG_ID)
    }

    pub fn new_local_store(&mut self) -> Self {
        // Make sure the allocator doesn't overwrite our local catalog
        {
            let local_catalog_id = self.local_catalog_id;
            let catalog = &mut self.inner_ref_mut().catalog;
            catalog.max_id = catalog.max_id.max(local_catalog_id + 1)
        }

        Self {
            inner: self.inner.clone(),
            local_catalog_id: self.local_catalog_id + 1,
        }
    }
}
