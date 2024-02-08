use super::{inner::MemoInner, ArcMemo};
use crate::{
    owner::{Stored, StoredData},
    signal::MappedSignalReadGuard,
    traits::{DefinedAt, Readable, Track},
};
use std::{fmt::Debug, panic::Location};

pub struct Memo<T: Send + Sync + 'static> {
    #[cfg(debug_assertions)]
    defined_at: &'static Location<'static>,
    inner: Stored<ArcMemo<T>>,
}

impl<T: Send + Sync + 'static> Memo<T> {
    #[track_caller]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all,)
    )]
    pub fn new(fun: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Self
    where
        T: PartialEq,
    {
        Self {
            #[cfg(debug_assertions)]
            defined_at: Location::caller(),
            inner: Stored::new(ArcMemo::new(fun)),
        }
    }
}

impl<T: Send + Sync + 'static> Copy for Memo<T> {}

impl<T: Send + Sync + 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static> Debug for Memo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memo")
            .field("type", &std::any::type_name::<T>())
            .field("store", &self.inner)
            .finish()
    }
}

impl<T: Send + Sync + 'static> StoredData for Memo<T> {
    type Data = ArcMemo<T>;

    fn get_value(&self) -> Option<Self::Data> {
        self.inner.get()
    }

    fn dispose(&self) {
        self.inner.dispose();
    }
}

impl<T: Send + Sync + 'static> DefinedAt for Memo<T> {
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        #[cfg(debug_assertions)]
        {
            Some(self.defined_at)
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}

impl<T: Send + Sync + 'static> Track for Memo<T> {
    fn track(&self) {
        if let Some(inner) = self.get_value() {
            inner.track();
        }
    }
}

impl<T: Send + Sync + 'static> Readable for Memo<T> {
    type Value = MappedSignalReadGuard<MemoInner<T>, T>;

    fn try_read(&self) -> Option<Self::Value> {
        self.get_value().map(|inner| inner.read())
    }
}
