use tonic::Status;

pub trait ProtoFrom<T>
where
    Self: Sized,
{
    fn proto_from(value: T) -> Result<Self, Status>;
}

pub trait FromProto<P>
where
    Self: Sized,
{
    fn from_proto(proto: P) -> Result<Self, Status>;
}
