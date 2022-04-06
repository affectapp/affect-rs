use crate::protobuf::from::{FromProto, ProtoFrom};
use tonic::Status;

pub trait IntoProto<P>
where
    P: Sized,
{
    fn into_proto(self) -> Result<P, Status>;
}

impl<P, T> IntoProto<P> for T
where
    P: ProtoFrom<T>,
{
    fn into_proto(self) -> Result<P, Status> {
        P::proto_from(self)
    }
}

pub trait ProtoInto<T>
where
    Self: Sized,
{
    fn proto_into(self) -> Result<T, Status>;

    fn proto_field_into(self, field_name: &str) -> Result<T, Status>;
}

impl<P, T> ProtoInto<T> for P
where
    T: FromProto<P>,
{
    fn proto_into(self) -> Result<T, Status> {
        T::from_proto(self)
    }

    fn proto_field_into(self, field_name: &str) -> Result<T, Status> {
        T::from_proto_field(self, field_name)
    }
}
