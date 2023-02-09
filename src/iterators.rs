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
    use std::string::ParseError;

    #[test]
    fn test_try_into_elementwise() {
        let v = vec!["1", "2", "3"];
        let result: Result<Vec<&str>, ParseError> = v.into_iter().try_into_elementwise();
        assert_eq!(result.unwrap(), vec!["1", "2", "3"]);
    }

    #[test]
    fn test_into_elementwise() {
        let v = vec!["1", "2", "3"];
        let result: Vec<&str> = v.into_elementwise();
        assert_eq!(result, vec!["1", "2", "3"]);
    }
}
