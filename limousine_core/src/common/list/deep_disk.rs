use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::StoreID;

pub struct DeepDiskNode<N, PA> {
    pub inner: N,
    next: StoreID,
    previous: StoreID,
    parent: Option<PA>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct DeepDiskListCatalogPage<PA> {
    // We should only persist parents when we are in a deep persisted layer, in a boundary layer we
    // keep them in transient memory
    parents: HashMap<StoreID, PA>,
}
