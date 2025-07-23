use crate::errors::ZapError;
use chrono::{DateTime, TimeDelta, Utc};
use filetime::FileTime;
use std::fs::Metadata;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A specification for file times that can hold both access and modification times.
/// Using Option allows for selective setting of either or both times.
#[derive(Debug, Clone, Copy)]
pub struct FileTimeSpec {
    pub atime: Option<FileTime>,
    pub mtime: Option<FileTime>,
}

impl FileTimeSpec {
    /// Create a new FileTimeSpec with both atime and mtime set to the same value
    pub fn both(time: FileTime) -> Self {
        Self {
            atime: Some(time),
            mtime: Some(time),
        }
    }

    /// Create a new FileTimeSpec with only atime set
    pub fn access_only(time: FileTime) -> Self {
        Self {
            atime: Some(time),
            mtime: None,
        }
    }

    /// Create a new FileTimeSpec with only mtime set
    pub fn modification_only(time: FileTime) -> Self {
        Self {
            atime: None,
            mtime: Some(time),
        }
    }

    /// Create from a DateTime<Utc>, setting both times to the same value
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());
        Self::both(file_time)
    }

    /// Create from current time, setting both times
    pub fn now() -> Self {
        Self::from_datetime(Utc::now())
    }

    /// Create from a reference file's metadata
    pub fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            atime: Some(FileTime::from_last_access_time(metadata)),
            mtime: Some(FileTime::from_last_modification_time(metadata)),
        }
    }

    /// Apply CLI flags to determine which times should be set
    pub fn with_flags(mut self, set_access: bool, set_modification: bool) -> Self {
        if !set_access {
            self.atime = None;
        }
        if !set_modification {
            self.mtime = None;
        }
        self
    }

    /// Check if any time is set
    pub fn has_any_time(&self) -> bool {
        self.atime.is_some() || self.mtime.is_some()
    }

    /// Apply adjustment to both times that are present
    pub fn adjust_by_string(self, adjustment_str: &str) -> Result<Self, ZapError> {
        let adjusted_atime = if let Some(atime) = self.atime {
            Some(
                AdjustableFileTime::from_file_time(atime)
                    .adjust_by_string(adjustment_str)?
                    .into_file_time(),
            )
        } else {
            None
        };

        let adjusted_mtime = if let Some(mtime) = self.mtime {
            Some(
                AdjustableFileTime::from_file_time(mtime)
                    .adjust_by_string(adjustment_str)?
                    .into_file_time(),
            )
        } else {
            None
        };

        Ok(Self {
            atime: adjusted_atime,
            mtime: adjusted_mtime,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdjustableFileTime {
    file_time: FileTime,
}

impl AdjustableFileTime {
    /// Create from an existing FileTime
    pub fn from_file_time(file_time: FileTime) -> Self {
        Self { file_time }
    }

    /// Create from file metadata's access time
    pub fn from_metadata_atime(metadata: &Metadata) -> Self {
        Self {
            file_time: FileTime::from_last_access_time(metadata),
        }
    }

    /// Create from file metadata's modification time
    pub fn from_metadata_mtime(metadata: &Metadata) -> Self {
        Self {
            file_time: FileTime::from_last_modification_time(metadata),
        }
    }

    /// Create from a DateTime<Utc>
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());
        Self { file_time }
    }

    /// Create from the current time
    pub fn now() -> Self {
        Self::from_datetime(Utc::now())
    }

    /// Adjust the time by a number of seconds (positive or negative)
    pub fn adjust_by_seconds(self, seconds: i64) -> Result<Self, ZapError> {
        // Convert FileTime to SystemTime for easier arithmetic
        let system_time = self.to_system_time()?;

        let adjusted_time = if seconds >= 0 {
            system_time
                .checked_add(Duration::from_secs(seconds as u64))
                .ok_or(ZapError::TimeAdjustmentOverflow)?
        } else {
            system_time
                .checked_sub(Duration::from_secs((-seconds) as u64))
                .ok_or(ZapError::TimeAdjustmentUnderflow)?
        };

        Ok(Self {
            file_time: FileTime::from_system_time(adjusted_time),
        })
    }

    /// Adjust the time by a chrono TimeDelta
    pub fn adjust_by_delta(self, delta: TimeDelta) -> Result<Self, ZapError> {
        let seconds = delta.num_seconds();
        self.adjust_by_seconds(seconds)
    }

    /// Adjust the time by parsing an adjustment string (like "3600" for +1 hour or "-30" for -30 seconds)
    pub fn adjust_by_string(self, adjustment_str: &str) -> Result<Self, ZapError> {
        let seconds = crate::parsedate::parse_adjust(adjustment_str)
            .map_err(|e| ZapError::TimeAdjustmentParse(e.to_string()))?;
        self.adjust_by_seconds(seconds as i64)
    }

    /// Convert to FileTime for use with filetime crate functions
    pub fn into_file_time(self) -> FileTime {
        self.file_time
    }

    /// Get the underlying FileTime (by reference)
    pub fn as_file_time(&self) -> &FileTime {
        &self.file_time
    }

    /// Convert to SystemTime for easier arithmetic operations
    fn to_system_time(self) -> Result<SystemTime, ZapError> {
        let duration_since_epoch = Duration::new(
            self.file_time.unix_seconds() as u64,
            self.file_time.nanoseconds(),
        );

        UNIX_EPOCH
            .checked_add(duration_since_epoch)
            .ok_or(ZapError::TimeConversionError)
    }

    /// Convert to DateTime<Utc> for display or further processing
    pub fn to_datetime(&self) -> Result<DateTime<Utc>, ZapError> {
        DateTime::from_timestamp(self.file_time.unix_seconds(), self.file_time.nanoseconds())
            .ok_or(ZapError::TimeConversionError)
    }
}

impl From<FileTime> for AdjustableFileTime {
    fn from(file_time: FileTime) -> Self {
        Self::from_file_time(file_time)
    }
}

impl From<DateTime<Utc>> for AdjustableFileTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::from_datetime(dt)
    }
}

impl From<AdjustableFileTime> for FileTime {
    fn from(adjustable: AdjustableFileTime) -> Self {
        adjustable.file_time
    }
}

/// Convenience function to adjust both access and modification times from metadata
pub fn adjust_file_times_from_metadata(
    metadata: &Metadata,
    adjustment_str: &str,
) -> Result<FileTimeSpec, ZapError> {
    FileTimeSpec::from_metadata(metadata).adjust_by_string(adjustment_str)
}

/// Sets both atime and mtime, handling symlinks appropriately.
/// Uses a single syscall for efficiency when setting both times.
pub fn set_both_times(
    path: &std::path::Path,
    atime: FileTime,
    mtime: FileTime,
    symlink_only: bool,
) -> Result<(), ZapError> {
    if symlink_only {
        filetime::set_symlink_file_times(path, atime, mtime).map_err(ZapError::SetTimesError)
    } else {
        filetime::set_file_times(path, atime, mtime).map_err(ZapError::SetTimesError)
    }
}

/// Sets only the access time, handling symlinks appropriately.
/// For symlinks, we need to preserve the existing mtime.
pub fn set_access_time_only(
    path: &std::path::Path,
    atime: FileTime,
    symlink_only: bool,
) -> Result<(), ZapError> {
    if symlink_only {
        // For symlinks, we need to get the current mtime to preserve it
        let metadata = std::fs::symlink_metadata(path)?;
        let mtime = filetime::FileTime::from_last_modification_time(&metadata);
        filetime::set_symlink_file_times(path, atime, mtime).map_err(ZapError::SetTimesError)
    } else {
        filetime::set_file_atime(path, atime).map_err(ZapError::SetTimesError)
    }
}

/// Sets only the modification time, handling symlinks appropriately.
/// For symlinks, we need to preserve the existing atime.
pub fn set_modification_time_only(
    path: &std::path::Path,
    mtime: FileTime,
    symlink_only: bool,
) -> Result<(), ZapError> {
    if symlink_only {
        // For symlinks, we need to get the current atime to preserve it
        let metadata = std::fs::symlink_metadata(path)?;
        let atime = filetime::FileTime::from_last_access_time(&metadata);
        filetime::set_symlink_file_times(path, atime, mtime).map_err(ZapError::SetTimesError)
    } else {
        filetime::set_file_mtime(path, mtime).map_err(ZapError::SetTimesError)
    }
}

/// Sets file times based on the provided FileTimeSpec and symlink mode.
/// This function handles the logic for different combinations of atime/mtime settings,
/// applying the appropriate filetime functions based on whether we're operating on a symlink or regular file.
pub fn set_times_with_mode(
    path: &std::path::Path,
    times: &FileTimeSpec,
    symlink_only: bool,
) -> Result<(), ZapError> {
    match (times.atime, times.mtime) {
        (Some(atime), Some(mtime)) => set_both_times(path, atime, mtime, symlink_only),
        (Some(atime), None) => set_access_time_only(path, atime, symlink_only),
        (None, Some(mtime)) => set_modification_time_only(path, mtime, symlink_only),
        (None, None) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs::File;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_datetime_conversion() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let adjustable = AdjustableFileTime::from_datetime(dt);
        let converted_back = adjustable.to_datetime().unwrap();

        assert_eq!(dt.timestamp(), converted_back.timestamp());
    }

    #[test]
    fn test_time_adjustment() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let adjustable = AdjustableFileTime::from_datetime(dt);

        let adjusted = adjustable.adjust_by_seconds(3600).unwrap();
        let result_dt = adjusted.to_datetime().unwrap();

        assert_eq!(result_dt.timestamp(), dt.timestamp() + 3600);
    }

    #[test]
    fn test_negative_adjustment() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let adjustable = AdjustableFileTime::from_datetime(dt);

        let adjusted = adjustable.adjust_by_seconds(-1800).unwrap();
        let result_dt = adjusted.to_datetime().unwrap();

        assert_eq!(result_dt.timestamp(), dt.timestamp() - 1800);
    }

    #[test]
    fn test_file_time_spec_both() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());
        let spec = FileTimeSpec::both(file_time);

        assert!(spec.atime.is_some());
        assert!(spec.mtime.is_some());
        assert_eq!(spec.atime.unwrap().unix_seconds(), dt.timestamp());
        assert_eq!(spec.mtime.unwrap().unix_seconds(), dt.timestamp());
    }

    #[test]
    fn test_file_time_spec_access_only() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());
        let spec = FileTimeSpec::access_only(file_time);

        assert!(spec.atime.is_some());
        assert!(spec.mtime.is_none());
        assert_eq!(spec.atime.unwrap().unix_seconds(), dt.timestamp());
    }

    #[test]
    fn test_file_time_spec_modification_only() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());
        let spec = FileTimeSpec::modification_only(file_time);

        assert!(spec.atime.is_none());
        assert!(spec.mtime.is_some());
        assert_eq!(spec.mtime.unwrap().unix_seconds(), dt.timestamp());
    }

    #[test]
    fn test_file_time_spec_from_datetime() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let spec = FileTimeSpec::from_datetime(dt);

        assert!(spec.atime.is_some());
        assert!(spec.mtime.is_some());
        assert_eq!(spec.atime.unwrap().unix_seconds(), dt.timestamp());
        assert_eq!(spec.mtime.unwrap().unix_seconds(), dt.timestamp());
    }

    #[test]
    fn test_file_time_spec_with_flags() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let spec = FileTimeSpec::from_datetime(dt);

        // Test setting only access time
        let access_only = spec.with_flags(true, false);
        assert!(access_only.atime.is_some());
        assert!(access_only.mtime.is_none());

        // Test setting only modification time
        let mtime_only = spec.with_flags(false, true);
        assert!(mtime_only.atime.is_none());
        assert!(mtime_only.mtime.is_some());

        // Test setting both
        let both = spec.with_flags(true, true);
        assert!(both.atime.is_some());
        assert!(both.mtime.is_some());

        // Test setting neither
        let neither = spec.with_flags(false, false);
        assert!(neither.atime.is_none());
        assert!(neither.mtime.is_none());
    }

    #[test]
    fn test_file_time_spec_has_any_time() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());

        let both = FileTimeSpec::both(file_time);
        assert!(both.has_any_time());

        let access_only = FileTimeSpec::access_only(file_time);
        assert!(access_only.has_any_time());

        let mtime_only = FileTimeSpec::modification_only(file_time);
        assert!(mtime_only.has_any_time());

        let neither = FileTimeSpec {
            atime: None,
            mtime: None,
        };
        assert!(!neither.has_any_time());
    }

    #[test]
    fn test_file_time_spec_adjust_by_string() {
        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let spec = FileTimeSpec::from_datetime(dt);

        let adjusted = spec.adjust_by_string("010101").unwrap(); // 01 hour 01 minute 01 second = 3661 seconds
        assert!(adjusted.atime.is_some());
        assert!(adjusted.mtime.is_some());
        assert_eq!(
            adjusted.atime.unwrap().unix_seconds(),
            dt.timestamp() + 3661
        );
        assert_eq!(
            adjusted.mtime.unwrap().unix_seconds(),
            dt.timestamp() + 3661
        );

        // Test with only access time set
        let access_only = FileTimeSpec::access_only(FileTime::from_unix_time(
            dt.timestamp(),
            dt.timestamp_subsec_nanos(),
        ));
        let adjusted_access = access_only.adjust_by_string("-3001").unwrap(); // -30 minutes 01 seconds = -1801 seconds
        assert!(adjusted_access.atime.is_some());
        assert!(adjusted_access.mtime.is_none());
        assert_eq!(
            adjusted_access.atime.unwrap().unix_seconds(),
            dt.timestamp() - 1801
        );
    }

    #[test]
    fn test_set_times_with_mode() {
        // Create a temporary directory for test files
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");

        // Create the test file
        let _ = File::create(&file_path).unwrap();

        // Current time to set
        let dt = Utc::now();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());

        // Test with both times set
        let times = FileTimeSpec::both(file_time);
        assert!(set_times_with_mode(Path::new(&file_path), &times, false).is_ok());

        // Test with only access time set
        let access_only = FileTimeSpec::access_only(file_time);
        assert!(set_times_with_mode(Path::new(&file_path), &access_only, false).is_ok());

        // Test with only modification time set
        let mtime_only = FileTimeSpec::modification_only(file_time);
        assert!(set_times_with_mode(Path::new(&file_path), &mtime_only, false).is_ok());

        // Test with no times set (should do nothing)
        let neither = FileTimeSpec {
            atime: None,
            mtime: None,
        };
        assert!(set_times_with_mode(Path::new(&file_path), &neither, false).is_ok());
    }

    #[test]
    fn test_set_both_times() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_both.txt");
        let _ = File::create(&file_path).unwrap();

        let dt = Utc::now();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());

        // Test with regular file
        assert!(set_both_times(Path::new(&file_path), file_time, file_time, false).is_ok());

        // Verify the times were set
        let metadata = std::fs::metadata(&file_path).unwrap();
        let atime = FileTime::from_last_access_time(&metadata);
        let mtime = FileTime::from_last_modification_time(&metadata);

        // FileTime doesn't implement Eq, so compare the unix seconds
        assert_eq!(atime.unix_seconds(), file_time.unix_seconds());
        assert_eq!(mtime.unix_seconds(), file_time.unix_seconds());
    }

    #[test]
    fn test_set_access_time_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_atime.txt");
        let _ = File::create(&file_path).unwrap();

        // Create a distinct time
        let dt = Utc::now();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());

        // Set only the access time
        assert!(set_access_time_only(Path::new(&file_path), file_time, false).is_ok());

        // Verify that only the access time was changed
        let metadata = std::fs::metadata(&file_path).unwrap();
        let atime = FileTime::from_last_access_time(&metadata);

        assert_eq!(atime.unix_seconds(), file_time.unix_seconds());
    }

    #[test]
    fn test_set_modification_time_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_mtime.txt");
        let _ = File::create(&file_path).unwrap();

        // Create a distinct time
        let dt = Utc::now();
        let file_time = FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());

        // Set only the modification time
        assert!(set_modification_time_only(Path::new(&file_path), file_time, false).is_ok());

        // Verify that only the modification time was changed
        let metadata = std::fs::metadata(&file_path).unwrap();
        let mtime = FileTime::from_last_modification_time(&metadata);

        assert_eq!(mtime.unix_seconds(), file_time.unix_seconds());
    }
}
