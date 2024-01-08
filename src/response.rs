use crate::*;

/// Response builder. Use [`Response::new()`] to start a response, then build it,
/// then call [`Response::send()`] on it to fire.
pub struct Response {
    inherit: bool,
    ipc: Option<Vec<u8>>,
    metadata: Option<String>,
    blob: Option<Blob>,
    capabilities: Vec<Capability>,
}

#[allow(dead_code)]
impl Response {
    /// Start building a new response. Attempting to send this response will
    /// not succeed until its `ipc` has been set with `ipc()` or `try_ipc()`.
    pub fn new() -> Self {
        Response {
            inherit: false,
            ipc: None,
            metadata: None,
            blob: None,
            capabilities: vec![],
        }
    }
    /// Set whether this response will "inherit" the blob of the request
    /// that this process most recently received. Unlike with requests, the
    /// inherit field of a response only deals with blob attachment, since
    /// responses don't themselves have to consider responses or contexts.
    ///
    /// *Note that if the blob is set for this response, this flag will not
    /// override it.*
    pub fn inherit(mut self, inherit: bool) -> Self {
        self.inherit = inherit;
        self
    }
    /// Set the IPC (Inter-Process Communication) value for this message. This field
    /// is mandatory. An IPC is simply a vector of bytes. Process developers are
    /// responsible for architecting the serialization/derserialization strategy
    /// for these bytes, but the simplest and most common strategy is just to use
    /// a JSON spec that gets stored in bytes as a UTF-8 string.
    ///
    /// If the serialization strategy is complex, it's best to define it as an impl
    /// of [`TryInto`] on your IPC type, then use `try_ipc()` instead of this.
    pub fn ipc<T>(mut self, ipc: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        self.ipc = Some(ipc.into());
        self
    }
    /// Set the IPC (Inter-Process Communication) value for this message, using a
    /// type that's got an implementation of [`TryInto`] for `Vec<u8>`. It's best
    /// to define an IPC type within your app, then implement TryFrom/TryInto for
    /// all IPC serialization/deserialization.
    pub fn try_ipc<T>(mut self, ipc: T) -> anyhow::Result<Self>
    where
        T: TryInto<Vec<u8>, Error = anyhow::Error>,
    {
        self.ipc = Some(ipc.try_into()?);
        Ok(self)
    }
    /// Set the metdata field for this response. Metadata is simply a [`String`].
    /// Metadata should usually be used for middleware and other message-passing
    /// situations that require the original IPC and blob to be preserved.
    /// As such, metadata should not always be expected to reach the final destination
    /// of this response unless the full chain of behavior is known / controlled by
    /// the developer.
    pub fn metadata(mut self, metadata: &str) -> Self {
        self.metadata = Some(metadata.to_string());
        self
    }
    /// Set the blob of this response. A [`Blob`] holds bytes and an optional
    /// MIME type.
    ///
    /// The purpose of having a blob field distinct from the IPC field is to enable
    /// performance optimizations in all sorts of situations. Blobs are only brought
    /// across the runtime<>WASM boundary if the process calls `get_blob()`, and this
    /// saves lots of work in data-intensive pipelines.
    ///
    /// Blobs also provide a place for less-structured data, such that an IPC type
    /// can be quickly locked in and upgraded within an app-protocol without breaking
    /// changes, while still allowing freedom to adjust the contents and shape of a
    /// blob. IPC formats should be rigorously defined.
    pub fn blob(mut self, blob: Blob) -> Self {
        self.blob = Some(blob);
        self
    }
    /// Set the blob's MIME type. If a blob has not been set, it will be set here
    /// as an empty vector of bytes. If it has been set, the MIME type will be replaced
    /// or created.
    pub fn blob_mime(mut self, mime: &str) -> Self {
        if self.blob.is_none() {
            self.blob = Some(Blob {
                mime: Some(mime.to_string()),
                bytes: vec![],
            });
            self
        } else {
            self.blob = Some(Blob {
                mime: Some(mime.to_string()),
                bytes: self.blob.unwrap().bytes,
            });
            self
        }
    }
    /// Set the blob's bytes. If a blob has not been set, it will be set here with
    /// no MIME type. If it has been set, the bytes will be replaced with these bytes.
    pub fn blob_bytes<T>(mut self, bytes: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        if self.blob.is_none() {
            self.blob = Some(Blob {
                mime: None,
                bytes: bytes.into(),
            });
            self
        } else {
            self.blob = Some(Blob {
                mime: self.blob.unwrap().mime,
                bytes: bytes.into(),
            });
            self
        }
    }
    /// Set the blob's bytes with a type that implements `TryInto<Vec<u8>>`
    /// and may or may not successfully be set.
    pub fn try_blob_bytes<T>(mut self, bytes: T) -> anyhow::Result<Self>
    where
        T: TryInto<Vec<u8>, Error = anyhow::Error>,
    {
        if self.blob.is_none() {
            self.blob = Some(Blob {
                mime: None,
                bytes: bytes.try_into()?,
            });
            Ok(self)
        } else {
            self.blob = Some(Blob {
                mime: self.blob.unwrap().mime,
                bytes: bytes.try_into()?,
            });
            Ok(self)
        }
    }
    /// Add capabilities to this response. Capabilities are a way to pass
    pub fn capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }
    /// Attempt to send the response. This will only fail if the IPC field of
    /// the response has not yet been set using `ipc()` or `try_ipc()`.
    pub fn send(self) -> anyhow::Result<()> {
        if let Some(ipc) = self.ipc {
            crate::send_response(
                &crate::nectar::process::standard::Response {
                    inherit: self.inherit,
                    ipc,
                    metadata: self.metadata,
                    capabilities: self.capabilities,
                },
                self.blob.as_ref(),
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("missing IPC"))
        }
    }
}
