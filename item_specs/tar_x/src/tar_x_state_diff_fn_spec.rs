use std::cmp::Ordering;

use crate::{FileMetadata, FileMetadatas, TarXError, TarXStateDiff};

/// Tar extraction status diff function.
#[derive(Debug)]
pub struct TarXStateDiffFnSpec;

impl TarXStateDiffFnSpec {
    pub async fn state_diff(
        file_metadatas_current: &FileMetadatas,
        file_metadatas_desired: &FileMetadatas,
    ) -> Result<TarXStateDiff, TarXError> {
        let mut current_metadata_iter = file_metadatas_current.iter();
        let mut desired_metadata_iter = file_metadatas_desired.iter();

        let mut added = Vec::<FileMetadata>::new();
        let mut modified = Vec::<FileMetadata>::new();
        let mut removed = Vec::<FileMetadata>::new();

        let mut current_metadata_opt = current_metadata_iter.next();
        let mut desired_metadata_opt = desired_metadata_iter.next();
        loop {
            match (current_metadata_opt, desired_metadata_opt) {
                (Some(current_metadata), Some(desired_metadata)) => {
                    match current_metadata.path().cmp(desired_metadata.path()) {
                        Ordering::Less => {
                            // extracted file name is smaller than file name in tar
                            // meaning extracted file has been removed.
                            removed.push(current_metadata.clone());

                            current_metadata_opt = current_metadata_iter.next();
                            continue;
                        }
                        Ordering::Equal => {
                            match current_metadata
                                .modified_time()
                                .cmp(&desired_metadata.modified_time())
                            {
                                Ordering::Less | Ordering::Greater => {
                                    // Should we not overwrite if destination file is greater?
                                    modified.push(desired_metadata.clone());

                                    current_metadata_opt = current_metadata_iter.next();
                                    desired_metadata_opt = desired_metadata_iter.next();
                                }
                                Ordering::Equal => {
                                    // don't include in the diff, it's in sync
                                    current_metadata_opt = current_metadata_iter.next();
                                    desired_metadata_opt = desired_metadata_iter.next();
                                }
                            }
                        }
                        Ordering::Greater => {
                            // extracted file name is greater than file name in tar
                            // meaning tar file is newly added.
                            added.push(desired_metadata.clone());

                            desired_metadata_opt = desired_metadata_iter.next();
                            continue;
                        }
                    }
                }
                (Some(current_metadata), None) => {
                    removed.push(current_metadata.clone());
                    removed.extend(current_metadata_iter.cloned());
                    break;
                }
                (None, Some(desired_metadata)) => {
                    added.push(desired_metadata.clone());
                    added.extend(desired_metadata_iter.cloned());
                    break;
                }
                (None, None) => break,
            }
        }

        if added.is_empty() && modified.is_empty() && removed.is_empty() {
            Ok(TarXStateDiff::ExtractionInSync)
        } else {
            let added = FileMetadatas::from(added);
            let modified = FileMetadatas::from(modified);
            let removed = FileMetadatas::from(removed);

            Ok(TarXStateDiff::ExtractionOutOfSync {
                added,
                modified,
                removed,
            })
        }
    }
}
