

#[derive(Debug, Default)]
pub struct S3ObjectInfo {
    pub bucket: String,
    pub name: String,
    pub size: u64,
}