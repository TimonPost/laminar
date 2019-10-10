#[derive(Debug)]
pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}
