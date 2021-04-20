#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(min_specialization)]

use imgui_baseview::{HiDpiMode, RenderSettings};
use std::sync::Arc;

use baseplug::{Plugin, ProcessContext, WindowOpenResult};
use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use imgui::*;
use imgui_baseview::{ImguiWindow, Settings};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use serde::{Deserialize, Serialize};
use vst::util::AtomicFloat;

const WINDOW_WIDTH: i16 = 1024;
const WINDOW_HEIGHT: i16 = 512;

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct GainModel {
        #[model(min = -90.0, max = 3.0)]
        #[parameter(name = "gain", unit = "Decibels",
            gradient = "Power(0.15)")]
        gain: f32,
    }
}

impl Default for GainModel {
    fn default() -> Self {
        Self {
            // "gain" is converted from dB to coefficient in the parameter handling code,
            // so in the model here it's a coeff.
            // -0dB == 1.0
            gain: 1.0,
        }
    }
}
struct GainUIParameters {
    // The plugin's state consists of a single parameter: gain.
    gain: AtomicFloat,
}

struct Gain {}

fn setup_logging() {
    let log_folder = ::dirs::home_dir().unwrap().join("tmp");

    let _ = ::std::fs::create_dir(log_folder.clone());

    let log_file =
        ::std::fs::File::create(log_folder.join("IMGUIBaseplugBaseviewTest.log")).unwrap();

    let log_config = ::simplelog::ConfigBuilder::new()
        .set_time_to_local(true)
        .build();

    let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file);

    ::log_panics::init();

    ::log::info!("init");
}

impl Plugin for Gain {
    const NAME: &'static str = "imgui-baseplug gain";
    const PRODUCT: &'static str = "imgui-baseplug gain";
    const VENDOR: &'static str = "DGriffin";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = GainModel;

    #[inline]
    fn new(_sample_rate: f32, _model: &GainModel) -> Self {
        setup_logging();
        Self {}
    }

    #[inline]
    fn process(&mut self, model: &GainModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            output[0][i] = input[0][i] * model.gain[i];
            output[1][i] = input[1][i] * model.gain[i];
        }
    }
}

impl baseplug::PluginUI for Gain {
    type Handle = Arc<GainUIParameters>;

    fn ui_size() -> (i16, i16) {
        (WINDOW_WIDTH, WINDOW_HEIGHT)
    }

    fn ui_open(parent: &impl HasRawWindowHandle) -> WindowOpenResult<Self::Handle> {
        let settings = Settings {
            window: WindowOpenOptions {
                title: String::from("imgui-baseview demo window"),
                size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            clear_color: (0.0, 0.0, 0.0),
            hidpi_mode: HiDpiMode::Default,
            render_settings: RenderSettings::default(),
        };

        let params = Arc::new(GainUIParameters {
            gain: AtomicFloat::new(0.0),
        });

        ImguiWindow::open_parented(
            parent,
            settings,
            params.clone(),
            |_context: &mut Context, _state: &mut Arc<GainUIParameters>| {},
            |run: &mut bool, ui: &Ui, state: &mut Arc<GainUIParameters>| {
                ui.show_demo_window(run);
                let w = Window::new(im_str!("Example 1: Basic sliders"))
                    .size([200.0, 200.0], Condition::Appearing)
                    .position([20.0, 20.0], Condition::Appearing);
                w.build(&ui, || {
                    let mut val = state.gain.get();
                    if Slider::new(im_str!("Gain"))
                        .range(-90.0..=3.0)
                        .build(&ui, &mut val)
                    {
                        state.gain.set(val)
                    }
                });
            },
        );

        Ok(params.clone())
    }

    fn ui_param_notify(
        handle: &Self::Handle,
        param: &'static baseplug::Param<Self, <Self::Model as baseplug::Model<Self>>::Smooth>,
        val: f32,
    ) {
        // TODO: doesn't work
        match param.get_name() {
            "gain" => handle.gain.set(val),
            _ => {}
        }
    }

    fn ui_close(mut _handle: Self::Handle) {
        //handle.close_window().unwrap();
    }
}

struct VstParent(*mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}

baseplug::vst2!(Gain, b"tRbE");
