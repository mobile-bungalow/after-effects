use crate::*;

define_suite!(
    WorldSuite2,
    PF_WorldSuite2,
    kPFWorldSuite,
    kPFWorldSuiteVersion2
);

impl WorldSuite2 {
    /// Acquire this suite from the host. Returns error if the suite is not available.
    /// Suite is released on drop.
    pub fn new() -> Result<Self, Error> {
        crate::Suite::new()
    }
    pub fn get_pixel_format(&self, effect_world: EffectWorld) -> Result<PixelFormat, Error> {
        Ok(call_suite_fn_single!(self, PF_GetPixelFormat -> ae_sys::PF_PixelFormat, effect_world.as_ptr())?.into())
    }
}
