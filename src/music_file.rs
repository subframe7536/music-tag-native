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
        format!("Failed accessing '{}': {}", display_path(path), error),
    )
}

/// Format a path for an error message. `canonicalize` returns verbatim paths on
/// Windows (`\\?\C:\...`, `\\?\UNC\server\share\...`); show them in their usual
/// form instead.
#[cfg(not(target_arch = "wasm32"))]
fn display_path(path: &Path) -> String {
    let text = path.display().to_string();
    if cfg!(windows) {
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{rest}");
        }
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return rest.to_owned();
        }
    }
    text
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

/// Add owner write permission only. `Permissions::set_readonly(false)` would
/// make the file world-writable on Unix.
#[cfg(not(target_arch = "wasm32"))]
fn owner_writable(mut permissions: fs::Permissions) -> fs::Permissions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(permissions.mode() | 0o200);
    }

    #[cfg(not(unix))]
    {
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
    }

    permissions
}

/// Refuse to replace an existing destination the caller cannot write.
///
/// The atomic rename only needs a writable directory, so without this check a
/// read-only file would be replaced on Unix (and fail late on Windows). Opening
/// for writing without truncating checks the real access rights and leaves the
/// file untouched.
#[cfg(not(target_arch = "wasm32"))]
fn ensure_writable(target: &Path, dest_display: &str) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }

    fs::OpenOptions::new()
        .write(true)
        .open(target)
        .map(|_| ())
        .map_err(|error| {
            Error::new(
                Status::GenericFailure,
                format!("Destination file '{dest_display}' is not writable: {error}"),
            )
        })
}

/// Save tags by writing a temporary copy of the source next to the destination
/// and then atomically replacing the resolved destination. Used both for saving
/// in place (`src == dest`) and for saving to a custom path, so an interrupted
/// save never leaves a half-written file behind.
#[cfg(not(target_arch = "wasm32"))]
fn save_to_path_atomic(paths: &SavePaths, file: &LoftyTaggedFile) -> Result<()> {
    let dest_display = paths.dest_display.as_str();
    let save_error = |error: &dyn std::fmt::Display| {
        Error::new(
            Status::GenericFailure,
            format!("Failed saving to file '{dest_display}': {error}"),
        )
    };

    let target = resolve_target_path(&paths.dest)?;
    ensure_writable(&target, dest_display)?;
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

    fs::copy(&paths.src, temporary_path).map_err(|error| {
        Error::new(
            Status::GenericFailure,
            format!(
                "Failed copying '{}' to '{dest_display}': {error}",
                paths.src_display
            ),
        )
    })?;

    // `fs::copy` gives the copy the source's permissions, which may be read-only.
    // Make it writable for the tag write, then apply the final permissions: the
    // destination's own when it already exists, otherwise the source's.
    let source_permissions = fs::metadata(temporary_path)
        .map_err(|error| save_error(&error))?
        .permissions();
    let final_permissions = fs::metadata(&target)
        .map(|metadata| metadata.permissions())
        .unwrap_or_else(|_| source_permissions.clone());
    fs::set_permissions(temporary_path, owner_writable(source_permissions))
        .map_err(|error| save_error(&error))?;

    file.save_to_path(temporary_path, WriteOptions::default())
        .map_err(|error| save_error(&error))?;

    fs::set_permissions(temporary_path, final_permissions).map_err(|error| save_error(&error))?;

    temporary.persist(&target).map_err(|error| {
        Error::new(
            Status::GenericFailure,
            format!("Failed replacing file '{dest_display}': {error}"),
        )
    })?;

    Ok(())
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
    Path(SavePaths),
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
            AsyncSaveTarget::Path(paths) => {
                save_to_path_atomic(paths, &self.file)?;
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
    /// Path saves write a temporary file next to the destination and then replace
    /// the destination atomically, so an interrupted save never leaves a partially
    /// written file.
    ///
    /// @param bufferOrPath Optional output file path (Node.js only) or source buffer. If provided,
    /// saves to this path (or a new buffer that creates from the source buffer with new tags) for this call.
    ///
    /// @throws If the file was loaded from a buffer and no buffer is provided.
    /// @throws If the file was loaded from a buffer and wants to save to a custom path.
    /// @throws If custom path is provided in WebAssembly environments
    /// @throws If the destination file already exists and is read-only
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
            None => AsyncSaveTarget::Path(self.path_save_target(None)?),
            #[cfg(not(target_arch = "wasm32"))]
            Some(Either::B(path)) => AsyncSaveTarget::Path(self.path_save_target(Some(path))?),
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
    /// @throws If the destination file already exists and is read-only
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
                save_to_path_atomic(&self.path_save_target(None)?, &self.file)?;
                Ok(Either::A(()))
            }
            #[cfg(not(target_arch = "wasm32"))]
            Some(Either::B(path)) => {
                save_to_path_atomic(&self.path_save_target(Some(path))?, &self.file)?;
                Ok(Either::A(()))
            }
        }
    }
}
