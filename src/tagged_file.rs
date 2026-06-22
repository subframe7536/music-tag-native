use std::{io::Cursor, path::Path};

use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFile as LoftyTaggedFile, TaggedFileExt},
    probe::Probe,
};
use napi::{
    bindgen_prelude::{AsyncTask, PromiseRaw, Uint8Array},
    Either, Env, Error, Result, Status, Task,
};
use napi_derive::napi;

mod helper;
mod properties;
mod tag;

const ERR_INVALID_IN_WASM: &str = "This method is invalid in wasm build";
const ERR_FILE_LOADED_FROM_BUFFER: &str = "This file was loaded from a buffer";

/// Core logic for saving tags to a custom path: copies the source file if needed, then writes tags.
fn save_to_custom_path_impl(
    src_path: Option<&str>,
    dest_path: &str,
    file: &LoftyTaggedFile,
) -> Result<()> {
    let target = Path::new(dest_path);
    if let Some(src) = src_path {
        if target != Path::new(src) {
            std::fs::copy(src, target).map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!(
                        "Failed saving '{}' to custom path '{}': {}",
                        src, dest_path, e
                    ),
                )
            })?;
        }
    }
    file.save_to_path(target, WriteOptions::default())
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed saving to file '{}': {}", dest_path, e),
            )
        })
}

pub(crate) enum TaggedFileInner {
    Buffer { source_len: usize },
    Path(String),
}

pub struct AsyncLoadPath {
    path: String,
}

fn load_from_path_impl(path: &String) -> Result<TaggedFile> {
    let file = Probe::open(path)
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .guess_file_type()
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .read()
        .map_err(|e| Error::new(Status::InvalidArg, e))?;

    Ok(TaggedFile {
        file,
        inner: TaggedFileInner::Path(path.clone()),
    })
}

#[napi]
impl Task for AsyncLoadPath {
    type Output = TaggedFile;

    type JsValue = TaggedFile;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        load_from_path_impl(&self.path)
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

pub struct AsyncSavePath {
    src_path: Option<String>,
    dest_path: String,
    file: LoftyTaggedFile,
}

#[napi]
impl Task for AsyncSavePath {
    type Output = ();

    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        save_to_custom_path_impl(self.src_path.as_deref(), &self.dest_path, &self.file)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
pub struct TaggedFile {
    file: LoftyTaggedFile,
    inner: TaggedFileInner,
}

#[cfg(test)]
impl TaggedFile {
    pub(crate) fn new_for_test(file: LoftyTaggedFile, inner: TaggedFileInner) -> Self {
        Self { file, inner }
    }
}

#[napi]
impl TaggedFile {
    /// Load music file from a file path
    ///
    /// @param path The file system path to the audio file
    ///
    /// @throws If the path doesn't exist or isn't accessible
    /// @throws If the file doesn't contain a valid audio format
    /// @throws If runs in WebAssembly environments (due to file system restrictions).
    #[napi]
    pub fn load_from_path(path: String) -> Result<AsyncTask<AsyncLoadPath>> {
        if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
            return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
        }

        Ok(AsyncTask::new(AsyncLoadPath { path }))
    }

    /// Load music file from a file path
    ///
    /// This is the synchronous version of {@link loadFromPath}
    ///
    /// @param path The file system path to the audio file
    ///
    /// @throws If the path doesn't exist or isn't accessible
    /// @throws If the file doesn't contain a valid audio format
    /// @throws If runs in WebAssembly environments (due to file system restrictions).
    #[napi]
    pub fn load_from_path_sync(path: String) -> Result<TaggedFile> {
        if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
            return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
        }

        load_from_path_impl(&path)
    }

    /// Load music file from a byte buffer, the file buffer won't be stored in
    /// the TaggedFile instance.
    ///
    /// @param buffer A Uint8Array containing the audio file data
    ///
    /// @throws If the buffer doesn't contain a valid audio file
    #[napi]
    pub fn load_from_buffer(buffer: Uint8Array) -> Result<TaggedFile> {
        let file = Probe::new(Cursor::new(&buffer))
            .guess_file_type()
            .map_err(|e| Error::new(Status::InvalidArg, e))?
            .read()
            .map_err(|e| Error::new(Status::InvalidArg, e))?;

        Ok(TaggedFile {
            file,
            inner: TaggedFileInner::Buffer {
                source_len: buffer.len(),
            },
        })
    }

    /// Current audio file path
    ///
    /// For files loaded via `loadFromPath()`, this returns the file path.
    /// For files loaded via `loadFromBuffer()`, this returns `null`.
    #[napi]
    pub fn path(&self) -> Option<&String> {
        match &self.inner {
            TaggedFileInner::Buffer { .. } => None,
            TaggedFileInner::Path(path) => Some(path),
        }
    }

    /// Save tags into a buffer, returning the new buffer contents.
    fn save_to_new_buffer(&self, source_buffer: &Uint8Array) -> Result<Vec<u8>> {
        let mut buf = source_buffer.to_vec();
        let mut cursor = Cursor::new(&mut buf);
        self.file
            .save_to(&mut cursor, WriteOptions::default())
            .map_err(|x| Error::from_reason(x.to_string()))?;
        Ok(buf)
    }

    /// Save metadata changes to the provided buffer, existing path, or a custom path
    ///
    /// @param bufferOrPath Optional output file path (Node.js only) or source buffer. If provided,
    /// saves to this path (or a new buffer that creates from the source buffer with new tags) for this call.
    ///
    /// @throws If the file was loaded from a buffer and no buffer is provided.
    /// @throws If the file was loaded from a buffer and wants to save to a custom path.
    /// @throws If custom path is provided in WebAssembly environments
    /// @throws If saving fails due to file format constraints
    #[napi(ts_return_type = "Promise<Uint8Array | void>")]
    #[allow(clippy::type_complexity)]
    pub fn save(
        &self,
        env: &Env,
        buffer_or_path: Option<Either<Uint8Array, String>>,
    ) -> Result<Either<PromiseRaw<'_, Either<(), Uint8Array>>, AsyncTask<AsyncSavePath>>> {
        match buffer_or_path {
            None => {
                match &self.inner {
                    TaggedFileInner::Buffer { .. } => {
                        Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER))
                    }
                    TaggedFileInner::Path(path) => {
                        // Create a snapshot of TaggedFile, to send to the task
                        let file = LoftyTaggedFile::new(
                            self.file.file_type(),
                            self.file.properties().clone(),
                            self.file.tags().to_owned().to_vec(),
                        );
                        Ok(Either::B(AsyncTask::new(AsyncSavePath {
                            src_path: None,
                            dest_path: path.clone(),
                            file,
                        })))
                    }
                }
            }
            Some(buffer_or_path) => match buffer_or_path {
                Either::A(buffer) => {
                    let buf = self.save_to_new_buffer(&buffer)?;
                    Ok(Either::A(PromiseRaw::resolve(
                        env,
                        Either::B(Uint8Array::from(buf)),
                    )?))
                }
                Either::B(path) => {
                    if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
                        return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
                    }

                    if matches!(&self.inner, TaggedFileInner::Buffer { .. }) {
                        return Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER));
                    }

                    let src_path = match &self.inner {
                        TaggedFileInner::Path(current)
                            if Path::new(current) != Path::new(&path) =>
                        {
                            Some(current.clone())
                        }
                        _ => None,
                    };

                    // Create a snapshot of TaggedFile, to send to the task
                    let file = LoftyTaggedFile::new(
                        self.file.file_type(),
                        self.file.properties().clone(),
                        self.file.tags().to_owned().to_vec(),
                    );

                    Ok(Either::B(AsyncTask::new(AsyncSavePath {
                        src_path,
                        dest_path: path,
                        file,
                    })))
                }
            },
        }
    }

    /// Save metadata changes to the provided buffer, existing path, or a custom path
    ///
    /// This is the synchronous version of {@link save}
    ///
    /// @param bufferOrPath Optional output file path (Node.js only) or source buffer. If provided,
    /// saves to this path (or a new buffer that creates from the source buffer with new tags) for this call.
    ///
    /// @throws If the file was loaded from a buffer and no buffer is provided.
    /// @throws If the file was loaded from a buffer and wants to save to a custom path.
    /// @throws If custom path is provided in WebAssembly environments
    /// @throws If saving fails due to file format constraints
    #[napi]
    pub fn save_sync(
        &self,
        buffer_or_path: Option<Either<Uint8Array, String>>,
    ) -> Result<Either<(), Uint8Array>> {
        match buffer_or_path {
            None => match &self.inner {
                TaggedFileInner::Buffer { .. } => {
                    Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER))
                }
                TaggedFileInner::Path(path) => {
                    self.file
                        .save_to_path(path, WriteOptions::default())
                        .map_err(|e| {
                            Error::new(
                                Status::GenericFailure,
                                format!("Failed saving to file '{}': {}", path, e),
                            )
                        })?;
                    Ok(Either::A(()))
                }
            },
            Some(buffer_or_path) => match buffer_or_path {
                Either::A(buffer) => {
                    let buf = self.save_to_new_buffer(&buffer)?;
                    Ok(Either::B(Uint8Array::from(buf)))
                }
                Either::B(path) => {
                    if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
                        return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
                    }

                    if matches!(&self.inner, TaggedFileInner::Buffer { .. }) {
                        return Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER));
                    }

                    let src_path = match &self.inner {
                        TaggedFileInner::Path(current) => Some(current.as_str()),
                        TaggedFileInner::Buffer { .. } => None,
                    };

                    save_to_custom_path_impl(src_path, &path, &self.file)?;
                    Ok(Either::A(()))
                }
            },
        }
    }
}
