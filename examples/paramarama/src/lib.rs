use after_effects as ae;
use after_effects_sys as ae_sys;

// This sample exercises some of After Effects' image processing callback functions.

const PARAMARAMA_AMOUNT_MIN:  i32 = 0;
const PARAMARAMA_AMOUNT_MAX:  i32 = 100;
const PARAMARAMA_AMOUNT_DFLT: i32 = 93;
const DEFAULT_RED:            u8 = 111;
const DEFAULT_GREEN:          u8 = 222;
const DEFAULT_BLUE:           u8 = 33;
const DEFAULT_FLOAT_VAL:      f64 = 27.62;
const DEFAULT_ANGLE_VAL:      f32 = 35.5;
const FLOAT_MIN:              f32 = 2.387;
const FLOAT_MAX:              f32 = 987.653;
const DEFAULT_POINT_VALS:     f64 = 50.0;
const KERNEL_SIZE:            i32 = 3;
const AEFX_AUDIO_DEFAULT_CURVE_TOLERANCE: f32 = 0.05;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    Amount,
    Color,
    FloatVal,
    Angle,
    Popup,
    Downsample,
    Point3D,
    Button,
    Layer,
}

struct Plugin { }

ae::define_plugin!(Plugin, (), Params);

impl Default for Plugin {
    fn default() -> Self {
        Self { }
    }
}

impl AdobePluginGlobal for Plugin {
    fn can_load(_host_name: &str, _host_version: &str) -> bool {
        true
    }

    fn params_setup(&self, params: &mut ae::Parameters<Params>, in_data: InData, _: OutData) -> Result<(), Error> {
        params.add(Params::Amount, "An obsolete slider", ae::SliderDef::setup(|f| {
            f.set_valid_min(PARAMARAMA_AMOUNT_MIN);
            f.set_valid_max(PARAMARAMA_AMOUNT_MAX);
            f.set_slider_min(PARAMARAMA_AMOUNT_MIN);
            f.set_slider_max(PARAMARAMA_AMOUNT_MAX);
            f.set_default(PARAMARAMA_AMOUNT_DFLT);
        }))?;

        params.add(Params::Color, "Color to mix", ae::ColorDef::setup(|f| {
            f.set_default(ae::Pixel8 {
                red: DEFAULT_RED,
                green: DEFAULT_GREEN,
                blue: DEFAULT_BLUE,
                alpha: 255
            });
        }))?;

        params.add(Params::FloatVal, "Some float value", ae::FloatSliderDef::setup(|f| {
            f.set_valid_min(FLOAT_MIN);
            f.set_valid_max(FLOAT_MAX);
            f.set_slider_min(FLOAT_MIN);
            f.set_slider_max(FLOAT_MAX);
            f.set_default(DEFAULT_FLOAT_VAL);
            f.set_curve_tolerance(AEFX_AUDIO_DEFAULT_CURVE_TOLERANCE);
            f.set_flags(ae::FSliderFlag::WANT_PHASE);
        }))?;

        params.add(Params::Downsample, "Some checkbox", ae::CheckBoxDef::setup(|f| {
            f.set_default(false);
            f.set_label("(with comment!)");
        }))?;

        params.add(Params::Angle, "An angle control", ae::AngleDef::setup(|f| {
            f.set_default(DEFAULT_ANGLE_VAL);
        }))?;

        params.add(Params::Popup, "Pop-up param", ae::PopupDef::setup(|f| {
            f.set_options(&["Make Slower", "Make Jaggy", "(-", "Plan A", "Plan B"]);
            f.set_default(1);
        }))?;

        // Only add 3D point and button where supported, starting in AE CS5.5
        if in_data.version().0 >= ae_sys::PF_AE105_PLUG_IN_VERSION as _ && in_data.version().1 >= ae_sys::PF_AE105_PLUG_IN_SUBVERS as _ {
            if in_data.application_id() != *b"FXTC" {
                params.add(Params::Point3D, "3D Point", ae::Point3DDef::setup(|f| {
                    f.set_default_x(DEFAULT_POINT_VALS);
                    f.set_default_y(DEFAULT_POINT_VALS);
                    f.set_default_z(DEFAULT_POINT_VALS);
                }))?;
            } else {
                // Add a placeholder for hosts that don't support 3D points
                params.add_with_flags(Params::Point3D, "3D Point", ae::ArbitraryDef::new(), ae::ParamFlag::empty(), ae::ParamUIFlags::NO_ECW_UI)?;
            }

            params.add_with_flags(Params::Button, "Button", ae::ButtonDef::setup(|f| {
                f.set_label("Button Label");
            }), ae::ParamFlag::SUPERVISE, ae::ParamUIFlags::empty())?;

            params.add(Params::Layer, "Layer", ae::LayerDef::new())?;
        }

        Ok(())
    }

    fn handle_command(&mut self, cmd: ae::Command, in_data: InData, mut out_data: OutData, params: &mut ae::Parameters<Params>) -> Result<(), ae::Error> {
        match cmd {
            ae::Command::About => {
                out_data.set_return_msg("Paramarama, v2.1,\rParameter Party!\rExercising all parameter types.\rCopyright 2007-2023 Adobe Inc.");
            }
            ae::Command::Render { in_layer, mut out_layer } => {
                let sharpen = (params.get(Params::Amount)?.as_slider()?.value() as f32 / 16.0).ceil();

                // If sharpen is set to 0, just copy the source to the destination
                if sharpen == 0.0 {
                    // Premiere Pro/Elements doesn't support WorldTransformSuite1, but it does support many of the callbacks in utils
                    if in_data.quality() == ae::Quality::Hi && in_data.application_id() != *b"PrMr" {
                        ae::pf::suites::WorldTransform::new()?.copy_hq(in_data.effect_ref(), in_layer, out_layer, None, None)?;
                    } else if in_data.application_id() != *b"PrMr" {
                        ae::pf::suites::WorldTransform::new()?.copy(in_data.effect_ref(), in_layer, out_layer, None, None)?;
                    } else {
                        out_layer.copy_from(&in_layer, None, None)?;
                    }
                } else {
                    let mut kernel_sum = 256.0 * 9.0;
                    let mut conv_kernel = [0u32; 9];
                    conv_kernel[4] = (sharpen * kernel_sum).round() as _;
                    kernel_sum	= (256.0 * 9.0 - conv_kernel[4] as f32) / 4.0;
                    let kernel_sum_rounded = kernel_sum.round() as _;
                    conv_kernel[1] = kernel_sum_rounded;
                    conv_kernel[3] = kernel_sum_rounded;
                    conv_kernel[5] = kernel_sum_rounded;
                    conv_kernel[7] = kernel_sum_rounded;
                    let kernel_ptr = conv_kernel.as_mut_ptr() as *mut _;

                    // Premiere Pro/Elements doesn't support WorldTransformSuite1, but it does support many of the callbacks in utils
                    if in_data.application_id() != *b"PrMr"	{
                        ae::pf::suites::WorldTransform::new()?.convolve(
                            in_data.effect_ref(),
                            &in_layer,
                            Some(in_data.extent_hint()),
                            ae::KernelFlags::TWO_D | ae::KernelFlags::CLAMP,
                            KERNEL_SIZE,
                            kernel_ptr,
                            kernel_ptr,
                            kernel_ptr,
                            kernel_ptr,
                            &mut out_layer
                        )?;
                    } else {
                        in_data.utils().convolve(
                            &in_layer,
                            Some(in_data.extent_hint()),
                            ae::KernelFlags::TWO_D | ae::KernelFlags::CLAMP,
                            KERNEL_SIZE,
                            kernel_ptr,
                            kernel_ptr,
                            kernel_ptr,
                            kernel_ptr,
                            &mut out_layer
                        )?;
                    }
                }
            }
            ae::Command::UserChangedParam { param_index } => {
                if params.type_for_index(param_index) == Params::Button {
                    out_data.set_return_msg("Paramarama button hit!");

                    if in_data.application_id() != *b"PrMr" {
                        out_data.set_out_flag(ae_sys::PF_OutFlag_DISPLAY_ERROR_MESSAGE, true);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}