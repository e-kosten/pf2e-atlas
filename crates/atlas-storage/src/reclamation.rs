//! Optional physical maintenance after a validated reference publication.
use crate::{StorageResult, UpdateOwner};
use std::{io, path::Path, path::PathBuf};

impl UpdateOwner {
    /// Reclaim at least 32 MiB of free SQLite pages during an explicit update.
    /// Returns false when unnecessary or skipped for insufficient/unknown space.
    /// A skipped reclamation leaves normal admission open. Once VACUUM starts,
    /// failures retain the maintenance marker for validated recovery.
    pub fn reclaim_space(&self, schema: u32) -> StorageResult<bool> {
        self.reclaim_space_with_capacity(schema, |path| fs2::available_space(path))
    }

    fn reclaim_space_with_capacity(
        &self,
        schema: u32,
        available: impl Fn(&Path) -> io::Result<u64>,
    ) -> StorageResult<bool> {
        let required = {
            let read = self.workspace.read(schema)?;
            let (free, size): (u64, u64) = read.connection().query_row(
                "SELECT page_size*freelist_count,page_size*page_count FROM pragma_page_size(),pragma_freelist_count(),pragma_page_count()",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            if free < 32 * 1024 * 1024 {
                return Ok(false);
            }
            // SQLite documents up to twice the original database size in extra
            // space for the temporary database and rollback journal/WAL. Check
            // both filesystems conservatively, even when they are the same one.
            size.saturating_mul(2)
        };
        let capacity = (|| -> io::Result<bool> {
            let temporary = vacuum_temp_directory()?;
            for path in [self.workspace.path(), temporary.as_path()] {
                if available(path)? < required {
                    return Ok(false);
                }
            }
            Ok(true)
        })();
        match capacity {
            Ok(true) => {}
            Ok(false) => {
                tracing::warn!(target: "atlas_progress", phase="reclaim_space", required_bytes=required,
                    "Database space was not reclaimed: insufficient temporary disk space.");
                return Ok(false);
            }
            Err(error) => {
                tracing::warn!(target: "atlas_progress", phase="reclaim_space", %error,
                    "Database space was not reclaimed: available disk space could not be checked.");
                return Ok(false);
            }
        }
        // Capacity checks do not reserve space. Keep the existing exclusive
        // admission and durable recovery marker across fallible VACUUM work.
        let session = self.maintenance(schema, "reclaiming database space")?;
        tracing::info!(target:"atlas_progress",phase="reclaim_space","Reclaiming unused database space");
        session.connection().execute_batch(
            "PRAGMA wal_checkpoint(TRUNCATE); VACUUM; PRAGMA wal_checkpoint(TRUNCATE);",
        )?;
        session
            .connection()
            .execute("UPDATE workspace SET maintenance=NULL WHERE id=1", [])?;
        Ok(true)
    }
}

fn vacuum_temp_directory() -> io::Result<PathBuf> {
    #[cfg(unix)]
    {
        // Match bundled SQLite's Unix temporary-directory preference. Probe
        // writability without changing process-global SQLite/environment state.
        let paths = ["SQLITE_TMPDIR", "TMPDIR"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(PathBuf::from)
            .chain(["/var/tmp", "/usr/tmp", "/tmp", "."].map(PathBuf::from));
        for path in paths {
            match tempfile::NamedTempFile::new_in(&path) {
                Ok(_) => return Ok(path),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::PermissionDenied
                            | io::ErrorKind::NotFound
                            | io::ErrorKind::NotADirectory
                    ) => {}
                // In particular, a full preferred volume must cause a skip,
                // not make the preflight assume SQLite will use another one.
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::other("no writable SQLite temporary directory"))
    }
    #[cfg(not(unix))]
    {
        Ok(std::env::temp_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_failure_skips_vacuum_without_closing_admission() {
        let (_directory, workspace) = crate::tests::fixture();
        let owner = workspace.update_owner().unwrap();
        assert!(
            !owner
                .reclaim_space_with_capacity(1, |_| panic!(
                    "small databases do not need capacity probes"
                ))
                .unwrap()
        );
        let write = workspace.write(1).unwrap();
        write.connection().execute_batch("CREATE TABLE obsolete(value BLOB); INSERT INTO obsolete VALUES(zeroblob(40*1024*1024)); DROP TABLE obsolete;").unwrap();
        write.commit().unwrap();
        let read = workspace.read(1).unwrap();
        let required: u64 = read
            .connection()
            .query_row(
                "SELECT page_size*page_count*2 FROM pragma_page_size(),pragma_page_count()",
                [],
                |r| r.get(0),
            )
            .unwrap();
        drop(read);
        for failure in [0, 1, 2] {
            let checks = std::cell::Cell::new(0);
            assert!(
                !owner
                    .reclaim_space_with_capacity(1, |_| {
                        let i = checks.get();
                        checks.set(i + 1);
                        match failure {
                            0 => Ok(required - 1),
                            1 if i == 0 => Ok(u64::MAX),
                            1 => Ok(required - 1),
                            _ => Err(io::Error::other("capacity unavailable")),
                        }
                    })
                    .unwrap()
            );
            assert_eq!(checks.get(), if failure == 1 { 2 } else { 1 });
            let read = workspace.read(1).unwrap();
            assert_eq!(read.reference_revision().unwrap(), 0);
            assert_eq!(
                read.connection()
                    .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
                    .unwrap(),
                "retained"
            );
            let free: u64 = read.connection().query_row("SELECT page_size*freelist_count FROM pragma_page_size(),pragma_freelist_count()", [], |r| r.get(0)).unwrap();
            assert!(free >= 32 * 1024 * 1024);
        }
        assert!(
            owner
                .reclaim_space_with_capacity(1, |_| Ok(required))
                .unwrap()
        );
    }
}
