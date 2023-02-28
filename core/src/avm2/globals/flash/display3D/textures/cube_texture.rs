use crate::avm2::Activation;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};
use crate::avm2_stub_method;

pub fn upload_from_bitmap_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(texture) = this.and_then(|this| this.as_texture()) {
        if let Some(source) = args[0].coerce_to_object(activation)?.as_bitmap_data() {
            let side = args[1].coerce_to_u32(activation)?;
            let mip_level = args[2].coerce_to_u32(activation)?;

            eprintln!("Texture BitmapData upload: {:?}", texture);

            if mip_level == 0 {
                texture.context3d().copy_bitmap_to_texture(
                    activation,
                    source,
                    texture.handle(),
                    // FIXME - is this right?
                    side,
                );
            } else {
                avm2_stub_method!(
                    activation,
                    "flash.display3D.textures.CubeTexture",
                    "uploadFromBitmapData",
                    "with miplevel != 0"
                );
            }
        } else {
            panic!("Invalid source: {:?}", args[0]);
        }
    }
    Ok(Value::Undefined)
}
