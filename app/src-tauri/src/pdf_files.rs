//! Private staging and atomic publication of a signed copy.

use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

use crate::error::{CommandError, CommandResult};

/// Limits both the IPC response and PDF parsing memory exposure.
pub(crate) const MAX_PDF_BYTES: u64 = 50 * 1024 * 1024;

/// Checks size and the required PDF header before opening either parser.
pub(crate) fn validate_pdf(header: &[u8], length: u64) -> CommandResult<()> {
    if length > MAX_PDF_BYTES || !header.starts_with(b"%PDF-") {
        return Err(CommandError::Message(
            "Choose a PDF smaller than 50 MB.".to_owned(),
        ));
    }
    Ok(())
}

/// Reads at most the size limit even if another process grows the file mid-read.
pub(crate) fn read_pdf(path: &Path) -> CommandResult<Vec<u8>> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_PDF_BYTES + 1).read_to_end(&mut bytes)?;
    validate_pdf(&bytes, u64::try_from(bytes.len()).unwrap_or(u64::MAX))?;
    Ok(bytes)
}

/// A private job directory beside the destination, removed on every normal exit.
pub(crate) struct PdfWorkspace {
    /// Directory this job alone created.
    pub(crate) directory: PathBuf,
}

impl PdfWorkspace {
    /// Creates a fresh directory; never reuses or cleans a pre-existing one.
    pub(crate) fn create(destination: &Path, id: u64) -> CommandResult<Self> {
        if destination.exists() {
            return Err(CommandError::Message(
                "Choose a new filename. Existing files are never replaced.".to_owned(),
            ));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| CommandError::Message("Choose an output folder.".to_owned()))?;
        let directory = parent.join(format!(".swakshar-pdf-{}-{id}", std::process::id()));
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt as _;
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(&directory)?;
        }
        #[cfg(not(unix))]
        fs::create_dir(&directory)?;
        Ok(Self { directory })
    }

    /// Creates private files without following a pre-existing symlink.
    pub(crate) fn write(&self, name: &str, bytes: &[u8]) -> CommandResult<PathBuf> {
        let path = self.directory.join(name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        options.open(&path)?.write_all(bytes)?;
        Ok(path)
    }

    /// Publishes atomically without overwriting a destination created meanwhile.
    pub(crate) fn publish(&self, destination: &Path) -> CommandResult<()> {
        let output = self.directory.join("signed.pdf");
        let file = File::open(&output)?;
        if file.metadata()?.len() == 0 {
            return Err(CommandError::Message(
                "The PDF engine produced no signed document.".to_owned(),
            ));
        }
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        fs::hard_link(output, destination).map_err(|_| {
            CommandError::Message(
                "Could not save the signed copy. Choose a new filename on a local disk.".to_owned(),
            )
        })?;
        Ok(())
    }
}

impl Drop for PdfWorkspace {
    /// Deletes only the fresh directory created by this job.
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.directory) {
            log::warn!("could not remove a PDF staging directory: {}", error.kind());
        }
    }
}
