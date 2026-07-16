use crate::entities::Library;
use crate::PgPool;
use domus_common::{Error, Result};
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct LibraryRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl LibraryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, _owner_id: Option<Uuid>) -> Result<Vec<Library>> {
        Err(Error::NotImplemented("LibraryRepository::list"))
    }

    pub async fn create(
        &self,
        _owner_id: Uuid,
        _name: &str,
        _import_paths: &[String],
    ) -> Result<Library> {
        Err(Error::NotImplemented("LibraryRepository::create"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("LibraryRepository::delete"))
    }
}
