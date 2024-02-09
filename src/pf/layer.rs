use super::*;

#[derive(Debug)]
pub struct Layer {
    pub(crate) in_data: *const ae_sys::PF_InData,
    pub(crate) layer_ptr: *mut ae_sys::PF_LayerDef,
}

//pub world_flags: PF_WorldFlags,
//pub data: PF_PixelPtr,
//pub rowbytes: A_long,
//pub width: A_long,
//pub height: A_long,
//pub extent_hint: PF_UnionableRect,
//pub platform_ref: *mut ::std::os::raw::c_void,
//pub pix_aspect_ratio: PF_RationalScale,
//pub origin_x: A_long,
//pub origin_y: A_long,
//pub dephault: A_long,

impl Layer {
    pub fn from_raw(
        in_data: *const ae_sys::PF_InData,
        layer_ptr: *mut ae_sys::PF_LayerDef,
    ) -> Self {
        assert!(!in_data.is_null());
        assert!(!layer_ptr.is_null());
        Self { in_data, layer_ptr }
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.layer_ptr).width }
    }
    pub fn height(&self) -> i32 {
        unsafe { (*self.layer_ptr).height }
    }
    pub fn buffer_stride(&self) -> usize {
        unsafe { (*self.layer_ptr).rowbytes }.abs() as usize
    }
    pub fn rowbytes(&self) -> i32 {
        unsafe { (*self.layer_ptr).rowbytes }
    }
    pub fn extent_hint(&self) -> Rect {
        unsafe { (*self.layer_ptr).extent_hint.into() }
    }
    pub fn bit_depth(&self) -> i16 {
        let flags = WorldFlags::from_bits(unsafe { (*self.layer_ptr).world_flags } as _).unwrap();
        if flags.contains(WorldFlags::DEEP) {
            16
        } else if flags.contains(WorldFlags::RESERVED1) {
            32
        } else {
            (self.rowbytes().abs() as f32 / self.width() as f32).floor() as i16 / 4 * 8
        }
    }

    pub fn buffer(&self) -> &[u8] {
        // Stride can be negative, so we need to offset the pointer to get to the real beginning of the buffer
        let offset = if self.rowbytes() < 0 {
            (self.rowbytes() * (self.height() - 1)) as isize
        } else {
            0
        };
        unsafe {
            assert!(self.rowbytes().abs() > 0);
            assert!(!(*self.layer_ptr).data.is_null());
            std::slice::from_raw_parts(
                ((*self.layer_ptr).data as *const u8).offset(offset),
                (self.height() * self.rowbytes().abs()) as usize,
            )
        }
    }
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        // Stride can be negative, so we need to offset the pointer to get to the real beginning of the buffer
        let offset = if self.rowbytes() < 0 {
            (self.rowbytes() * (self.height() - 1)) as isize
        } else {
            0
        };
        unsafe {
            assert!(self.rowbytes().abs() > 0);
            assert!(!(*self.layer_ptr).data.is_null());
            std::slice::from_raw_parts_mut(
                ((*self.layer_ptr).data as *mut u8).offset(offset),
                (self.height() * self.rowbytes().abs()) as usize,
            )
        }
    }

    pub fn copy_from(
        &mut self,
        src: &Self,
        src_rect: Option<Rect>,
        dst_rect: Option<Rect>,
    ) -> Result<(), Error> {
        unsafe {
            if self.in_data.is_null()
                || self.layer_ptr.is_null()
                || src.layer_ptr.is_null()
                || (*self.in_data).utils.is_null()
                || (*self.in_data).effect_ref.is_null()
            {
                return Err(Error::BadCallbackParameter);
            }
            let copy_fn = (*(*self.in_data).utils)
                .copy
                .ok_or(Error::BadCallbackParameter)?;
            match copy_fn(
                (*self.in_data).effect_ref,
                src.layer_ptr,
                self.layer_ptr,
                src_rect
                    .map(|x| &mut x.into() as *mut _)
                    .unwrap_or(std::ptr::null_mut()),
                dst_rect
                    .map(|x| &mut x.into() as *mut _)
                    .unwrap_or(std::ptr::null_mut()),
            ) {
                0 => Ok(()),
                e => return Err(e.into()),
            }
        }
    }
    pub fn fill(&mut self, color: Pixel, rect: Option<Rect>) -> Result<(), Error> {
        unsafe {
            if self.in_data.is_null()
                || self.layer_ptr.is_null()
                || (*self.in_data).utils.is_null()
                || (*self.in_data).effect_ref.is_null()
            {
                return Err(Error::BadCallbackParameter);
            }
            let fill_fn = (*(*self.in_data).utils)
                .fill
                .ok_or(Error::BadCallbackParameter)?;
            match fill_fn(
                (*self.in_data).effect_ref,
                &color.into() as *const _,
                rect.map(|x| &x.into() as *const _)
                    .unwrap_or(std::ptr::null_mut()),
                self.layer_ptr,
            ) {
                0 => Ok(()),
                e => return Err(e.into()),
            }
        }
    }

    pub fn iterate<F>(
        &self,
        output: &mut Self,
        progress_base: i32,
        progress_final: i32,
        extent_hint: Rect,
        cb: F,
    ) -> Result<(), Error>
    where
        F: Fn(i32, i32, Pixel) -> Result<Pixel, Error>,
    {
        unsafe {
            if self.in_data.is_null()
                || self.layer_ptr.is_null()
                || output.layer_ptr.is_null()
                || (*self.in_data).utils.is_null()
            {
                return Err(Error::BadCallbackParameter);
            }
            let iterate_fn = (*(*self.in_data).utils)
                .iterate
                .ok_or(Error::BadCallbackParameter)?;

            let callback =
                Box::<Box<dyn Fn(i32, i32, Pixel) -> Result<Pixel, Error>>>::new(Box::new(cb));
            let refcon = &callback as *const _;
            match iterate_fn(
                self.in_data as *mut _,
                progress_base,
                progress_final,
                self.layer_ptr,
                &ae_sys::PF_LRect::from(extent_hint),
                refcon as *mut _,
                Some(iterate_8_func),
                output.layer_ptr,
            ) {
                0 => Ok(()),
                e => return Err(e.into()),
            }
        }
    }
}

unsafe extern "C" fn iterate_8_func(
    refcon: *mut std::ffi::c_void,
    x: i32,
    y: i32,
    in_p: *mut ae_sys::PF_Pixel,
    out_p: *mut ae_sys::PF_Pixel,
) -> ae_sys::PF_Err {
    if refcon.is_null() || in_p.is_null() || out_p.is_null() {
        return ae_sys::PF_Err_BAD_CALLBACK_PARAM as ae_sys::PF_Err;
    }
    let cb = &*(refcon as *const Box<Box<dyn Fn(i32, i32, Pixel) -> Result<Pixel, Error>>>);

    let ret = match cb(x, y, (*in_p).into()) {
        Ok(px) => {
            *out_p = px.into();
            ae_sys::PF_Err_NONE as ae_sys::PF_Err
        }
        Err(e) => e as ae_sys::PF_Err,
    };

    ret
}
