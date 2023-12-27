use flash_lso::{amf3::read::AMF3Decoder, types::AMFVersion};

use crate::avm2::{
    object::MessageChannelMessage, parameters::ParametersExt, Activation, Error, Object, TObject,
    Value,
};

pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg = args[0];
    let queue_limit = args.get_i32(activation, 1)?;

    if queue_limit != -1 {
        return Err(Error::RustError(
            format!("queue_limit of {queue_limit} not yet implemented").into(),
        ));
    }

    let classes = activation.avm2().classes();
    let worker_class = classes.worker.into();
    let message_channel_class = classes.messagechannel.into();
    let byte_array_class = classes.bytearray.into();
    let mutex_class = classes.mutex.into();
    let condition_class = classes.condition.into();

    if let Some(obj) = arg.as_object() {
        if obj.is_instance_of(activation, worker_class)? {
            return Err(Error::RustError(
                "Sending Worker not yet implemented".into(),
            ));
        } else if obj.is_instance_of(activation, message_channel_class)? {
            return Err(Error::RustError(
                "Sending MessageChannel not yet implemented".into(),
            ));
        } else if obj.is_instance_of(activation, byte_array_class)? {
            return Err(Error::RustError(
                "Sending ByteArray not yet implemented".into(),
            ));
        } else if obj.is_instance_of(activation, mutex_class)? {
            return Err(Error::RustError("Sending Mutex not yet implemented".into()));
        } else if obj.is_instance_of(activation, condition_class)? {
            return Err(Error::RustError(
                "Sending ByteArray not yet implemented".into(),
            ));
        }
    }

    let bytes = crate::avm2::amf::serialize_value_no_header(activation, arg, AMFVersion::AMF3)?;

    let channel = this.as_message_channel_object().unwrap();
    channel.send(MessageChannelMessage::AMF3(bytes), activation)?;

    Ok(Value::Undefined)
}

pub fn receive<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let block_until_received = args.get_bool(0);
    let channel = this.as_message_channel_object().unwrap();
    let msg = channel.receive(block_until_received, activation)?;

    if !block_until_received && msg.is_none() {
        return Ok(Value::Null);
    }

    let msg = msg.expect("Should have received a message with blocking receive");
    let value = match msg {
        MessageChannelMessage::AMF3(bytes) => {
            let mut decoder = AMF3Decoder::default();
            let (_, amf) = decoder
                .parse_single_element(&bytes)
                .map_err(|_| "Error: Invalid object")?;
            crate::avm2::amf::deserialize_value(activation, &amf)?
        }
    };
    Ok(value)
}
