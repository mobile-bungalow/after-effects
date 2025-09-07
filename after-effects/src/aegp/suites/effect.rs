use crate::*;
use crate::aegp::*;
use ae_sys::{ AEGP_EffectRefH, AEGP_LayerH };

define_suite!(
    /// Access the effects applied to a layer. This suite provides access to all parameter data streams.
    ///
    /// Use the [`suites::Stream`](aegp::suites::Stream) to work with those streams.
    ///
    /// An [`EffectRefHandle`] is a reference to an applied effect. An [`InstalledEffectKey`] is a reference to an installed effect, which may or may not be currently applied to anything.
    ///
    /// If Foobarocity is applied to a layer twice, there will be two distinct [`EffectRefHandle`]s, but they'll both return the same [`InstalledEffectKey`].
    EffectSuite,
    AEGP_EffectSuite4,
    kAEGPEffectSuite,
    kAEGPEffectSuiteVersion4
);

define_suite!(
    EffectSuite5,
    AEGP_EffectSuite5,
    kAEGPEffectSuite,
    kAEGPEffectSuiteVersion5
);

impl EffectSuite {
    /// Acquire this suite from the host. Returns error if the suite is not available.
    /// Suite is released on drop.
    pub fn new() -> Result<Self, Error> {
        crate::Suite::new()
    }

    /// Get the number of effects applied to a layer.
    pub fn layer_num_effects(&self, layer: impl AsPtr<AEGP_LayerH>) -> Result<i32, Error> {
        Ok(call_suite_fn_single!(self, AEGP_GetLayerNumEffects -> ae_sys::A_long, layer.as_ptr())? as i32)
    }

    /// Retrieves (by index) a reference to an effect applied to the layer.
    pub fn layer_effect_by_index(&self, layer: impl AsPtr<AEGP_LayerH>, plugin_id: PluginId, index: i32) -> Result<EffectRefHandle, Error> {
        Ok(EffectRefHandle::from_raw(
            call_suite_fn_single!(self, AEGP_GetLayerEffectByIndex -> ae_sys::AEGP_EffectRefH, plugin_id, layer.as_ptr(), index)?
        ))
    }

    /// Given an [`EffectRefHandle`], retrieves its associated [`InstalledEffectKey`].
    pub fn installed_key_from_layer_effect(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>) -> Result<InstalledEffectKey, Error> {
        Ok(call_suite_fn_single!(self, AEGP_GetInstalledKeyFromLayerEffect -> ae_sys::AEGP_InstalledEffectKey, effect_ref.as_ptr())?.into())
    }

    /// Returns description of effect parameter.
    ///
    /// Do not use the value(s) in the Param returned by this function (Use [`suites::Stream::new_stream_value()`](aegp::suites::Stream::new_stream_value) instead);
    /// it's provided so AEGPs can access parameter defaults, checkbox names, and pop-up strings.
    ///
    /// Use [`suites::Stream::effect_num_param_streams()`](aegp::suites::Stream::effect_num_param_streams) to get the stream count, useful for determining the maximum `param_index`.
    pub fn effect_param_union_by_index(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, plugin_id: PluginId, param_index: i32) -> Result<pf::Param<'_>, Error> {
        let (param_type, u) = call_suite_fn_double!(self, AEGP_GetEffectParamUnionByIndex -> ae_sys::PF_ParamType, ae_sys::PF_ParamDefUnion, plugin_id, effect_ref.as_ptr(), param_index)?;

        unsafe {
            match param_type {
                ae_sys::PF_Param_ANGLE          => Ok(Param::Angle      (AngleDef      ::from_owned(u.ad))),
                ae_sys::PF_Param_ARBITRARY_DATA => Ok(Param::Arbitrary  (ArbitraryDef  ::from_owned(u.arb_d))),
                ae_sys::PF_Param_BUTTON         => Ok(Param::Button     (ButtonDef     ::from_owned(u.button_d))),
                ae_sys::PF_Param_CHECKBOX       => Ok(Param::CheckBox   (CheckBoxDef   ::from_owned(u.bd))),
                ae_sys::PF_Param_COLOR          => Ok(Param::Color      (ColorDef      ::from_owned(u.cd))),
                ae_sys::PF_Param_FLOAT_SLIDER   => Ok(Param::FloatSlider(FloatSliderDef::from_owned(u.fs_d))),
                ae_sys::PF_Param_POPUP          => Ok(Param::Popup      (PopupDef      ::from_owned(u.pd))),
                ae_sys::PF_Param_SLIDER         => Ok(Param::Slider     (SliderDef     ::from_owned(u.sd))),
                ae_sys::PF_Param_POINT          => Ok(Param::Point      (PointDef      ::from_owned(u.td))),
                ae_sys::PF_Param_POINT_3D       => Ok(Param::Point3D    (Point3DDef    ::from_owned(u.point3d_d))),
                ae_sys::PF_Param_PATH           => Ok(Param::Path       (PathDef       ::from_owned(u.path_d))),
                _ => Err(Error::InvalidParms),
            }
        }
    }

    /// Obtains the flags for the given [`EffectRefHandle`].
    pub fn effect_flags(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>) -> Result<EffectFlags, Error> {
        Ok(EffectFlags::from_bits_truncate(call_suite_fn_single!(self, AEGP_GetEffectFlags -> ae_sys::AEGP_EffectFlags, effect_ref.as_ptr())?))
    }

    /// Sets the flags for the given [`EffectRefHandle`], masked by a different set of effect flags.
    pub fn set_effect_flags(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, set_mask: EffectFlags, flags: EffectFlags) -> Result<(), Error> {
        call_suite_fn!(self, AEGP_SetEffectFlags, effect_ref.as_ptr(), set_mask.bits(), flags.bits())
    }

    /// Change the order of applied effects (pass the requested index).
    pub fn reorder_effect(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, index: i32) -> Result<(), Error> {
        call_suite_fn!(self, AEGP_ReorderEffect, effect_ref.as_ptr(), index)
    }

    /// Call an effect plug-in, and pass it a pointer to any data you like; the effect can modify it.
    ///
    /// This is how AEGPs communicate with effects.
    ///
    /// Pass [`Command::CompletelyGeneral`](crate::Command::CompletelyGeneral) for `command` to get the old behaviour.
    pub fn effect_call_generic<T: Sized>(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, plugin_id: PluginId, time: Time, command: &pf::Command, extra_payload: Option<&T>) -> Result<(), Error> {
        // T is Sized so it can never be a fat pointer which means we are safe to transmute here.
        // Alternatively we could write extra_payload.map(|p| p as *const _).unwrap_or(core::ptr::null())
        call_suite_fn!(self, AEGP_EffectCallGeneric, plugin_id, effect_ref.as_ptr(), &time.into() as *const _, command.as_raw(), std::mem::transmute(extra_payload))
    }

    /// Disposes of an [`EffectRefHandle`]. Use this to dispose of any [`EffectRefHandle`] returned by After Effects.
    pub fn dispose_effect(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>) -> Result<(), Error> {
        call_suite_fn!(self, AEGP_DisposeEffect, effect_ref.as_ptr())
    }

    /// Apply an effect to a given layer. Returns the newly-created [`EffectRefHandle`].
    pub fn apply_effect(&self, layer: impl AsPtr<AEGP_LayerH>, plugin_id: PluginId, installed_effect_key: InstalledEffectKey) -> Result<EffectRefHandle, Error> {
        Ok(EffectRefHandle::from_raw(
            call_suite_fn_single!(self, AEGP_ApplyEffect -> ae_sys::AEGP_EffectRefH, plugin_id, layer.as_ptr(), installed_effect_key.into())?
        ))
    }

    /// Remove an applied effect.
    pub fn delete_layer_effect(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>) -> Result<(), Error> {
        call_suite_fn!(self, AEGP_DeleteLayerEffect, effect_ref.as_ptr())
    }

    /// Returns the count of effects installed in After Effects.
    pub fn num_installed_effects(&self) -> Result<i32, Error> {
        Ok(call_suite_fn_single!(self, AEGP_GetNumInstalledEffects -> ae_sys::A_long)? as i32)
    }

    /// Returns the [`InstalledEffectKey`] of the next installed effect.
    ///
    /// Pass [`InstalledEffectKey::None`] as the first parameter to obtain the first [`InstalledEffectKey`].
    pub fn next_installed_effect(&self, installed_effect_key: InstalledEffectKey) -> Result<InstalledEffectKey, Error> {
        Ok(call_suite_fn_single!(self, AEGP_GetNextInstalledEffect -> ae_sys::AEGP_InstalledEffectKey, installed_effect_key.into())?.into())
    }

    /// Get name of the effect. `name` can be up to `48` characters long.
    ///
    /// Note: use [`suites::DynamicStream::set_stream_name()`](aegp::suites::DynamicStream::set_stream_name) to change the display name of an effect.
    pub fn effect_name(&self, installed_effect_key: InstalledEffectKey) -> Result<String, Error> {
        let mut name = [0i8; ae_sys::AEGP_MAX_EFFECT_NAME_SIZE as usize + 1];
        call_suite_fn!(self, AEGP_GetEffectName, installed_effect_key.into(), name.as_mut_ptr() as _)?;
        Ok(unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// Get match name of an effect (defined in PiPL). `match_name` up to `48` characters long.
    ///
    /// Match names are in 7-bit ASCII. UI names are in the current application runtime encoding;
    /// for example, ISO 8859-1 for most languages on Windows.
    pub fn effect_match_name(&self, installed_effect_key: InstalledEffectKey) -> Result<String, Error> {
        let mut name = [0i8; ae_sys::AEGP_MAX_EFFECT_MATCH_NAME_SIZE as usize + 1];
        call_suite_fn!(self, AEGP_GetEffectMatchName, installed_effect_key.into(), name.as_mut_ptr() as _)?;
        // TODO: It's not UTF-8
        Ok(unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// Menu category of effect. `category` can be up to `48` characters long.
    pub fn effect_category(&self, installed_effect_key: InstalledEffectKey) -> Result<String, Error> {
        let mut name = [0i8; ae_sys::AEGP_MAX_EFFECT_CATEGORY_NAME_SIZE as usize + 1];
        call_suite_fn!(self, AEGP_GetEffectCategory, installed_effect_key.into(), name.as_mut_ptr() as _)?;
        Ok(unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// Duplicates a given [`EffectRefHandle`]. Caller must dispose of duplicate when finished.
    pub fn duplicate_effect(&self, original_effect_ref: impl AsPtr<AEGP_EffectRefH>) -> Result<EffectRefHandle, Error> {
        Ok(EffectRefHandle::from_raw(
            call_suite_fn_single!(self, AEGP_DuplicateEffect -> ae_sys::AEGP_EffectRefH, original_effect_ref.as_ptr())?
        ))
    }

    /// New in CC 2014. How many masks are on this effect?
    pub fn num_effect_mask(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>) -> Result<usize, Error> {
        Ok(call_suite_fn_single!(self, AEGP_NumEffectMask -> ae_sys::A_u_long, effect_ref.as_ptr())? as usize)
    }

    /// New in CC 2014. For a given `mask_index`, returns the corresponding `AEGP_MaskIDVal` for use in uniquely identifying the mask.
    pub fn effect_mask_id(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, mask_index: usize) -> Result<ae_sys::AEGP_MaskIDVal, Error> {
        call_suite_fn_single!(self, AEGP_GetEffectMaskID -> ae_sys::AEGP_MaskIDVal, effect_ref.as_ptr(), mask_index as ae_sys::A_u_long)
    }

    /// New in CC 2014. Add an effect mask, which may be created using the [`suites::Mask`](aegp::suites::Mask).
    ///
    /// Returns the local stream of the effect ref - useful if you want to add keyframes.
    ///
    /// Caller must dispose of [`StreamReferenceHandle`] when finished.
    ///
    /// Undoable.
    pub fn add_effect_mask(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, id_val: ae_sys::AEGP_MaskIDVal) -> Result<StreamReferenceHandle, Error> {
        Ok(StreamReferenceHandle::from_raw(
            call_suite_fn_single!(self, AEGP_AddEffectMask -> ae_sys::AEGP_StreamRefH, effect_ref.as_ptr(), id_val)?
        ))
    }

    /// New in CC 2014. Remove an effect mask.
    ///
    /// Undoable.
    pub fn remove_effect_mask(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, id_val: ae_sys::AEGP_MaskIDVal) -> Result<(), Error> {
        call_suite_fn!(self, AEGP_RemoveEffectMask, effect_ref.as_ptr(), id_val)
    }

    /// New in CC 2014. Set an effect mask on an existing index.
    ///
    /// Returns the local stream of the effect ref - useful if you want to add keyframes.
    ///
    /// Caller must dispose of [`StreamReferenceHandle`] when finished.
    ///
    /// Undoable.
    pub fn set_effect_mask(&self, effect_ref: impl AsPtr<AEGP_EffectRefH>, mask_index: usize, id_val: ae_sys::AEGP_MaskIDVal) -> Result<StreamReferenceHandle, Error> {
        Ok(StreamReferenceHandle::from_raw(
            call_suite_fn_single!(self, AEGP_SetEffectMask -> ae_sys::AEGP_StreamRefH, effect_ref.as_ptr(), mask_index as ae_sys::A_u_long, id_val)?
        ))
    }
    pub fn is_internal_effect(&self, installed_effect_key: ae_sys::AEGP_InstalledEffectKey) -> Result<bool, Error> {
        let v5 = EffectSuite5::new()?;
        Ok(call_suite_fn_single!(v5, AEGP_GetIsInternalEffect -> ae_sys::A_Boolean, installed_effect_key)? != 0)
    }
}

// ――――――――――――――――――――――――――――――――――――――― Types ――――――――――――――――――――――――――――――――――――――――

register_handle!(AEGP_EffectRefH);
define_handle_wrapper!(EffectRefHandle, AEGP_EffectRefH);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InstalledEffectKey {
    None,
    Key(i32)
}
impl From<ae_sys::AEGP_InstalledEffectKey> for InstalledEffectKey {
    fn from(key: ae_sys::AEGP_InstalledEffectKey) -> Self {
        if key == ae_sys::AEGP_InstalledEffectKey_NONE as ae_sys::AEGP_InstalledEffectKey  {
            InstalledEffectKey::None
        } else {
            InstalledEffectKey::Key(key as _)
        }
    }
}
impl Into<ae_sys::AEGP_InstalledEffectKey> for InstalledEffectKey {
    fn into(self) -> ae_sys::AEGP_InstalledEffectKey {
        match self {
            InstalledEffectKey::None     => ae_sys::AEGP_InstalledEffectKey_NONE as ae_sys::AEGP_InstalledEffectKey,
            InstalledEffectKey::Key(key) => key as ae_sys::AEGP_InstalledEffectKey,
        }
    }
}

bitflags::bitflags! {
    pub struct EffectFlags: ae_sys::A_long {
        const NONE       = ae_sys::AEGP_EffectFlags_NONE       as ae_sys::A_long;
        const ACTIVE     = ae_sys::AEGP_EffectFlags_ACTIVE     as ae_sys::A_long;
        const AUDIO_ONLY = ae_sys::AEGP_EffectFlags_AUDIO_ONLY as ae_sys::A_long;
        const AUDIO_TOO  = ae_sys::AEGP_EffectFlags_AUDIO_TOO  as ae_sys::A_long;
        const MISSING    = ae_sys::AEGP_EffectFlags_MISSING    as ae_sys::A_long;
    }
}

define_suite_item_wrapper!(
    ae_sys::AEGP_EffectRefH, EffectRefHandle,
    suite: EffectSuite,
    stream: aegp::suites::Stream,
    /// Access the effects applied to a layer. This suite provides access to all parameter data streams.
    ///
    /// Use the [`suites::Stream`](aegp::suites::Stream) to work with those streams.
    ///
    /// An [`EffectRefHandle`] is a reference to an applied effect. An [`InstalledEffectKey`] is a reference to an installed effect, which may or may not be currently applied to anything.
    ///
    /// If Foobarocity is applied to a layer twice, there will be two distinct [`EffectRefHandle`]s, but they'll both return the same [`InstalledEffectKey`].
    Effect {
        dispose: suite.dispose_effect;

        /// Retrieves its associated [`InstalledEffectKey`].
        installed_key() -> InstalledEffectKey => suite.installed_key_from_layer_effect,

        /// Returns description of effect parameter.
        ///
        /// Do not use the value(s) in the Param returned by this function (Use [`suites::Stream::new_stream_value()`](aegp::suites::Stream::new_stream_value) instead);
        /// it's provided so AEGPs can access parameter defaults, checkbox names, and pop-up strings.
        ///
        /// Use [`suites::Stream::effect_num_param_streams()`](aegp::suites::Stream::effect_num_param_streams) to get the stream count, useful for determining the maximum `param_index`.
        param_union_by_index(plugin_id: PluginId, param_index: i32) -> pf::Param<'_> => suite.effect_param_union_by_index,

        /// Obtains the flags for this [`Effect`].
        flags() -> EffectFlags => suite.effect_flags,

        /// Sets the flags for this [`Effect`], masked by a different set of effect flags.
        set_flags(set_mask: EffectFlags, flags: EffectFlags) -> () => suite.set_effect_flags,

        /// Change the order of applied effects (pass the requested index).
        reorder(index: i32) -> () => suite.reorder_effect,

        /// Remove an applied effect.
        delete_layert() -> () => suite.delete_layer_effect,

        /// Duplicates this [`Effect`]. Caller must dispose of duplicate when finished.
        duplicate() -> EffectRefHandle => suite.duplicate_effect,

        /// New in CC 2014. How many masks are on this effect?
        num_mask() -> usize => suite.num_effect_mask,

        /// New in CC 2014. For a given `mask_index`, returns the corresponding `AEGP_MaskIDVal` for use in uniquely identifying the mask.
        mask_id(mask_index: usize) -> ae_sys::AEGP_MaskIDVal => suite.effect_mask_id,

        /// New in CC 2014. Add an effect mask, which may be created using the [`suites::Mask`](aegp::suites::Mask).
        ///
        /// Returns the local stream of the effect ref - useful if you want to add keyframes.
        ///
        /// Undoable.
        add_mask(id_val: ae_sys::AEGP_MaskIDVal) -> Stream => suite.add_effect_mask,

        /// New in CC 2014. Remove an effect mask.
        ///
        /// Undoable.
        remove_mask(id_val: ae_sys::AEGP_MaskIDVal) -> () => suite.remove_effect_mask,

        /// New in CC 2014. Set an effect mask on an existing index.
        ///
        /// Returns the local stream of the effect ref - useful if you want to add keyframes.
        ///
        /// Undoable.
        set_mask(mask_index: usize, id_val: ae_sys::AEGP_MaskIDVal) -> Stream => suite.set_effect_mask,

        // ―――――――――――――――――――――――――――― Stream suite functions ――――――――――――――――――――――――――――

        /// Get number of parameter streams associated with an effect.
        num_param_streams() -> i32 => stream.effect_num_param_streams,

        /// Get an effect's parameter stream.
        new_stream_by_index(plugin_id: PluginId, index: i32) -> Stream => stream.new_effect_stream_by_index,
    }
);

impl Effect {
    /// Returns a new AEGP effect from the provided `effect_ref` and AEGP plugin ID.
    pub fn new(effect_ref: impl AsPtr<ae_sys::PF_ProgPtr>, plugin_id: PluginId) -> Result<Self, Error> {
        Ok(Self::from_handle(
            aegp::suites::PFInterface::new()?.new_effect_for_effect(effect_ref, plugin_id)?,
            true
        ))
    }

    /// Creates a new [`aegp::LayerRenderOptions`] from this layer.
    pub fn layer_render_options(&self, plugin_id: PluginId) -> Result<aegp::LayerRenderOptions, Error> {
        aegp::LayerRenderOptions::from_upstream_of_effect(self.handle.as_ptr(), plugin_id)
    }

    /// Returns the count of effects installed in After Effects.
    pub fn num_installed_effects() -> Result<i32, Error> {
        EffectSuite::new()?.num_installed_effects()
    }

    /// Returns the [`InstalledEffectKey`] of the next installed effect.
    ///
    /// Pass [`InstalledEffectKey::None`] as the first parameter to obtain the first [`InstalledEffectKey`].
    pub fn next_installed_effect(installed_effect_key: InstalledEffectKey) -> Result<InstalledEffectKey, Error> {
        EffectSuite::new()?.next_installed_effect(installed_effect_key)
    }

    /// Get name of the effect. `name` can be up to `48` characters long.
    ///
    /// Note: use [`suites::DynamicStream::set_stream_name()`](aegp::suites::DynamicStream::set_stream_name) to change the display name of an effect.
    pub fn name_of(installed_effect_key: InstalledEffectKey) -> Result<String, Error> {
        EffectSuite::new()?.effect_name(installed_effect_key)
    }

    /// Get match name of an effect (defined in PiPL). Can be up to `48` characters long.
    ///
    /// Match names are in 7-bit ASCII. UI names are in the current application runtime encoding;
    /// for example, ISO 8859-1 for most languages on Windows.
    pub fn match_name_of(installed_effect_key: InstalledEffectKey) -> Result<String, Error> {
        EffectSuite::new()?.effect_match_name(installed_effect_key)
    }

    /// Menu category of effect. Can be up to `48` characters long.
    pub fn category_of(installed_effect_key: InstalledEffectKey) -> Result<String, Error> {
        EffectSuite::new()?.effect_category(installed_effect_key)
    }
}
