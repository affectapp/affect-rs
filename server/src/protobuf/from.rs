use affect_status::invalid_argument;
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

    fn from_proto_field(proto: P, field_name: &str) -> Result<Self, Status> {
        Self::from_proto(proto)
            .map_err(|e| invalid_argument!("'{}' is invalid: {:?}", field_name, e))
    }
}
