use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFile as LoftyTaggedFile, TaggedFileExt},
    probe::Probe,
};
use napi::{
    bindgen_prelude::{AsyncTask, Uint8Array},
    Either, Env, Error, Result, Status, Task,
};
use napi_derive::napi;
use tempfile::Builder;

#[path = "helper.rs"]
mod helper;
#[path = "properties.rs"]
mod properties;
#[path = "tag.rs"]
mod tag;

const ERR_INVALID_IN_WASM: &str = "This method is invalid in wasm build";
const ERR_FILE_LOADED_FROM_BUFFER: &str = "This file was loaded from a buffer";

fn path_error(path: &Path, error: impl std::fmt::Display) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("Failed accessing '{}': {}", path.display(), error),
    )
}

/// Resolve an output path to the real file when it already exists. This keeps a
/// symlink directory entry intact while allowing the target file to be replaced
/// atomically. For a new path, canonicalize its parent so the temporary file is
/// created on the same filesystem as the eventual destination.
fn resolve_target_path(path: &Path) -> Result<PathBuf> {
    if path.exists() || fs::symlink_metadata(path).is_ok() {
        return path.canonicalize().map_err(|error| path_error(path, error));
    }

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|error| path_error(parent, error))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| Error::new(Status::InvalidArg, "Destination path has no file name"))?;

    Ok(parent.join(file_name))
}

/// Save a source file to a custom path by writing a temporary copy and then
/// atomically replacing the resolved destination.
fn save_to_custom_path_impl(src_path: &str, dest_path: &str, file: &LoftyTaggedFile) -> Result<()> {
    let target = resolve_target_path(Path::new(dest_path))?;
    let parent = target.parent().ok_or_else(|| {
        Error::new(
            Status::InvalidArg,
            format!("Destination path '{}' has no parent", dest_path),
        )
    })?;

    let suffix = target
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()));
    let mut builder = Builder::new();
    builder.prefix(".music-tag-native-");
    if let Some(suffix) = suffix.as_deref() {
        builder.suffix(suffix);
    }
    let temporary = builder
        .tempfile_in(parent)
        .map_err(|error| path_error(parent, error))?
        .into_temp_path();
    let temporary_path: &Path = temporary.as_ref();

    if let Err(error) = fs::copy(src_path, temporary_path) {
        return Err(Error::new(
            Status::GenericFailure,
            format!(
                "Failed saving '{}' to custom path '{}': {}",
                src_path, dest_path, error
            ),
        ));
    }

    file.save_to_path(temporary_path, WriteOptions::default())
        .map_err(|error| {
            Error::new(
                Status::GenericFailure,
                format!("Failed saving to file '{}': {}", dest_path, error),
            )
        })?;

    temporary.persist(&target).map_err(|error| {
        Error::new(
            Status::GenericFailure,
            format!("Failed replacing file '{}': {}", dest_path, error),
        )
    })?;

    Ok(())
}

pub(crate) enum MusicFileInner {
    Buffer { source_len: usize },
    Path(String),
}

pub enum AsyncLoadSource {
    Path(String),
    Buffer(Vec<u8>),
}

pub struct AsyncLoad {
    source: AsyncLoadSource,
}

fn load_from_path_impl(path: &String) -> Result<MusicFile> {
    let file = Probe::open(path)
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .guess_file_type()
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .read()
        .map_err(|e| Error::new(Status::InvalidArg, e))?;

    Ok(MusicFile {
        file,
        inner: MusicFileInner::Path(path.clone()),
    })
}

fn load_from_buffer_impl(buffer: &[u8]) -> Result<MusicFile> {
    let file = Probe::new(Cursor::new(buffer))
        .guess_file_type()
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .read()
        .map_err(|e| Error::new(Status::InvalidArg, e))?;

    Ok(MusicFile {
        file,
        inner: MusicFileInner::Buffer {
            source_len: buffer.len(),
        },
    })
}

#[napi]
impl Task for AsyncLoad {
    type Output = MusicFile;

    type JsValue = MusicFile;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        match &self.source {
            AsyncLoadSource::Path(path) => load_from_path_impl(path),
            AsyncLoadSource::Buffer(buffer) => load_from_buffer_impl(buffer),
        }
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

pub enum AsyncSaveTarget {
    InPlace(String),
    CustomPath { src_path: String, dest_path: String },
    Buffer(Vec<u8>),
}

pub struct AsyncSave {
    target: AsyncSaveTarget,
    file: LoftyTaggedFile,
}

#[napi]
impl Task for AsyncSave {
    type Output = Option<Vec<u8>>;

    type JsValue = Either<(), Uint8Array>;

    fn compute(&mut self) -> Result<Self::Output> {
        match &mut self.target {
            AsyncSaveTarget::InPlace(path) => {
                self.file
                    .save_to_path(path.as_str(), WriteOptions::default())
                    .map_err(|error| {
                        Error::new(
                            Status::GenericFailure,
                            format!("Failed saving to file '{}': {}", path, error),
                        )
                    })?;
                Ok(None)
            }
            AsyncSaveTarget::CustomPath {
                src_path,
                dest_path,
            } => {
                save_to_custom_path_impl(src_path.as_str(), dest_path.as_str(), &self.file)?;
                Ok(None)
            }
            AsyncSaveTarget::Buffer(buffer) => {
                let mut cursor = Cursor::new(std::mem::take(buffer));

                self.file
                    .save_to(&mut cursor, WriteOptions::default())
                    .map_err(|error| Error::from_reason(error.to_string()))?;

                Ok(Some(cursor.into_inner()))
            }
        }
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(match output {
            Some(buffer) => Either::B(Uint8Array::from(buffer)),
            None => Either::A(()),
        })
    }
}

#[napi]
pub struct MusicFile {
    file: LoftyTaggedFile,
    inner: MusicFileInner,
}

#[cfg(test)]
impl MusicFile {
    pub(crate) fn new_for_test(file: LoftyTaggedFile, inner: MusicFileInner) -> Self {
        Self { file, inner }
    }
}

#[napi]
impl MusicFile {
    /// Load music file from a file path or byte buffer
    ///
    /// @param source The file system path or a Uint8Array containing the audio file data
    ///
    /// @throws If the path doesn't exist or isn't accessible
    /// @throws If the file doesn't contain a valid audio format
    /// @throws If runs in WebAssembly environments (due to file system restrictions).
    #[napi(ts_type = r#"(path: string): Promise<MusicFile>
  static load(buffer: Uint8Array): Promise<MusicFile>"#)]
    pub fn load(source: Either<Uint8Array, String>) -> Result<AsyncTask<AsyncLoad>> {
        let source = match source {
            Either::A(buffer) => AsyncLoadSource::Buffer(buffer.to_vec()),
            Either::B(path) => {
                if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
                    return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
                }

                AsyncLoadSource::Path(path)
            }
        };

        Ok(AsyncTask::new(AsyncLoad { source }))
    }

    /// Load music file from a file path or byte buffer
    ///
    /// This is the synchronous version of {@link load}
    ///
    /// @param source The file system path or a Uint8Array containing the audio file data
    ///
    /// @throws If the path doesn't exist or isn't accessible
    /// @throws If the file doesn't contain a valid audio format
    /// @throws If runs in WebAssembly environments (due to file system restrictions).
    #[napi(ts_type = r#"(path: string): MusicFile
  static loadSync(buffer: Uint8Array): MusicFile"#)]
    pub fn load_sync(source: Either<Uint8Array, String>) -> Result<MusicFile> {
        match source {
            Either::A(buffer) => load_from_buffer_impl(&buffer),
            Either::B(path) => {
                if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
                    return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
                }

                load_from_path_impl(&path)
            }
        }
    }

    /// Current audio file path
    ///
    /// For files loaded from path, this returns the file path.
    /// For files loaded from buffer, this returns `null`.
    #[napi]
    pub fn path(&self) -> Option<&String> {
        match &self.inner {
            MusicFileInner::Buffer { .. } => None,
            MusicFileInner::Path(path) => Some(path),
        }
    }

    /// Save tags into a buffer, returning the new buffer contents.
    fn save_to_new_buffer(&self, mut buf: Vec<u8>) -> Result<Vec<u8>> {
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
    #[napi(ts_type = r#"(path?: string | null): Promise<void>
  save(buffer: Uint8Array): Promise<Uint8Array>"#)]
    pub fn save(
        &self,
        buffer_or_path: Option<Either<Uint8Array, String>>,
    ) -> Result<AsyncTask<AsyncSave>> {
        let target = match buffer_or_path {
            None => match &self.inner {
                MusicFileInner::Buffer { .. } => {
                    Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER))
                }
                MusicFileInner::Path(path) => Ok(AsyncSaveTarget::InPlace(path.clone())),
            },
            Some(buffer_or_path) => match buffer_or_path {
                Either::A(buffer) => Ok(AsyncSaveTarget::Buffer(buffer.to_vec())),
                Either::B(path) => {
                    if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
                        return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
                    }

                    if matches!(&self.inner, MusicFileInner::Buffer { .. }) {
                        return Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER));
                    }

                    let current = match &self.inner {
                        MusicFileInner::Path(current) => current.clone(),
                        MusicFileInner::Buffer { .. } => unreachable!(
                            "buffer-loaded files are rejected before constructing a path target"
                        ),
                    };

                    Ok(AsyncSaveTarget::CustomPath {
                        src_path: current,
                        dest_path: path,
                    })
                }
            },
        }?;

        // Create a snapshot of MusicFile, to send to the background task.
        let file = LoftyTaggedFile::new(
            self.file.file_type(),
            self.file.properties().clone(),
            self.file.tags().to_owned().to_vec(),
        );

        Ok(AsyncTask::new(AsyncSave { target, file }))
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
    #[napi(ts_type = r#"(path?: string | null): void
  saveSync(buffer: Uint8Array): Uint8Array"#)]
    pub fn save_sync(
        &self,
        buffer_or_path: Option<Either<Uint8Array, String>>,
    ) -> Result<Either<(), Uint8Array>> {
        match buffer_or_path {
            None => match &self.inner {
                MusicFileInner::Buffer { .. } => {
                    Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER))
                }
                MusicFileInner::Path(path) => {
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
                    let buf = self.save_to_new_buffer(buffer.to_vec())?;
                    Ok(Either::B(Uint8Array::from(buf)))
                }
                Either::B(path) => {
                    if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
                        return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
                    }

                    if matches!(&self.inner, MusicFileInner::Buffer { .. }) {
                        return Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER));
                    }

                    let src_path = match &self.inner {
                        MusicFileInner::Path(current) => current.as_str(),
                        MusicFileInner::Buffer { .. } => unreachable!(
                            "buffer-loaded files are rejected before constructing a path target"
                        ),
                    };

                    save_to_custom_path_impl(src_path, &path, &self.file)?;
                    Ok(Either::A(()))
                }
            },
        }
    }
}
