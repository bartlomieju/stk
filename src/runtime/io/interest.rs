use crate::runtime::io::Ready;

use std::{fmt, ops};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Interest(mio::Interest);

impl Interest {
    /// Interest in all readable events.
    ///
    /// Readable interest includes read-closed events.
    pub const READABLE: Interest = Interest(mio::Interest::READABLE);

    /// Interest in all writable events.
    ///
    /// Writable interest includes write-closed events.
    pub const WRITABLE: Interest = Interest(mio::Interest::WRITABLE);

    /// Returns true if the value includes readable interest.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::io::Interest;
    ///
    /// assert!(Interest::READABLE.is_readable());
    /// assert!(!Interest::WRITABLE.is_readable());
    ///
    /// let both = Interest::READABLE | Interest::WRITABLE;
    /// assert!(both.is_readable());
    /// ```
    pub const fn is_readable(self) -> bool {
        self.0.is_readable()
    }

    /// Returns true if the value includes writable interest.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::io::Interest;
    ///
    /// assert!(!Interest::READABLE.is_writable());
    /// assert!(Interest::WRITABLE.is_writable());
    ///
    /// let both = Interest::READABLE | Interest::WRITABLE;
    /// assert!(both.is_writable());
    /// ```
    pub const fn is_writable(self) -> bool {
        self.0.is_writable()
    }

    /// Add together two `Interest` values.
    ///
    /// This function works from a `const` context.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::io::Interest;
    ///
    /// const BOTH: Interest = Interest::READABLE.add(Interest::WRITABLE);
    ///
    /// assert!(BOTH.is_readable());
    /// assert!(BOTH.is_writable());
    pub const fn add(self, other: Interest) -> Interest {
        Interest(self.0.add(other.0))
    }

    // This function must be crate-private to avoid exposing a `mio` dependency.
    pub(crate) const fn to_mio(self) -> mio::Interest {
        self.0
    }

    pub(super) fn mask(self) -> Ready {
        match self {
            Interest::READABLE => Ready::READABLE | Ready::READ_CLOSED,
            Interest::WRITABLE => Ready::WRITABLE | Ready::WRITE_CLOSED,
            _ => Ready::EMPTY,
        }
    }
}

impl ops::BitOr for Interest {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        self.add(other)
    }
}

impl ops::BitOrAssign for Interest {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.0 = (*self | other).0;
    }
}

impl fmt::Debug for Interest {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(fmt)
    }
}
