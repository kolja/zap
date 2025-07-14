use crate::errors::ZapError;
use chrono::{DateTime, TimeDelta, Utc};
use filetime::FileTime;
use std::fs::Metadata;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        let file_time =
            FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos());
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
    fn to_system_time(&self) -> Result<SystemTime, ZapError> {
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
) -> Result<(FileTime, FileTime), ZapError> {
    let adjusted_atime = AdjustableFileTime::from_metadata_atime(metadata)
        .adjust_by_string(adjustment_str)?
        .into_file_time();

    let adjusted_mtime = AdjustableFileTime::from_metadata_mtime(metadata)
        .adjust_by_string(adjustment_str)?
        .into_file_time();

    Ok((adjusted_atime, adjusted_mtime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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
}

