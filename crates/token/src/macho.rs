//! Architecture check for macOS drivers: an arm64 process cannot load an
//! x86_64-only dylib, which surfaces later as an opaque load error.

use std::fs::File;
use std::io::Read as _;
use std::path::Path;

/// Mach-O `CPU_TYPE_X86_64`.
const CPU_TYPE_X86_64: u32 = 0x0100_0007;
/// Mach-O `CPU_TYPE_ARM64`.
const CPU_TYPE_ARM64: u32 = 0x0100_000c;
/// Bytes read from the start of the file; enough for any fat header.
const HEADER_BYTES: usize = 4_096;

/// Whether this process can load a driver file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchSupport {
    /// The file contains a slice for this process's architecture.
    Native,
    /// The file is Mach-O but has no slice for this architecture.
    Incompatible,
    /// Not a Mach-O file, or not checked on this platform.
    Unknown,
}

/// Reads the start of `path` and reports whether this process can load it.
pub fn arch_support(path: &Path) -> ArchSupport {
    let mut header = Vec::with_capacity(HEADER_BYTES);
    let read =
        File::open(path).and_then(|file| file.take(HEADER_BYTES as u64).read_to_end(&mut header));
    match (read, native_cpu_type()) {
        (Ok(_), Some(native)) => classify(&cpu_types(&header), native),
        _ => ArchSupport::Unknown,
    }
}

/// Mach-O CPU type of this process, when it is one we check.
fn native_cpu_type() -> Option<u32> {
    match std::env::consts::ARCH {
        "aarch64" => Some(CPU_TYPE_ARM64),
        "x86_64" => Some(CPU_TYPE_X86_64),
        _ => None,
    }
}

/// Maps the slices found onto [`ArchSupport`].
fn classify(types: &[u32], native: u32) -> ArchSupport {
    if types.is_empty() {
        ArchSupport::Unknown
    } else if types.contains(&native) {
        ArchSupport::Native
    } else {
        ArchSupport::Incompatible
    }
}

/// CPU types in a thin or fat Mach-O header; empty when it is neither.
pub(crate) fn cpu_types(header: &[u8]) -> Vec<u32> {
    match read_u32_be(header, 0) {
        Some(0xcafe_babe) => fat_cpu_types(header, 20),
        Some(0xcafe_babf) => fat_cpu_types(header, 32),
        Some(0xcffa_edfe) => read_u32_le(header, 4).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// CPU types listed in a fat header whose entries are `entry_size` bytes.
fn fat_cpu_types(header: &[u8], entry_size: usize) -> Vec<u32> {
    let count = read_u32_be(header, 4).unwrap_or_default();
    (0..count.min(32) as usize)
        .filter_map(|index| read_u32_be(header, 8 + index * entry_size))
        .collect()
}

/// Big-endian u32 at `offset`, if the header is long enough.
fn read_u32_be(header: &[u8], offset: usize) -> Option<u32> {
    let bytes: [u8; 4] = header.get(offset..offset + 4)?.try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
}

/// Little-endian u32 at `offset`, if the header is long enough.
fn read_u32_le(header: &[u8], offset: usize) -> Option<u32> {
    let bytes: [u8; 4] = header.get(offset..offset + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::{ArchSupport, CPU_TYPE_ARM64, CPU_TYPE_X86_64, classify, cpu_types};

    /// A universal header lists both slices.
    #[test]
    fn reads_fat_header() {
        let mut header = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 2];
        header.extend_from_slice(&CPU_TYPE_X86_64.to_be_bytes());
        header.extend_from_slice(&[0; 16]);
        header.extend_from_slice(&CPU_TYPE_ARM64.to_be_bytes());
        header.extend_from_slice(&[0; 16]);
        assert_eq!(cpu_types(&header), [CPU_TYPE_X86_64, CPU_TYPE_ARM64]);
    }

    /// A thin 64-bit header has one little-endian CPU type.
    #[test]
    fn reads_thin_header() {
        let mut header = vec![0xcf, 0xfa, 0xed, 0xfe];
        header.extend_from_slice(&CPU_TYPE_X86_64.to_le_bytes());
        assert_eq!(cpu_types(&header), [CPU_TYPE_X86_64]);
        assert_eq!(
            classify(&cpu_types(&header), CPU_TYPE_ARM64),
            ArchSupport::Incompatible
        );
        assert_eq!(classify(&[], CPU_TYPE_ARM64), ArchSupport::Unknown);
    }
}
