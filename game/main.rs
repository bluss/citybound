#![feature(custom_derive, conservative_impl_trait)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![allow(dead_code)]
// Enable this for memory tracking with Instruments/MacOS
// and for much better stacktraces for memory issues
//#![feature(alloc_system)]
//extern crate alloc_system;

extern crate ordered_float;
extern crate itertools;
extern crate rand;
extern crate fnv;
extern crate roaring;

extern crate compact;
#[macro_use]
extern crate compact_macros;
extern crate kay;
extern crate monet;
extern crate descartes;
extern crate stagemaster;
#[macro_use]
extern crate imgui;
#[macro_use]
extern crate serde_derive;

use stagemaster::environment::Environment;

pub const ENV: &'static Environment = &Environment {
    name: "Citybound",
    author: "ae play",
    version: "0.1.3",
};

mod core;
mod transport;
mod economy;

use compact::CVec;
use monet::GrouperID;
use transport::lane::{LaneID, TransferLaneID};
use transport::rendering::LaneRendererID;
use transport::planning::current_plan::CurrentPlanID;
use economy::households::family::FamilyID;
use economy::households::tasks::TaskEndSchedulerID;
use economy::buildings::rendering::BuildingRendererID;

fn main() {
    core::init::first_time_open_wiki_release_page();

    let mut system = kay::ActorSystem::new(
        core::init::create_init_callback(),
        core::init::networking_from_env_args(),
    );

    let world = &mut system.world();

    system.networking_connect();

    let simulatables = vec![
        LaneID::local_broadcast(world).into(),
        TransferLaneID::local_broadcast(world).into(),
        FamilyID::local_broadcast(world).into(),
        TaskEndSchedulerID::local_first(world).into(),
    ].into();
    let simulation = core::simulation::setup(&mut system, simulatables);

    let renderables: CVec<_> = vec![
        LaneRendererID::global_broadcast(world).into(),
        GrouperID::global_broadcast(world).into(),
        CurrentPlanID::global_broadcast(world).into(),
        BuildingRendererID::global_broadcast(&mut system.world()).into(),
    ].into();

    let machine_id = system.networking_machine_id();

    let (user_interface, renderer) = stagemaster::setup(
        &mut system,
        renderables,
        *ENV,
        core::init::build_window(machine_id),
        (0.6, 0.75, 0.4, 1.0)
    );

    transport::setup(&mut system, user_interface, renderer, simulation);
    economy::setup(&mut system, user_interface, simulation);

    core::init::print_version(user_interface, world);

    system.process_all_messages();

    let mut frame_counter = core::init::FrameCounter::new();

    loop {
        frame_counter.start_frame();
        frame_counter.print_fps(user_interface, world);

        core::init::print_instance_counts(&mut system, user_interface);
        core::init::print_network_turn(&mut system, user_interface);

        user_interface.process_events(world);

        system.process_all_messages();

        simulation.do_tick(world);

        system.process_all_messages();

        renderer.render(world);

        system.process_all_messages();

        system.networking_send_and_receive();
        system.process_all_messages();

        user_interface.start_frame(world);

        system.process_all_messages();

        system.networking_finish_turn();
    }
}
