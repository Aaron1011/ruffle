//! `flash.display.BitmapData` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{bitmapdata_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapData;
use crate::bitmap::is_size_valid;
use crate::character::Character;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.BitmapData`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        let name = this.instance_of_class_definition().map(|c| c.read().name());
        let character = this
            .instance_of()
            .and_then(|t| {
                activation
                    .context
                    .library
                    .avm2_class_registry()
                    .class_symbol(t)
            })
            .and_then(|(movie, chara_id)| {
                activation
                    .context
                    .library
                    .library_for_movie_mut(movie)
                    .character_by_id(chara_id)
                    .cloned()
            });

        let new_bitmap_data =
            GcCell::allocate(activation.context.gc_context, BitmapData::default());

        if let Some(Character::Bitmap(bd)) = character {
            let bitmap_handle = bd.bitmap_handle();

            if let Some(bitmap_handle) = bitmap_handle {
                if let Some(bitmap_pixels) =
                    activation.context.renderer.get_bitmap_pixels(bitmap_handle)
                {
                    let bitmap_pixels: Vec<i32> = bitmap_pixels.into();
                    new_bitmap_data
                        .write(activation.context.gc_context)
                        .set_pixels(
                            bd.width().into(),
                            bd.height().into(),
                            true,
                            bitmap_pixels.into_iter().map(|p| p.into()).collect(),
                        );
                } else {
                    log::warn!(
                        "Could not read bitmap data associated with class {:?}",
                        name
                    );
                }
            }
        } else {
            if character.is_some() {
                //TODO: Determine if mismatched symbols will still work as a
                //regular BitmapData subclass, or if this should throw
                log::warn!(
                    "BitmapData subclass {:?} is associated with a non-bitmap symbol",
                    name
                );
            }

            let width = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)? as u32;
            let height = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)? as u32;
            let transparency = args
                .get(2)
                .unwrap_or(&Value::Bool(true))
                .coerce_to_boolean();
            let fill_color = args
                .get(3)
                .unwrap_or(&Value::Unsigned(0xFFFFFFFF))
                .coerce_to_u32(activation)?;

            if !is_size_valid(activation.context.swf.version(), width, height) {
                return Err("Bitmap size is not valid".into());
            }

            new_bitmap_data
                .write(activation.context.gc_context)
                .init_pixels(width, height, transparency, fill_color as i32);
        }

        new_bitmap_data
            .write(activation.context.gc_context)
            .init_object2(this);
        this.init_bitmap_data(activation.context.gc_context, new_bitmap_data);
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.BitmapData`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `BitmapData.width`'s getter.
pub fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        return Ok((bitmap_data.read().width() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.height`'s getter.
pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        return Ok((bitmap_data.read().height() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.transparent`'s getter.
pub fn transparent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        return Ok(bitmap_data.read().transparency().into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel`.
pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        return Ok((bitmap_data.read().get_pixel(x, y) as u32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.draw`
pub fn draw<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("BitmapData.draw - not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `BitmapData.lock`
pub fn lock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("BitmapData.lock - not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `BitmapData.unlock`
pub fn unlock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("BitmapData.unlock - not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `BitmapData.fillRect`
pub fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let rectangle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        if let Some(color_val) = args.get(1) {
            let color = color_val.coerce_to_i32(activation)?;

            let x = rectangle
                .get_property(&QName::dynamic_name("x").into(), activation)?
                .coerce_to_u32(activation)?;
            let y = rectangle
                .get_property(&QName::dynamic_name("y").into(), activation)?
                .coerce_to_u32(activation)?;
            let width = rectangle
                .get_property(&QName::dynamic_name("width").into(), activation)?
                .coerce_to_u32(activation)?;
            let height = rectangle
                .get_property(&QName::dynamic_name("height").into(), activation)?
                .coerce_to_u32(activation)?;

            bitmap_data.write(activation.context.gc_context).fill_rect(
                x,
                y,
                width,
                height,
                color.into(),
            );
        }
        return Ok(Value::Undefined);
    }

    Ok(Value::Undefined)
}

pub fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let source_bitmap = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let source_rect = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let src_min_x = source_rect
            .get_property(&QName::dynamic_name("x").into(), activation)?
            .coerce_to_number(activation)? as i32;
        let src_min_y = source_rect
            .get_property(&QName::dynamic_name("y").into(), activation)?
            .coerce_to_number(activation)? as i32;
        let src_width = source_rect
            .get_property(&QName::dynamic_name("width").into(), activation)?
            .coerce_to_number(activation)? as i32;
        let src_height = source_rect
            .get_property(&QName::dynamic_name("height").into(), activation)?
            .coerce_to_number(activation)? as i32;

        let dest_point = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let dest_x = dest_point
            .get_property(&QName::dynamic_name("x").into(), activation)?
            .coerce_to_number(activation)? as i32;
        let dest_y = dest_point
            .get_property(&QName::dynamic_name("y").into(), activation)?
            .coerce_to_number(activation)? as i32;

        if let Some(src_bitmap) = source_bitmap.as_bitmap_data() {
            // dealing with object aliasing...
            let src_bitmap_clone: BitmapData; // only initialized if source is the same object as self
            let src_bitmap_gc_ref; // only initialized if source is a different object than self
            let source_bitmap_ref = // holds the reference to either of the ones above
                if GcCell::ptr_eq(src_bitmap, bitmap_data) {
                    src_bitmap_clone = src_bitmap.read().clone();
                    &src_bitmap_clone
                } else {
                    src_bitmap_gc_ref = src_bitmap.read();
                    &src_bitmap_gc_ref
                };

            if args.len() >= 5 {
                let alpha_bitmap = args
                    .get(3)
                    .unwrap_or(&Value::Null);

                if let Some(alpha_bitmap) = alpha_bitmap.coerce_to_object(activation).ok().and_then(|b| b.as_bitmap_data()) {
                    let alpha_point = args
                        .get(4)
                        .unwrap_or(&Value::Null)
                        .coerce_to_object(activation)?;

                    let alpha_x = alpha_point
                        .get_property(&QName::dynamic_name("x").into(), activation)?
                        .coerce_to_number(activation)? as i32;

                    let alpha_y = alpha_point
                        .get_property(&QName::dynamic_name("y").into(), activation)?
                        .coerce_to_number(activation)? as i32;



                    // dealing with aliasing the same way as for the source
                    let alpha_bitmap_clone: BitmapData;
                    let alpha_bitmap_gc_ref;
                    let alpha_bitmap_ref = if GcCell::ptr_eq(alpha_bitmap, bitmap_data) {
                        alpha_bitmap_clone = alpha_bitmap.read().clone();
                        &alpha_bitmap_clone
                    } else {
                        alpha_bitmap_gc_ref = alpha_bitmap.read();
                        &alpha_bitmap_gc_ref
                    };

                    let merge_alpha = args.get(5).unwrap_or(&false.into()).coerce_to_boolean();

                    bitmap_data
                        .write(activation.context.gc_context)
                        .copy_pixels(
                            source_bitmap_ref,
                            (src_min_x, src_min_y, src_width, src_height),
                            (dest_x, dest_y),
                            Some((alpha_bitmap_ref, (alpha_x, alpha_y), merge_alpha)),
                        );
                }
            } else {
                bitmap_data
                    .write(activation.context.gc_context)
                    .copy_pixels(
                        source_bitmap_ref,
                        (src_min_x, src_min_y, src_width, src_height),
                        (dest_x, dest_y),
                        None,
                    );
            }
        }
    }
    return Ok(Value::Undefined);
}

/// Construct `BitmapData`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "BitmapData"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin(instance_init, "<BitmapData instance initializer>", mc),
        Method::from_builtin(class_init, "<BitmapData class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(bitmapdata_allocator);

    write.implements(QName::new(Namespace::package("flash.display"), "IBitmapDrawable").into());

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("width", Some(width), None),
        ("height", Some(height), None),
        ("transparent", Some(transparent), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("getPixel", get_pixel),
        ("draw", draw),
        ("lock", lock),
        ("unlock", unlock),
        ("fillRect", fill_rect),
        ("copyPixels", copy_pixels),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
