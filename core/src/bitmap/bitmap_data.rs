use crate::avm2::{Object as Avm2Object, Value as Avm2Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use bitflags::bitflags;
use gc_arena::Collect;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{Bitmap, BitmapFormat, BitmapHandle, PixelRegion, SyncHandle};
use ruffle_wstr::WStr;
use std::fmt::Debug;
use std::ops::Range;
use swf::{Rectangle, Twips};
use tracing::instrument;

/// An implementation of the Lehmer/Park-Miller random number generator
/// Uses the fixed parameters m = 2,147,483,647 and a = 16,807
pub struct LehmerRng {
    x: u32,
}

impl LehmerRng {
    pub fn with_seed(seed: u32) -> Self {
        Self { x: seed }
    }

    /// Generate the next value in the sequence via the following formula
    /// X_(k+1) = a * X_k mod m
    pub fn gen(&mut self) -> u32 {
        self.x = ((self.x as u64).overflowing_mul(16_807).0 % 2_147_483_647) as u32;
        self.x
    }

    pub fn gen_range(&mut self, rng: Range<u8>) -> u8 {
        rng.start + (self.gen() % ((rng.end - rng.start) as u32 + 1)) as u8
    }
}

/// This can represent both a premultiplied and an unmultiplied ARGB color value.
///
/// Note that most operations only make sense on one of these representations:
/// For example, blending on premultiplied values, and applying a `ColorTransform` on
/// unmultiplied values. Make sure to convert the color to the correct form beforehand.
// TODO: Maybe split the type into `PremultipliedColor(u32)` and
//   `UnmultipliedColor(u32)`?
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Collect)]
#[collect(no_drop)]
pub struct Color(u32);

#[derive(Debug, Clone)]
pub enum BitmapDataDrawError {
    Unimplemented,
}

impl Color {
    pub fn blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn green(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn red(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn alpha(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    #[must_use]
    pub fn to_premultiplied_alpha(self, transparency: bool) -> Self {
        // This has some accuracy issues with some alpha values

        let old_alpha = if transparency { self.alpha() } else { 255 };

        let a = old_alpha as u32;
        let r = ((self.red() as u32 * a + 127) / 255) as u8;
        let g = ((self.green() as u32 * a + 127) / 255) as u8;
        let b = ((self.blue() as u32 * a + 127) / 255) as u8;

        Self::argb(old_alpha, r, g, b)
    }

    #[must_use]
    pub fn to_un_multiplied_alpha(self) -> Self {
        // We need to match Flash's results, and this lookup table was generated by brute force.
        // For each alpha value, every value between 0..256^3 was tested to see if it produced the
        // correct color value when reversing the premultiplication.
        // Source code used to generate this table can be found at:
        // https://gist.github.com/pdewacht/614b428cd42c2052dc0fd292516c9f9f
        const FLASH_PREMUL_FACTOR: [u32; 256] = [
            0, 16678912, 8339456, 5559638, 4169728, 3335783, 2779819, 2386603, 2086230, 1855488,
            1667892, 1518251, 1391151, 1285234, 1193302, 1111928, 1043895, 981113, 927744, 879275,
            834621, 795535, 759126, 726358, 695839, 668183, 642538, 618737, 596651, 576171, 555964,
            538706, 522104, 506319, 490557, 477321, 464038, 451353, 439544, 428244, 417582, 407500,
            397768, 388535, 379630, 371117, 363179, 355235, 348050, 340965, 334052, 327038, 321269,
            315077, 309159, 303586, 298189, 293092, 287981, 283080, 278251, 273892, 269268, 265179,
            261087, 256971, 253160, 249322, 245508, 242164, 238575, 235245, 231859, 228848, 225785,
            222712, 219616, 216827, 213985, 211432, 208835, 206075, 203750, 201196, 198895, 196223,
            194301, 191987, 189686, 187636, 185559, 183426, 181453, 179444, 177638, 175855, 174054,
            171948, 170489, 168695, 166889, 165365, 163519, 162045, 160508, 158970, 157429, 156150,
            154610, 153081, 151803, 150511, 148986, 147709, 146420, 145116, 143868, 142586, 141545,
            140277, 139194, 137957, 136954, 135676, 134652, 133621, 132604, 131577, 130552, 129527,
            128508, 127476, 126451, 125432, 124670, 123645, 122818, 121847, 121082, 120060, 119288,
            118263, 117502, 116720, 115967, 115195, 114424, 113655, 112893, 112125, 111356, 110563,
            109811, 109048, 108287, 107766, 107004, 106236, 105724, 104953, 104434, 103676, 102904,
            102375, 101879, 101119, 100604, 99834, 99321, 98813, 98112, 97533, 97019, 96509, 95994,
            95486, 94713, 94185, 93689, 93179, 92667, 92149, 91643, 91129, 90621, 90068, 89597,
            89342, 88829, 88318, 87804, 87294, 87034, 86523, 85994, 85499, 85245, 84732, 84222,
            83956, 83450, 82937, 82685, 82173, 81840, 81405, 80889, 80638, 80127, 79862, 79354,
            79103, 78590, 78332, 78077, 77565, 77308, 76795, 76541, 76284, 75766, 75518, 75262,
            74748, 74493, 74238, 73691, 73470, 73214, 72959, 72447, 72189, 71935, 71671, 71166,
            70911, 70651, 70399, 70140, 69886, 69615, 69116, 68861, 68603, 68350, 68093, 67839,
            67576, 67326, 67070, 66813, 66556, 66302, 66046, 65791, 65408,
        ];

        let alpha_factor = FLASH_PREMUL_FACTOR[self.alpha() as usize];
        let unmultiply = |c| ((c as u32 * alpha_factor + 0x8000) >> 16) as u8;

        let r = unmultiply(self.red());
        let g = unmultiply(self.green());
        let b = unmultiply(self.blue());

        Self::argb(self.alpha(), r, g, b)
    }

    #[must_use]
    pub fn argb(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self(u32::from_le_bytes([blue, green, red, alpha]))
    }

    #[must_use]
    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self::argb(alpha, self.red(), self.green(), self.blue())
    }

    /// # Arguments
    ///
    /// * `self` - Must be in premultiplied form.
    /// * `source` - Must be in premultiplied form.
    #[must_use]
    pub fn blend_over(&self, source: &Self) -> Self {
        let sa = source.alpha();

        let r = source.red() + ((self.red() as u16 * (255 - sa as u16)) >> 8) as u8;
        let g = source.green() + ((self.green() as u16 * (255 - sa as u16)) >> 8) as u8;
        let b = source.blue() + ((self.blue() as u16 * (255 - sa as u16)) >> 8) as u8;
        let a = source.alpha() + ((self.alpha() as u16 * (255 - sa as u16)) >> 8) as u8;
        Self::argb(a, r, g, b)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:#x}", self.0))
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<u32> for Color {
    fn from(i: u32) -> Self {
        Color(i)
    }
}

impl From<swf::Color> for Color {
    fn from(c: swf::Color) -> Self {
        Self::argb(c.a, c.r, c.g, c.b)
    }
}

impl From<Color> for swf::Color {
    fn from(c: Color) -> Self {
        let r = c.red();
        let g = c.green();
        let b = c.blue();
        let a = c.alpha();
        Self { r, g, b, a }
    }
}

bitflags! {
    pub struct ChannelOptions: u8 {
        const RED = 1 << 0;
        const GREEN = 1 << 1;
        const BLUE = 1 << 2;
        const ALPHA = 1 << 3;
        const RGB = Self::RED.bits() | Self::GREEN.bits() | Self::BLUE.bits();
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BitmapData<'gc> {
    /// The pixels in the bitmap, stored as a array of pre-multiplied ARGB colour values
    pixels: Vec<Color>,
    width: u32,
    height: u32,
    transparency: bool,

    // Note that it's technically possible to have a BitmapData with zero width and height,
    // (by embedding it in the SWF instead of using the BitmapData constructor),
    // so we need a separate 'disposed' flag.
    disposed: bool,

    /// The bitmap handle for this data.
    ///
    /// This is lazily initialized; a value of `None` indicates that
    /// initialization has not yet happened.
    #[collect(require_static)]
    bitmap_handle: Option<BitmapHandle>,

    /// The AVM2 side of this `BitmapData`.
    ///
    /// AVM1 cannot retrieve `BitmapData` back from the display object tree, so
    /// this does not need to hold an AVM1 object.
    avm2_object: Option<Avm2Object<'gc>>,

    dirty_state: DirtyState,
}

#[derive(Clone, Collect, Debug)]
#[collect(require_static)]
enum DirtyState {
    // Both the CPU and GPU pixels are up to date. We do not need to wait for any syncs to complete
    Clean,

    // The CPU pixels have been modified, and need to be synced to the GPU via `update_dirty_texture`
    CpuModified(PixelRegion),

    // The GPU pixels have been modified, and need to be synced to the CPU via `BitmapDataWrapper::sync`
    GpuModified(Box<dyn SyncHandle>, PixelRegion),
}

mod wrapper {
    use crate::avm2::{Object as Avm2Object, Value as Avm2Value};
    use crate::context::RenderContext;
    use gc_arena::{Collect, GcCell, MutationContext};
    use ruffle_render::backend::RenderBackend;
    use ruffle_render::bitmap::{BitmapHandle, PixelRegion};
    use ruffle_render::commands::CommandHandler;
    use std::cell::Ref;

    use super::{copy_pixels_to_bitmapdata, BitmapData, DirtyState};

    /// A wrapper type that ensures that we always wait for a pending
    /// GPU -> CPU sync to complete (using `sync_handle`) before accessing
    /// the CPU-side pixels.
    ///
    /// This is overly conservative - we perform a sync before allowing any access
    /// to the underlying `BitmapData`, even if we wouldn't be accessing the pixels.
    /// Implementing more fine-grained tracking turned out to be extremely invasive,
    /// and made the code much less readable. This should be enough for the simple
    /// case where ActionScript calls `BitmapData.draw`, and then doesn't interact
    /// with the Bitmap/BitmapData object at all for some time.
    ///
    /// There are three ways that this type gets used:
    /// 1. Blocking on the current GPU->CPU sync via the `sync` method,
    ///    and obtainng a `GcCell<'gc, BitmapData<'gc>>` (or implicily through `as_bitmap_data`).
    ///    This is done for the vast majority of BitmapData AS2/AS3 methods, as they need to access CPU-side pixels.
    /// 2. Ignoring the current GPU->CPU sync state. This is done by the `render` method defined on this type,
    ///    since rendering only uses GPU-side data, and ignores CPU-side pixels entirely.
    /// 3. Explicitly cancelling any in-progress GPU->CPU sync via `overwrite_cpu_pixels_from_gpu`. This is
    ///    used by `BitmapData.draw` and `BitmapData.apply_filter`, since the new rendering result will completely
    ///    replace the current CPU-side pixels. This performs a CPU -> GPU sync, to ensure that the GPU side
    ///    is up to date before we overwrite the CPU-side pixels.
    ///    In the future, we could explore using this in additional
    ///    cases where we know that the entire CPU-side pixel array will be overwritten without being read
    ///    (e.g. `BitmapData.fillRect` with a rectangle covering the entire bitmap). However, `overwrite_cpu_pixels`
    ///    is always a performance optimization, and can always be safely replaced with `sync` (at the cost of worse performance)
    ///
    /// Note that we also perform CPU-GPU syncs from `BitmapData.update_dirty_texture` when `dirty` is set.
    /// `sync_handle` and `dirty` can never be set at the same time - we can only have one of them set, or none of them set.
    #[derive(Copy, Clone, Collect, Debug)]
    #[collect(no_drop)]
    pub struct BitmapDataWrapper<'gc>(GcCell<'gc, BitmapData<'gc>>);

    impl<'gc> BitmapDataWrapper<'gc> {
        pub fn new(data: GcCell<'gc, BitmapData<'gc>>) -> Self {
            BitmapDataWrapper(data)
        }

        // Creates a dummy BitmapData with no pixels or handle, marked as disposed.
        // This is used for AS3 `Bitmap` instances without a corresponding AS3 `BitmapData` instance.
        // Marking it as disposed skips rendering, and the unset `avm2_object` will cause this to
        // be inaccessible to AS3 code.
        pub fn dummy(mc: MutationContext<'gc, '_>) -> Self {
            BitmapDataWrapper(GcCell::allocate(
                mc,
                BitmapData {
                    pixels: Vec::new(),
                    width: 0,
                    height: 0,
                    transparency: false,
                    disposed: true,
                    bitmap_handle: None,
                    avm2_object: None,
                    dirty_state: DirtyState::Clean,
                },
            ))
        }

        // Provides access to the underlying `BitmapData`. If a GPU -> CPU sync
        // is in progress, waits for it to complete
        pub fn sync(&self) -> GcCell<'gc, BitmapData<'gc>> {
            // SAFETY: The only field that can store gc pointers is `avm2_object`,
            // which we don't update here. Ideally, we would refactor this so that
            // `BitmapData` doesn't contain any gc pointers, allowing us to use a normal
            // `RefCell` instead of a `GcCell`.
            let mut write = unsafe { self.0.borrow_mut() };
            match std::mem::replace(&mut write.dirty_state, DirtyState::Clean) {
                DirtyState::GpuModified(sync_handle, bounds) => {
                    sync_handle
                        .retrieve_offscreen_texture(Box::new(|buffer, buffer_width| {
                            copy_pixels_to_bitmapdata(&mut write, buffer, buffer_width, bounds)
                        }))
                        .expect("Failed to sync BitmapData");
                    write.dirty_state = DirtyState::Clean
                }
                old_state => write.dirty_state = old_state,
            }
            self.0
        }

        /// Provides access to the underlying `BitmapHandle`.
        /// If the CPU pixels are dirty, syncs them to the GPU.
        /// If the GPU pixels are dirty, then handle is returned immediately
        /// without waiting for the sync to complete, as a BitmapHandle can
        /// only be used to access the GPU data. Unlike `overwrite_cpu_pixels_from_gpu`,
        /// this does not cancel the GPU -> CPU sync.
        pub fn bitmap_handle(
            &self,
            gc_context: MutationContext<'gc, '_>,
            renderer: &mut dyn RenderBackend,
        ) -> BitmapHandle {
            let mut bitmap_data = self.0.write(gc_context);
            bitmap_data.update_dirty_texture(renderer);
            bitmap_data.bitmap_handle(renderer).unwrap()
        }

        /// Provides access to the underlying `BitmapData`.
        /// This should only be used when you will be overwriting the entire
        /// `pixels` vec without reading from it. Cancels any in-progress GPU -> CPU sync.
        /// This does not sync from cpu to gpu.
        #[allow(clippy::type_complexity)]
        pub fn overwrite_cpu_pixels_from_gpu(
            &self,
            mc: MutationContext<'gc, '_>,
        ) -> (GcCell<'gc, BitmapData<'gc>>, Option<PixelRegion>) {
            let mut write = self.0.write(mc);
            let dirty_rect = match write.dirty_state {
                DirtyState::GpuModified(_, rect) => {
                    write.dirty_state = DirtyState::Clean;
                    Some(rect)
                }
                DirtyState::CpuModified(_) | DirtyState::Clean => None,
            };
            (self.0, dirty_rect)
        }

        /// Provides read access to the BitmapData pixels.
        /// Only the provided region is guaranteed to be up-to-date.
        /// It is an error to access any other pixels outside of that region.
        pub fn read_area(&self, read_area: PixelRegion) -> Ref<'_, BitmapData<'gc>> {
            let needs_update = if let DirtyState::GpuModified(_, area) = self.0.read().dirty_state {
                area.intersects(read_area)
            } else {
                false
            };
            if needs_update {
                self.sync();
            }
            self.0.read()
        }

        // These methods do not require a sync to complete, as they do not depend on the
        // CPU-side pixels. They are implemented directly on `BitmapDataWrapper`, allowing
        // callers to avoid calling sync()

        pub fn height(&self) -> u32 {
            self.0.read().height
        }

        pub fn width(&self) -> u32 {
            self.0.read().width
        }

        pub fn object2(&self) -> Avm2Value<'gc> {
            self.0.read().object2()
        }

        pub fn disposed(&self) -> bool {
            self.0.read().disposed
        }

        pub fn transparency(&self) -> bool {
            self.0.read().transparency
        }

        pub fn check_valid(
            &self,
            activation: &mut crate::avm2::Activation<'_, 'gc>,
        ) -> Result<(), crate::avm2::Error<'gc>> {
            if self.disposed() {
                return Err(crate::avm2::Error::AvmError(
                    crate::avm2::error::argument_error(
                        activation,
                        "Error #2015: Invalid BitmapData.",
                        2015,
                    )?,
                ));
            }
            Ok(())
        }

        pub fn dispose(&self, mc: MutationContext<'gc, '_>) {
            self.0.write(mc).dispose();
        }

        pub fn init_object2(&self, mc: MutationContext<'gc, '_>, object: Avm2Object<'gc>) {
            self.0.write(mc).avm2_object = Some(object)
        }

        pub fn render(&self, smoothing: bool, context: &mut RenderContext<'_, 'gc>) {
            let mut inner_bitmap_data = self.0.write(context.gc_context);
            if inner_bitmap_data.disposed() {
                return;
            }

            // Note - we do a CPU -> GPU sync, but we do *not* do a GPU -> CPU sync
            // (rendering is done on the GPU, so the CPU pixels don't need to be up-to-date).
            inner_bitmap_data.update_dirty_texture(context.renderer);
            let handle = inner_bitmap_data
                .bitmap_handle(context.renderer)
                .expect("Missing bitmap handle");

            context
                .commands
                .render_bitmap(handle, context.transform_stack.transform(), smoothing);
        }

        pub fn can_read(&self, read_area: PixelRegion) -> bool {
            if let DirtyState::GpuModified(_, area) = self.0.read().dirty_state {
                !area.intersects(read_area)
            } else {
                true
            }
        }

        pub fn is_point_in_bounds(&self, x: i32, y: i32) -> bool {
            x >= 0 && x < self.width() as i32 && y >= 0 && y < self.height() as i32
        }

        pub fn ptr_eq(&self, other: BitmapDataWrapper<'gc>) -> bool {
            GcCell::ptr_eq(self.0, other.0)
        }
    }
}

pub use wrapper::BitmapDataWrapper;

impl std::fmt::Debug for BitmapData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitmapData")
            .field("dirty_state", &self.dirty_state)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("transparency", &self.transparency)
            .field("disposed", &self.disposed)
            .field("bitmap_handle", &self.bitmap_handle)
            .finish()
    }
}

impl<'gc> BitmapData<'gc> {
    pub fn new(width: u32, height: u32, transparency: bool, fill_color: u32) -> Self {
        Self {
            pixels: vec![
                Color(fill_color).to_premultiplied_alpha(transparency);
                width as usize * height as usize
            ],
            width,
            height,
            transparency,
            disposed: false,
            bitmap_handle: None,
            avm2_object: None,
            dirty_state: DirtyState::Clean,
        }
    }

    pub fn new_with_pixels(
        width: u32,
        height: u32,
        transparency: bool,
        pixels: Vec<Color>,
    ) -> Self {
        Self {
            pixels,
            width,
            height,
            transparency,
            bitmap_handle: None,
            avm2_object: None,
            disposed: false,
            dirty_state: DirtyState::Clean,
        }
    }

    pub fn disposed(&self) -> bool {
        self.disposed
    }

    pub fn dispose(&mut self) {
        self.width = 0;
        self.height = 0;
        self.pixels.clear();
        self.bitmap_handle = None;
        // There's no longer a handle to update
        self.dirty_state = DirtyState::Clean;
        self.disposed = true;
    }

    pub fn bitmap_handle(&mut self, renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        if self.bitmap_handle.is_none() {
            let bitmap = Bitmap::new(
                self.width(),
                self.height(),
                BitmapFormat::Rgba,
                self.pixels_rgba(),
            );
            let bitmap_handle = renderer.register_bitmap(bitmap);
            if let Err(e) = &bitmap_handle {
                tracing::warn!("Failed to register raw bitmap for BitmapData: {:?}", e);
            }
            self.bitmap_handle = bitmap_handle.ok();
        }

        self.bitmap_handle.clone()
    }

    pub fn transparency(&self) -> bool {
        self.transparency
    }

    pub fn set_gpu_dirty(&mut self, sync_handle: Box<dyn SyncHandle>, region: PixelRegion) {
        self.dirty_state = DirtyState::GpuModified(sync_handle, region);
    }

    pub fn set_cpu_dirty(&mut self, region: PixelRegion) {
        debug_assert!(region.x_max <= self.width);
        debug_assert!(region.y_max <= self.height);
        match &mut self.dirty_state {
            DirtyState::CpuModified(old_region) => old_region.union(region),
            DirtyState::Clean => self.dirty_state = DirtyState::CpuModified(region),
            DirtyState::GpuModified(_, _) => {
                panic!("Attempted to modify CPU dirty state while GPU sync is in progress!")
            }
        }
    }

    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    pub fn pixels_rgba(&self) -> Vec<u8> {
        // TODO: This could have been implemented as follows:
        //
        // self.pixels
        //     .iter()
        //     .flat_map(|p| [p.red(), p.green(), p.blue(), p.alpha()])
        //     .collect()
        //
        // But currently Rust emits suboptimal code in that case. For now we use
        // `Vec::with_capacity` manually to avoid unnecessary re-allocations.

        let mut output = Vec::with_capacity(self.pixels.len() * 4);
        for p in &self.pixels {
            output.extend_from_slice(&[p.red(), p.green(), p.blue(), p.alpha()])
        }
        output
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn is_point_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width() as i32 && y >= 0 && y < self.height() as i32
    }

    #[inline]
    pub fn set_pixel32_raw(&mut self, x: u32, y: u32, color: Color) {
        self.pixels[(x + y * self.width) as usize] = color;
    }

    #[inline]
    pub fn get_pixel32_raw(&self, x: u32, y: u32) -> Color {
        self.pixels[(x + y * self.width()) as usize]
    }

    pub fn raw_pixels_mut(&mut self) -> &mut Vec<Color> {
        &mut self.pixels
    }

    pub fn raw_pixels(&self) -> &[Color] {
        &self.pixels
    }

    // Updates the data stored with our `BitmapHandle` if this `BitmapData`
    // is dirty
    pub fn update_dirty_texture(&mut self, renderer: &mut dyn RenderBackend) {
        let handle = self.bitmap_handle(renderer).unwrap();
        match &self.dirty_state {
            DirtyState::CpuModified(region) => {
                if let Err(e) = renderer.update_texture(
                    &handle,
                    Bitmap::new(
                        self.width(),
                        self.height(),
                        BitmapFormat::Rgba,
                        self.pixels_rgba(),
                    ),
                    *region,
                ) {
                    tracing::error!("Failed to update dirty bitmap {:?}: {:?}", handle, e);
                }
                self.dirty_state = DirtyState::Clean;
            }
            DirtyState::Clean | DirtyState::GpuModified(_, _) => {}
        }
    }

    pub fn object2(&self) -> Avm2Value<'gc> {
        self.avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Null)
    }

    pub fn init_object2(&mut self, object: Avm2Object<'gc>) {
        self.avm2_object = Some(object)
    }
}

pub enum IBitmapDrawable<'gc> {
    BitmapData(BitmapDataWrapper<'gc>),
    DisplayObject(DisplayObject<'gc>),
}

impl IBitmapDrawable<'_> {
    pub fn bounds(&self) -> Rectangle<Twips> {
        match self {
            IBitmapDrawable::BitmapData(bmd) => Rectangle {
                x_min: Twips::ZERO,
                x_max: Twips::from_pixels(bmd.width() as f64),
                y_min: Twips::ZERO,
                y_max: Twips::from_pixels(bmd.height() as f64),
            },
            IBitmapDrawable::DisplayObject(o) => o.bounds(),
        }
    }
}

#[instrument(level = "debug", skip_all)]
fn copy_pixels_to_bitmapdata(
    write: &mut BitmapData,
    buffer: &[u8],
    buffer_width: u32,
    area: PixelRegion,
) {
    let buffer_width_pixels = buffer_width / 4;

    for y in area.y_min..area.y_max {
        for x in area.x_min..area.x_max {
            // note: this order of conversions helps llvm realize the index is 4-byte-aligned
            let ind = (((x - area.x_min) + (y - area.y_min) * buffer_width_pixels) as usize) * 4;

            // TODO(mid): optimize this A LOT
            let r = buffer[ind];
            let g = buffer[ind + 1usize];
            let b = buffer[ind + 2usize];
            let a = if write.transparency() {
                buffer[ind + 3usize]
            } else {
                255
            };

            // TODO(later): we might want to swap Color storage from argb to rgba, to make it cheaper
            let nc = Color::argb(a, r, g, b);

            // Ignore the original color entirely - the blending (including alpha)
            // was done by the renderer when it wrote over the previous texture contents.
            write.set_pixel32_raw(x, y, nc);
        }
    }
    write.set_cpu_dirty(area);
}

#[derive(Copy, Clone, Debug)]
pub enum ThresholdOperation {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
}

impl ThresholdOperation {
    pub fn from_wstr(str: &WStr) -> Option<Self> {
        if str == b"==" {
            Some(Self::Equals)
        } else if str == b"!=" {
            Some(Self::NotEquals)
        } else if str == b"<" {
            Some(Self::LessThan)
        } else if str == b"<=" {
            Some(Self::LessThanOrEquals)
        } else if str == b">" {
            Some(Self::GreaterThan)
        } else if str == b">=" {
            Some(Self::GreaterThanOrEquals)
        } else {
            None
        }
    }

    pub fn matches(&self, value: u32, masked_threshold: u32) -> bool {
        match self {
            ThresholdOperation::Equals => value == masked_threshold,
            ThresholdOperation::NotEquals => value != masked_threshold,
            ThresholdOperation::LessThan => value < masked_threshold,
            ThresholdOperation::LessThanOrEquals => value <= masked_threshold,
            ThresholdOperation::GreaterThan => value > masked_threshold,
            ThresholdOperation::GreaterThanOrEquals => value >= masked_threshold,
        }
    }
}
