use crate::{
    avm2::{parameters::ParametersExt, Activation, Error, Object, TObject, Value},
    tag_utils::SwfMovie,
    PlayerBuilder,
};

pub fn create_worker<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let swf = args
        .get_object(activation, 0, "swf")?
        .as_bytearray_object()
        .unwrap();
    let swf = swf.as_bytearray().unwrap();
    let bytes = swf.read_at(swf.len(), 0).unwrap();
    let give_app_privileges = args.get_bool(1);

    let new_player = PlayerBuilder::new()
        .with_movie(SwfMovie::from_data(bytes, String::new(), None).unwrap())
        .build();

    let worker = activation
        .context
        .worker
        .as_worker_object()
        .unwrap()
        .new_non_primordial(new_player, activation);
    Ok(worker.into())
}
