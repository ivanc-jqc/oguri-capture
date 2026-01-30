use audio::Audio;
use pipewire::channel;
use pipewire::context::ContextRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::registry::{self, RegistryRc};
use std::error::Error;
use std::os::fd::OwnedFd;
use video::Video;
pub struct PipewireManager {
    main_loop: MainLoopRc,
    context: ContextRc,
    registry: RegistryRc,
    registry_listener: registry::Listener,

    pub audio: Audio,
    pub video: Video,
}
pub struct TERMINATE;

impl PipewireManager {
    pub fn new(fd: OwnedFd, pw_rx: channel::Receiver<TERMINATE>) -> Self {
        let mainloop = MainLoopRc::new(None).unwrap();
        let temp = mainloop.clone();
        let _receiver = pw_rx.attach(temp.loop_(), {
            let mainloop = mainloop.clone();
            move |_| mainloop.quit()
        });
        let context = ContextRc::new(&mainloop, None).unwrap();
        let audio_core = context.connect_rc(None).unwrap();
        let video_core = context.connect_fd_rc(fd, None).unwrap();
        let registry = audio_core.get_registry_rc().unwrap();
        let reg_listener = registry
            .add_listener_local()
            .global(|global| {
                // dbg!(global);
            })
            .register();
        Self {
            main_loop: mainloop,
            context: context,
            registry: registry,
            registry_listener: reg_listener,
            audio: Audio::new(audio_core),
            video: Video::new(video_core),
        }
    }
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.main_loop.run();
        Ok(())
    }
}

mod audio;
mod video;
