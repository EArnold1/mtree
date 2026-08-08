#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    Directory,
    File,
}

#[derive(Debug, Clone, Copy)]
pub struct Payload<'a> {
    payload_type: PayloadType,
    file_name: &'a str,
    content_hash: &'a [u8],
}

impl<'a> Payload<'a> {
    pub fn new(payload_type: PayloadType, file_name: &'a str, content_hash: &'a [u8]) -> Self {
        Self {
            payload_type,
            file_name,
            content_hash,
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let payload_type = match self.payload_type {
            PayloadType::Directory => 0u8,
            PayloadType::File => 1u8,
        };

        let name_bytes = self.file_name.as_bytes();
        let name_size = u8::try_from(name_bytes.len())
            .expect("payload format supports file names up to 255 bytes"); // TODO: Consider using a larger size for file names if needed.

        let content_hash = self.content_hash;

        let mut payload = Vec::with_capacity(2 + name_bytes.len() + content_hash.len());
        payload.push(payload_type);
        payload.push(name_size);
        payload.extend_from_slice(name_bytes);
        payload.extend_from_slice(content_hash);
        payload
    }
}
