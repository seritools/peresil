use crate::Progress;

pub trait ProgressExt<P, T, E>: Sized {
    fn context<C, E2>(self, context: C) -> Progress<P, T, E2>
    where
        C: snafu::IntoError<E2, Source = E>,
        E2: std::error::Error + snafu::ErrorCompat;
}

impl<P, T, E> ProgressExt<P, T, E> for Progress<P, T, E> {
    fn context<C, E2>(self, context: C) -> Progress<P, T, E2>
    where
        C: snafu::IntoError<E2, Source = E>,
        E2: std::error::Error + snafu::ErrorCompat,
    {
        self.map_err(|e| context.into_error(e))
    }
}
