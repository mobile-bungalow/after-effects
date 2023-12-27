use crate::*;
pub use cstr_literal; // re-export

#[macro_export]
macro_rules! empty_impl_for {
    ($t:ty, $params_type:ty) => {
        impl AdobePluginInstance for $t {
            fn flatten(&self) -> Result<Vec<u8>, Error> { Ok(Vec::new()) }
            fn unflatten(_: &[u8]) -> Result<Self, Error> { Ok(Default::default()) }
            fn render(&self, in_data: InData, in_layer: &Layer, out_layer: &mut Layer, params: &ae::Parameters<$params_type>) -> Result<(), ae::Error> { Ok(()) }
            fn handle_command(&mut self, _: Command, _: InData, _: OutData) -> Result<(), Error> { Ok(()) }
        }
    };
}

#[macro_export]
macro_rules! register_plugin {
	($global_type:ty, $sequence_type:ty, $params_type:ty) => {
        use $crate::*;

        struct GlobalData {
            params: Parameters<$params_type>,
            plugin_instance: $global_type
        }

        trait AdobePluginGlobal : Default {
            fn can_load(host_name: &str, host_version: &str) -> bool;

            fn params_setup(&self, params: &mut Parameters<$params_type>) -> Result<(), Error>;
            fn handle_command(&self, command: Command, in_data: InData, out_data: OutData, ) -> Result<(), Error>;
        }
        trait AdobePluginInstance : Default {
            fn flatten(&self) -> Result<Vec<u8>, Error>;
            fn unflatten(serialized: &[u8]) -> Result<Self, Error>;

            fn render(&self, in_data: InData, in_layer: &Layer, out_layer: &mut Layer, params: &ae::Parameters<$params_type>) -> Result<(), ae::Error>;

            fn handle_command(&mut self, command: Command, in_data: InData, out_data: OutData) -> Result<(), Error>;
        }
        empty_impl_for!((), $params_type);

        unsafe fn get_sequence_handle<'a, S: AdobePluginInstance>(cmd: after_effects_sys::PF_Cmd, in_data_ptr: *mut after_effects_sys::PF_InData) -> Result<Option<(pf::Handle::<'a, S>, bool)>, Error> {
            Ok(if std::any::type_name::<S>() == "()" {
                // Don't allocate sequence data
                None
            } else if cmd == after_effects_sys::PF_Cmd_SEQUENCE_SETUP {
                // Allocate new sequence data
                Some((pf::Handle::new(S::default())?, true))
            } else if cmd == after_effects_sys::PF_Cmd_SEQUENCE_RESETUP {
                // Restore from flat handle
                if (*in_data_ptr).sequence_data.is_null() {
                    log::error!("Sequence data pointer is null in cmd: {:?}!", PfCmd(cmd));
                    Some((pf::Handle::new(S::default())?, true))
                } else {
                    let instance = FlatHandle::from_raw((*in_data_ptr).sequence_data as after_effects_sys::PF_Handle)?;
                    let bytes = instance.as_slice().ok_or(Error::InvalidIndex)?;

                    let handle = pf::Handle::new(S::unflatten(bytes).map_err(|_| Error::Struct)?)?;
                    Some((handle, true))
                }
            } else if cmd == after_effects_sys::PF_Cmd_RENDER || cmd == after_effects_sys::PF_Cmd_SMART_RENDER || cmd == after_effects_sys::PF_Cmd_FRAME_SETUP || cmd == after_effects_sys::PF_Cmd_FRAME_SETDOWN {
                // Read-only sequence data available through a suite only
                let seq_ptr = pf::EffectSequenceDataSuite1::new()
                    .and_then(|x| x.get_const_sequence_data(InData::from_raw(in_data_ptr)))
                    .unwrap_or((*in_data_ptr).sequence_data as *const _);
                if !seq_ptr.is_null() {
                    let instance_handle = pf::Handle::<S>::from_raw(seq_ptr as *mut _)?;
                    Some((instance_handle, false))
                } else {
                    log::error!("Sequence data pointer got through EffectSequenceDataSuite1 is null in cmd: {:?}!", PfCmd(cmd));
                    None
                }
            } else {
                if (*in_data_ptr).sequence_data.is_null() {
                    log::error!("Sequence data pointer is null in cmd: {:?}!", PfCmd(cmd));
                    None
                } else {
                    let instance_handle = pf::Handle::<S>::from_raw((*in_data_ptr).sequence_data)?;
                    Some((instance_handle, false))
                }
            })
        }

        unsafe fn handle_effect_main<T: AdobePluginGlobal, S: AdobePluginInstance, P>(
            cmd: after_effects_sys::PF_Cmd,
            in_data_ptr: *mut after_effects_sys::PF_InData,
            out_data_ptr: *mut after_effects_sys::PF_OutData,
            params: *mut *mut after_effects_sys::PF_ParamDef,
            output: *mut after_effects_sys::PF_LayerDef,
            extra: *mut std::ffi::c_void) -> Result<(), Error>
        {
            let _pica = crate::PicaBasicSuite::from_pf_in_data_raw(in_data_ptr);

            let in_data = InData::from_raw(in_data_ptr);
            let out_data = OutData::from_raw(out_data_ptr);

            // Allocate or restore global data pointer
            let mut global_handle = if cmd == after_effects_sys::PF_Cmd_GLOBAL_SETUP {
                // Allocate global data
                pf::Handle::new(GlobalData {
                    params: Parameters::<$params_type>::from_in_data_ptr(in_data_ptr),
                    plugin_instance: <$global_type>::default()
                })?
            } else {
                if (*in_data_ptr).global_data.is_null() {
                    log::error!("Global data pointer is null in cmd: {:?}!", PfCmd(cmd));
                    return Err(Error::BadCallbackParameter);
                }
                pf::Handle::<GlobalData>::from_raw((*in_data_ptr).global_data)?
            };

            // Allocate or restore sequence data pointer
            let sequence_handle = get_sequence_handle::<S>(cmd, in_data_ptr).unwrap_or(None);

            let global_lock = global_handle.lock()?;
            let global_inst = global_lock.as_ref_mut()?;
            global_inst.params.set_params(in_data_ptr, params);

            let command = Command::from_entry_point(cmd, in_data_ptr, params, output, extra);

            let global_err = global_inst.plugin_instance.handle_command(command, in_data, out_data);
            let mut sequence_err = None;

            match cmd {
                after_effects_sys::PF_Cmd_PARAMS_SETUP => {
                    global_inst.plugin_instance.params_setup(&mut global_inst.params)?;
                    (*out_data_ptr).num_params = global_inst.params.num_params() as i32;
                }
                _ => { }
            }

            if let Some((mut sequence_handle, needs_lock)) = sequence_handle {
                let (lock, inst) = if needs_lock {
                    let lock = sequence_handle.lock()?;
                    let inst = lock.as_ref_mut()?;
                    (Some(lock), inst)
                } else {
                    (None, sequence_handle.as_mut().unwrap())
                };
                let in_data = InData::from_raw(in_data_ptr);
                let out_data = OutData::from_raw(out_data_ptr);
                let command = Command::from_entry_point(cmd, in_data_ptr, params, output, extra);

                sequence_err = Some(inst.handle_command(command, in_data, out_data));

                match cmd {
                    after_effects_sys::PF_Cmd_RENDER => {
                        let in_layer = $crate::Layer::from_raw(in_data_ptr, &mut (*(*params)).u.ld);
                        let mut out_layer = $crate::Layer::from_raw(in_data_ptr, output);
                        sequence_err = Some(inst.render(InData::from_raw(in_data_ptr), &in_layer, &mut out_layer, &global_inst.params));
                    }
                    _ => { }
                }

                match cmd {
                    after_effects_sys::PF_Cmd_SEQUENCE_SETUP | after_effects_sys::PF_Cmd_SEQUENCE_RESETUP => {
                        drop(lock);
                        (*out_data_ptr).sequence_data = pf::Handle::into_raw(sequence_handle);
                    }
                    after_effects_sys::PF_Cmd_SEQUENCE_FLATTEN | after_effects_sys::PF_Cmd_GET_FLATTENED_SEQUENCE_DATA => {
                        let serialized = inst.flatten().map_err(|_| Error::InternalStructDamaged)?;
                        drop(lock);
                        if cmd == after_effects_sys::PF_Cmd_GET_FLATTENED_SEQUENCE_DATA {
                            let _ = pf::Handle::into_raw(sequence_handle); // don't deallocate
                        } else {
                            drop(sequence_handle);
                        }
                        (*out_data_ptr).sequence_data = pf::FlatHandle::into_raw(FlatHandle::new(serialized)?) as *mut _;
                    }
                    after_effects_sys::PF_Cmd_SEQUENCE_SETDOWN => {
                        (*out_data_ptr).sequence_data = std::ptr::null_mut();
                        // sequence will be dropped and deallocated here
                    }
                    _ => {
                        drop(lock);
                        let _ = pf::Handle::into_raw(sequence_handle); // don't deallocate
                    }
                }
            }

            match cmd {
                after_effects_sys::PF_Cmd_GLOBAL_SETUP => {
                    drop(global_lock);
                    (*out_data_ptr).global_data = pf::Handle::into_raw(global_handle);
                }
                after_effects_sys::PF_Cmd_GLOBAL_SETDOWN => {
                    (*out_data_ptr).global_data = std::ptr::null_mut();
                    // global will be dropped and de-allocated here
                }
                _ => {
                    drop(global_lock);
                    let _ = pf::Handle::into_raw(global_handle); // don't deallocate
                }
            }

            if global_err.is_err() {
                return global_err;
            }
            if sequence_err.is_some() && sequence_err.unwrap().is_err() {
                return sequence_err.unwrap();
            }

            Ok(())
        }

        #[no_mangle]
        pub unsafe extern "C" fn PluginDataEntryFunction2(
            in_ptr: after_effects_sys::PF_PluginDataPtr,
            in_plugin_data_callback_ptr: after_effects_sys::PF_PluginDataCB2,
            in_sp_basic_suite_ptr: *const after_effects_sys::SPBasicSuite,
            in_host_name: *const std::ffi::c_char,
            in_host_version: *const std::ffi::c_char) -> after_effects_sys::PF_Err
        {
            // let _pica = ae::PicaBasicSuite::from_sp_basic_suite_raw(in_sp_basic_suite_ptr);

            if in_host_name.is_null() || in_host_version.is_null() {
                return after_effects_sys::PF_Err_INVALID_CALLBACK as after_effects_sys::PF_Err;
            }

            let in_host_name = std::ffi::CStr::from_ptr(in_host_name);
            let in_host_version = std::ffi::CStr::from_ptr(in_host_version);

            if !<$global_type>::can_load(in_host_name.to_str().unwrap(), in_host_version.to_str().unwrap()) {
                // Plugin said we don't want to load in this host, so exit here
                return after_effects_sys::PF_Err_INVALID_CALLBACK as after_effects_sys::PF_Err;
            }
            if let Some(cb_ptr) = in_plugin_data_callback_ptr {
                use $crate::cstr_literal::cstr;
                cb_ptr(in_ptr,
                    cstr!(env!("PIPL_NAME"))       .as_ptr() as *const u8, // Name
                    cstr!(env!("PIPL_MATCH_NAME")) .as_ptr() as *const u8, // Match Name
                    cstr!(env!("PIPL_CATEGORY"))   .as_ptr() as *const u8, // Category
                    cstr!(env!("PIPL_ENTRYPOINT")) .as_ptr() as *const u8, // Entry point
                    env!("PIPL_KIND")              .parse().unwrap(),
                    env!("PIPL_AE_SPEC_VER_MAJOR") .parse().unwrap(),
                    env!("PIPL_AE_SPEC_VER_MINOR") .parse().unwrap(),
                    env!("PIPL_AE_RESERVED")       .parse().unwrap(),
                    cstr!(env!("PIPL_SUPPORT_URL")).as_ptr() as *const u8, // Support url
                )
            } else {
                after_effects_sys::PF_Err_INVALID_CALLBACK as after_effects_sys::PF_Err
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn EffectMain(
            cmd: after_effects_sys::PF_Cmd,
            in_data_ptr: *mut after_effects_sys::PF_InData,
            out_data_ptr: *mut after_effects_sys::PF_OutData,
            params: *mut *mut after_effects_sys::PF_ParamDef,
            output: *mut after_effects_sys::PF_LayerDef,
            extra: *mut std::ffi::c_void) -> after_effects_sys::PF_Err
        {
            if cmd == after_effects_sys::PF_Cmd_GLOBAL_SETUP {
                (*out_data_ptr).my_version = env!("PIPL_VERSION")  .parse::<u32>().unwrap();
                (*out_data_ptr).out_flags  = env!("PIPL_OUTFLAGS") .parse::<i32>().unwrap();
                (*out_data_ptr).out_flags2 = env!("PIPL_OUTFLAGS2").parse::<i32>().unwrap();
            }

            #[cfg(threaded_rendering)]
            {
                fn assert_impl<T: Sync>() { }
                assert_impl::<$global_type>();
                assert_impl::<$sequence_type>();
            }

            match handle_effect_main::<$global_type, $sequence_type, $params_type>(cmd, in_data_ptr, out_data_ptr, params, output, extra) {
                Ok(_) => after_effects_sys::PF_Err_NONE,
                Err(e) => e as after_effects_sys::PF_Err
            }
        }
	};
}