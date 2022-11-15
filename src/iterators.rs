pub trait TryIntoElementwise<A, B: TryInto<A, Error = E>, E>: IntoIterator {
    /// Performs try_into on each element of the iterator and collects the
    /// results into a Vec.
    fn try_into_elementwise(self) -> Result<Vec<A>, E>;
}

impl<A, B, E, I> TryIntoElementwise<A, B, E> for I
where
    B: TryInto<A, Error = E>,
    I: IntoIterator<Item = B>,
{
    fn try_into_elementwise(self) -> Result<Vec<A>, E> {
        self.into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<_>, E>>()
    }
}
