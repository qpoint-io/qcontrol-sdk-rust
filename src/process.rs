//! Process identity wrappers for public ABI metadata.
//!
//! These types expose the originating process image and ancestor executable
//! lineage carried through the public plugin ABI.

use std::{ffi::c_char, slice, str};

use crate::ffi;

/// Originating process identity attached to an event or context.
pub struct ProcessIdentity<'a> {
    inner: &'a ffi::qcontrol_process_identity_t,
}

impl<'a> ProcessIdentity<'a> {
    /// Create from a borrowed FFI identity.
    ///
    /// # Safety
    /// The reference must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_ref(identity: &'a ffi::qcontrol_process_identity_t) -> Self {
        Self { inner: identity }
    }

    /// Get the originating process ID.
    pub fn pid(&self) -> i32 {
        pid_to_i32(self.inner.pid)
    }

    /// Get the originating executable path when one is available.
    pub fn exec_path(&self) -> Option<&str> {
        cstr_opt(self.inner.exec_path, self.inner.exec_path_len)
    }

    /// Iterate over ancestor executable lineage from nearest to farthest.
    pub fn lineage(&self) -> impl Iterator<Item = ProcessLineageEntry<'a>> {
        let entries = if self.inner.lineage.is_null() || self.inner.lineage_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.inner.lineage, self.inner.lineage_len) }
        };

        entries.iter().map(ProcessLineageEntry::from_ref)
    }
}

/// One ancestor entry in a process lineage chain.
pub struct ProcessLineageEntry<'a> {
    inner: &'a ffi::qcontrol_process_lineage_entry_t,
}

impl<'a> ProcessLineageEntry<'a> {
    fn from_ref(entry: &'a ffi::qcontrol_process_lineage_entry_t) -> Self {
        Self { inner: entry }
    }

    /// Get the ancestor process ID.
    pub fn pid(&self) -> i32 {
        pid_to_i32(self.inner.pid)
    }

    /// Get the ancestor executable path when one is available.
    pub fn exec_path(&self) -> Option<&str> {
        cstr_opt(self.inner.exec_path, self.inner.exec_path_len)
    }
}

// `pid_t` is 32-bit or 64-bit depending on platform; widen to i128 before the checked i32 conversion and saturate by sign.
fn pid_to_i32(pid: ffi::pid_t) -> i32 {
    let pid = i128::from(pid);
    i32::try_from(pid).unwrap_or(if pid < 0 { i32::MIN } else { i32::MAX })
}

/// Decode a process ABI string from its explicit pointer-plus-length fields.
fn cstr_opt<'a>(ptr: *const c_char, len: usize) -> Option<&'a str> {
    if ptr.is_null() || len == 0 {
        None
    } else {
        let bytes = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len) };
        str::from_utf8(bytes).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::ProcessIdentity;
    use crate::ffi;

    #[test]
    fn exec_path_uses_explicit_length() {
        let exec_path = *b"/bin/qcontrol-extra";
        let identity = ffi::qcontrol_process_identity_t {
            pid: 7,
            exec_path: exec_path.as_ptr().cast(),
            exec_path_len: "/bin/qcontrol".len(),
            lineage: ptr::null(),
            lineage_len: 0,
        };

        let wrapped = unsafe { ProcessIdentity::from_ref(&identity) };
        assert_eq!(wrapped.exec_path(), Some("/bin/qcontrol"));
    }

    #[test]
    fn lineage_entries_uses_explicit_lengths() {
        let lineage_path = *b"/usr/bin/parent-trailer";
        let lineage = [ffi::qcontrol_process_lineage_entry_t {
            pid: 42,
            exec_path: lineage_path.as_ptr().cast(),
            exec_path_len: "/usr/bin/parent".len(),
        }];
        let identity = ffi::qcontrol_process_identity_t {
            pid: 7,
            exec_path: ptr::null(),
            exec_path_len: 0,
            lineage: lineage.as_ptr(),
            lineage_len: lineage.len(),
        };

        let wrapped = unsafe { ProcessIdentity::from_ref(&identity) };
        let lineage: Vec<_> = wrapped.lineage().collect();
        assert_eq!(lineage.len(), 1);
        assert_eq!(lineage[0].pid(), 42);
        assert_eq!(lineage[0].exec_path(), Some("/usr/bin/parent"));
    }
}
