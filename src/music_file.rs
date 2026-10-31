use std::io::Cursor;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs,
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
#[cfg(not(target_arch = "wasm32"))]
use tempfile::Builder;

#[cfg(target_arch = "wasm32")]
const ERR_INVALID_IN_WASM: &str = "This method is invalid in wasm build";
const ERR_FILE_LOADED_FROM_BUFFER: &str = "This file was loaded from a buffer";

#[cfg(not(target_arch = "wasm32"))]
fn path_error(path: &Path, error: impl std::fmt::Display) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("Failed accessing '{}': {}", path.display(), error),
    )
}

/// Source and destination of a path-based save
#[cfg(not(target_arch = "wasm32"))]
pub struct SavePaths {
    /// Absolute path of the loaded file
    src: PathBuf,
    /// Absolute destination path, equal to `src` for an in-place save
    dest: PathBuf,
    /// Source path as passed to `load`, for error messages
    src_display: String,
    /// Destination path as passed by the caller, for error messages
    dest_display: String,
}

/// Resolve an output path to the real file when it already exists. This keeps a
/// symlink directory entry intact while allowing the target file to be replaced
/// atomically. For a new path, canonicalize its parent so the temporary file is
/// created on the same filesystem as the eventual destination.
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
fn save_to_custom_path_impl(paths: &SavePaths, file: &LoftyTaggedFile) -> Result<()> {
    let dest_display = paths.dest_display.as_str();
    let target = resolve_target_path(&paths.dest)?;
    let parent = target.parent().ok_or_else(|| {
        Error::new(
            Status::InvalidArg,
            format!("Destination path '{dest_display}' has no parent"),
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

    if let Err(error) = fs::copy(&paths.src, temporary_path) {
        return Err(Error::new(
            Status::GenericFailure,
            format!(
                "Failed saving '{}' to custom path '{dest_display}': {error}",
                paths.src_display
            ),
        ));
    }

    file.save_to_path(temporary_path, WriteOptions::default())
        .map_err(|error| {
            Error::new(
                Status::GenericFailure,
                format!("Failed saving to file '{dest_display}': {error}"),
            )
        })?;

    temporary.persist(&target).map_err(|error| {
        Error::new(
            Status::GenericFailure,
            format!("Failed replacing file '{dest_display}': {error}"),
        )
    })?;

    Ok(())
}

/// Write the tags back into the loaded file
#[cfg(not(target_arch = "wasm32"))]
fn save_in_place_impl(paths: &SavePaths, file: &LoftyTaggedFile) -> Result<()> {
    file.save_to_path(&paths.dest, WriteOptions::default())
        .map_err(|error| {
            Error::new(
                Status::GenericFailure,
                format!("Failed saving to file '{}': {}", paths.dest_display, error),
            )
        })
}

/// Write the tags into a copy of the source buffer and return it
fn save_to_buffer(file: &LoftyTaggedFile, buffer: Vec<u8>) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(buffer);
    file.save_to(&mut cursor, WriteOptions::default())
        .map_err(|error| {
            Error::new(
                Status::GenericFailure,
                format!("Failed saving to buffer: {}", error),
            )
        })?;
    Ok(cursor.into_inner())
}

pub(crate) enum MusicFileInner {
    Buffer {
        source_len: usize,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Path {
        /// Path as passed by the caller, returned by `MusicFile.path`
        path: String,
        /// Absolute, canonical path resolved at load time. Saves use this, so a
        /// later `process.chdir()` cannot redirect them.
        resolved: PathBuf,
        /// File size at load time, used to estimate the bit rate
        file_size: u64,
    },
}

pub enum AsyncLoadSource {
    #[cfg(not(target_arch = "wasm32"))]
    Path(String),
    Buffer(Vec<u8>),
}

pub struct AsyncLoad {
    source: AsyncLoadSource,
}

#[cfg(not(target_arch = "wasm32"))]
fn load_from_path_impl(path: &str) -> Result<MusicFile> {
    let file = Probe::open(path)
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .guess_file_type()
        .map_err(|e| Error::new(Status::InvalidArg, e))?
        .read()
        .map_err(|e| Error::new(Status::InvalidArg, e))?;

    let resolved = fs::canonicalize(path).map_err(|error| path_error(Path::new(path), error))?;
    let file_size = fs::metadata(&resolved)
        .map_err(|error| path_error(&resolved, error))?
        .len();

    Ok(MusicFile {
        file,
        inner: MusicFileInner::Path {
            path: path.to_owned(),
            resolved,
            file_size,
        },
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
            #[cfg(not(target_arch = "wasm32"))]
            AsyncLoadSource::Path(path) => load_from_path_impl(path),
            AsyncLoadSource::Buffer(buffer) => load_from_buffer_impl(buffer),
        }
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

pub enum AsyncSaveTarget {
    #[cfg(not(target_arch = "wasm32"))]
    InPlace(SavePaths),
    #[cfg(not(target_arch = "wasm32"))]
    CustomPath(SavePaths),
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
            #[cfg(not(target_arch = "wasm32"))]
            AsyncSaveTarget::InPlace(paths) => {
                save_in_place_impl(paths, &self.file)?;
                Ok(None)
            }
            #[cfg(not(target_arch = "wasm32"))]
            AsyncSaveTarget::CustomPath(paths) => {
                save_to_custom_path_impl(paths, &self.file)?;
                Ok(None)
            }
            AsyncSaveTarget::Buffer(buffer) => {
                save_to_buffer(&self.file, std::mem::take(buffer)).map(Some)
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
    pub(crate) file: LoftyTaggedFile,
    pub(crate) inner: MusicFileInner,
}

#[cfg(test)]
impl MusicFile {
    pub(crate) fn new_for_test(file: LoftyTaggedFile, inner: MusicFileInner) -> Self {
        Self { file, inner }
    }
}

impl MusicFile {
    /// Resolve the source and destination paths for a path-based save.
    ///
    /// `dest_path` is `None` for an in-place save. A custom destination is made
    /// absolute here, on the calling thread, so it is relative to the working
    /// directory at call time.
    #[cfg(not(target_arch = "wasm32"))]
    fn path_save_target(&self, dest_path: Option<String>) -> Result<SavePaths> {
        let (src, src_display) = match &self.inner {
            MusicFileInner::Buffer { .. } => {
                return Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER));
            }
            MusicFileInner::Path { path, resolved, .. } => (resolved.clone(), path.clone()),
        };

        let (dest, dest_display) = match dest_path {
            None => (src.clone(), src_display.clone()),
            Some(dest_path) => (
                std::path::absolute(&dest_path)
                    .map_err(|error| path_error(Path::new(&dest_path), error))?,
                dest_path,
            ),
        };

        Ok(SavePaths {
            src,
            dest,
            src_display,
            dest_display,
        })
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
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = path;
                    return Err(Error::new(Status::InvalidArg, ERR_INVALID_IN_WASM));
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    AsyncLoadSource::Path(path)
                }
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
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = path;
                    return Err(Error::new(Status::InvalidArg, ERR_INVALID_IN_WASM));
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    load_from_path_impl(&path)
                }
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
            #[cfg(not(target_arch = "wasm32"))]
            MusicFileInner::Path { path, .. } => Some(path),
        }
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
            Some(Either::A(buffer)) => AsyncSaveTarget::Buffer(buffer.to_vec()),
            #[cfg(target_arch = "wasm32")]
            None => return Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER)),
            #[cfg(target_arch = "wasm32")]
            Some(Either::B(_)) => return Err(Error::new(Status::InvalidArg, ERR_INVALID_IN_WASM)),
            #[cfg(not(target_arch = "wasm32"))]
            None => AsyncSaveTarget::InPlace(self.path_save_target(None)?),
            #[cfg(not(target_arch = "wasm32"))]
            Some(Either::B(path)) => {
                AsyncSaveTarget::CustomPath(self.path_save_target(Some(path))?)
            }
        };

        // Create a snapshot of MusicFile, to send to the background task.
        let file = LoftyTaggedFile::new(
            self.file.file_type(),
            self.file.properties().clone(),
            self.file.tags().to_vec(),
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
            Some(Either::A(buffer)) => {
                let buffer = save_to_buffer(&self.file, buffer.to_vec())?;
                Ok(Either::B(Uint8Array::from(buffer)))
            }
            #[cfg(target_arch = "wasm32")]
            None => Err(Error::new(Status::InvalidArg, ERR_FILE_LOADED_FROM_BUFFER)),
            #[cfg(target_arch = "wasm32")]
            Some(Either::B(_)) => Err(Error::new(Status::InvalidArg, ERR_INVALID_IN_WASM)),
            #[cfg(not(target_arch = "wasm32"))]
            None => {
                save_in_place_impl(&self.path_save_target(None)?, &self.file)?;
                Ok(Either::A(()))
            }
            #[cfg(not(target_arch = "wasm32"))]
            Some(Either::B(path)) => {
                save_to_custom_path_impl(&self.path_save_target(Some(path))?, &self.file)?;
                Ok(Either::A(()))
            }
        }
    }
}
