use audio::Audio;
use pipewire::channel::{self, AttachedReceiver};
use pipewire::context::ContextRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::registry::{self, RegistryRc};
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::os::fd::OwnedFd;
use std::time::Duration;
use video::Video;

use crate::main;
pub struct PipewireManager<'a> {
    main_loop: MainLoopRc,
    context: ContextRc,
    registry: RegistryRc,
    registry_listener: registry::Listener,
    attached_receiver: Option<AttachedReceiver<'a,TERMINATE>>,
    pub audio: Audio,
    pub video: Video,
}
pub enum StreamCommand {
    Terminate,
    Disconnect,
}
pub struct TERMINATE;

impl<'a> PipewireManager<'a> {
    pub fn new(fd: OwnedFd, node_id: u32) -> Self {
        let mainloop = MainLoopRc::new(None).unwrap();
        
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
            attached_receiver: None,
            audio: Audio::new(audio_core),
            video: Video::new(video_core, node_id),
        }
    }
    // pub fn attach_receiver(&mut'a self,pw_rx: channel::Receiver<TERMINATE>){
    //     let receiver = pw_rx.attach(self.main_loop.loop_(), {
    //         let main_loop = self.main_loop.clone();
    //         move |_| main_loop.quit()
    //     });
    //     self.attached_receiver = Some(receiver);
    // }
    pub fn run(&self, pw_rx: channel::Receiver<StreamCommand>) -> Result<(), Box<dyn Error>> {
        let _receiver = pw_rx.attach(self.main_loop.loop_(), {
            let main_loop = self.main_loop.clone();
            let stream = self.video.video_streams[0].clone();
            move |cmd| match cmd {
                StreamCommand::Disconnect => {
                    stream.set_active(false).unwrap();
                }
                StreamCommand::Terminate => {
                    // dbg!("hi from receiver!");
                    main_loop.quit();
                }
            }
        });
        self.main_loop.run();
        Ok(())
    }
}

mod audio;
mod video;
