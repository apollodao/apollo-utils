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

pub trait IntoElementwise<A, B: Into<A>>: IntoIterator {
    /// Performs into on each element of the iterator and collects the
    /// results into a Vec.
    fn into_elementwise(self) -> Vec<A>;
}

impl<A, B, I> IntoElementwise<A, B> for I
where
    B: Into<A>,
    I: IntoIterator<Item = B>,
{
    fn into_elementwise(self) -> Vec<A> {
        self.into_iter().map(|x| x.into()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TryIntoElementwise;
    use crate::iterators::IntoElementwise;
    use std::num::TryFromIntError;
    use test_case::test_case;

    #[test_case(
        vec![1u64, 2u64, 3u64] => Ok(vec![1u32, 2u32, 3u32]);
        "Element wise OK")]
    #[test_case(
        vec![u64::MAX, 2u64, 3u64] => matches Err(_);
            "Element wise Fail")]
    fn test_try_into_elementwise(input: Vec<u64>) -> Result<Vec<u32>, TryFromIntError> {
        // let result: Result<Vec<u32>, ParseError> =
        input.into_iter().try_into_elementwise()
    }

    #[test_case(
        vec![1u32, 2u32, 3u32] => vec![1u64, 2u64, 3u64];
        "Element wise OK")]
    fn test_into_elementwise(input: Vec<u32>) -> Vec<u64> {
        input.into_elementwise()
    }
}
