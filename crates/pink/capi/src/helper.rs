pub trait ParamType {
    type T;
    type E;
}
impl<T> ParamType for Option<T> {
    type T = T;
    type E = ();
}
impl<T, E> ParamType for Result<T, E> {
    type T = T;
    type E = E;
}
pub type InnerType<T> = <T as ParamType>::T;
pub type InnerError<T> = <T as ParamType>::E;
