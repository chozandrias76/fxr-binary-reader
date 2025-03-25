use crash_handler::{CrashContext, CrashEventResult, CrashHandler, make_crash_event};
use eldenring::cs::CSSfxImp;
use eldenring_util::{singleton::get_instance, system::wait_for_system_init};
use std::{error::Error, fs::File, sync::Mutex};
use tracing_panic::panic_hook;
pub const DLL_PROCESS_ATTACH: u32 = 1;

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
/// Entry point for the DLL. This function is called when the DLL is loaded or unloaded.
///
/// # Safety
/// This function is marked as `unsafe` because it interacts with low-level system APIs
/// and performs operations that require careful handling, such as spawning threads
/// and initializing global state. It must only be called by the system when the DLL
/// is loaded or unloaded.
pub unsafe extern "C" fn DllMain(_hmodule: usize, reason: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        if setup().is_err() {
            tracing::error!("Failed to set up logging and crash handler.");
            return false;
        }

        std::thread::spawn(|| {
            // Give the CRT init a bit of leeway
            wait_for_system_init(5000).expect("System initialization timed out");

            init().expect("Failed to initialize the program.");
        });
    }
    true
}

fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = File::create("./fxr_binary_reader.log")?;
    let subscriber = tracing_subscriber::fmt()
        .with_writer(Mutex::new(log_file))
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    std::panic::set_hook(Box::new(panic_hook));

    #[allow(unsafe_code)]
    let handler = CrashHandler::attach(unsafe {
        make_crash_event(move |context: &CrashContext| {
            tracing::error!(
                "Exception: {:x} at {:x}",
                context.exception_code,
                (*(*context.exception_pointers).ExceptionRecord).ExceptionAddress as usize
            );

            CrashEventResult::Handled(true)
        })
    })?;
    std::mem::forget(handler);

    Ok(())
}

fn init() -> Result<(), Box<dyn Error>> {
    #[allow(unsafe_code)]
    let cssfx_imp: Option<&mut CSSfxImp> = unsafe { get_instance::<CSSfxImp>() }?;
    if let Some(cssfx_imp) = cssfx_imp {
        let scene_ctrl: &eldenring::pointer::OwnedPtr<eldenring::gxffx::GXFfxSceneCtrl> =
            &cssfx_imp.scene_ctrl;
        let graphics_resource_manager: &&mut eldenring::gxffx::GXFfxGraphicsResourceManager =
            &scene_ctrl.graphics_resource_manager;
        let resource_container: &eldenring::pointer::OwnedPtr<
            eldenring::gxffx::FxrResourceContainer,
        > = &graphics_resource_manager.resource_container;
        let fxr_definitions = &resource_container.fxr_definitions;
        tracing::info!("Found FXR Definitions");
        for fxr_definition in fxr_definitions.iter() {
            tracing::info!("FXR Definition ID: {}", fxr_definition.id);

            if fxr_definition.id == 303161u32 {
                let fxr_wrapper = &fxr_definition.fxr_wrapper;
                let fxr_ptr = fxr_wrapper.fxr;
                tracing::info!("FXR Pointer: {:#x}", fxr_ptr);
                break;
            }
        }
    } else {
        tracing::error!("Failed to find CSSfxImp instance.");
        return Err("Failed to find CSSfxImp instance.".into());
    }
    Ok(())
}
