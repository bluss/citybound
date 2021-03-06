use descartes::Circle;
use compact::CVec;
use kay::{ActorSystem, World, External};
use monet::{RendererID, Renderable, RenderableID, MSG_Renderable_setup_in_scene,
            MSG_Renderable_render_to_scene, GrouperID, GrouperIndividualID};
use stagemaster::geometry::AnyShape;
use stagemaster::{UserInterfaceID, Event3d, Interactable3d, Interactable3dID, Interactable2d,
                  Interactable2dID, MSG_Interactable3d_on_event, MSG_Interactable2d_draw_ui_2d};
use imgui::ImGuiSetCond_FirstUseEver;

use super::{Building, BuildingID};
use economy::households::HouseholdID;

mod architecture;

#[derive(Compact, Clone)]
pub struct BuildingInspector {
    id: BuildingInspectorID,
    user_interface: UserInterfaceID,
    current_building: Option<BuildingID>,
    current_households: CVec<HouseholdID>,
    households_todo: CVec<HouseholdID>,
    return_ui_to: Option<UserInterfaceID>,
}

impl BuildingInspector {
    pub fn spawn(
        id: BuildingInspectorID,
        user_interface: UserInterfaceID,
        _: &mut World,
    ) -> BuildingInspector {
        BuildingInspector {
            id,
            user_interface,
            current_building: None,
            current_households: CVec::new(),
            households_todo: CVec::new(),
            return_ui_to: None,
        }
    }

    pub fn set_inspected_building(
        &mut self,
        building: BuildingID,
        households: &CVec<HouseholdID>,
        world: &mut World,
    ) {
        self.current_building = Some(building);
        self.current_households = households.clone();
        self.households_todo.clear();
        self.user_interface.add_2d(self.id.into(), world);
    }

    pub fn ui_drawn(&mut self, imgui_ui: &External<::imgui::Ui<'static>>, world: &mut World) {
        let ui = imgui_ui.steal();

        if let Some(household) = self.households_todo.pop() {
            household.inspect(ui, self.id, world);
        } else {
            self.return_ui_to
                .expect("Should have return to set for UI")
                .ui_drawn(ui, world);
        }
    }
}

impl Interactable2d for BuildingInspector {
    fn draw_ui_2d(
        &mut self,
        imgui_ui: &External<::imgui::Ui<'static>>,
        return_to: UserInterfaceID,
        world: &mut World,
    ) {
        let ui = imgui_ui.steal();
        self.return_ui_to = Some(return_to);

        let new_building = if let Some(building) = self.current_building {
            let mut opened = true;

            ui.window(im_str!("Building"))
                .size((200.0, 300.0), ImGuiSetCond_FirstUseEver)
                .collapsible(false)
                .opened(&mut opened)
                .build(|| {
                    ui.text(im_str!("Building ID: {:?}", building._raw_id));
                    ui.text(im_str!(
                        "# of households: {}",
                        self.current_households.len()
                    ))
                });

            self.households_todo = self.current_households.clone();
            self.return_ui_to = Some(return_to);
            self.id.ui_drawn(ui, world);

            if opened { Some(building) } else { None }
        } else {
            return_to.ui_drawn(ui, world);
            None
        };

        self.current_building = new_building;
    }
}

impl Interactable3d for Building {
    fn on_event(&mut self, event: Event3d, world: &mut World) {
        if let Event3d::DragFinished { .. } = event {
            BuildingInspectorID::local_first(world)
                .set_inspected_building(self.id, self.households.clone(), world);
        };
    }
}

#[derive(Compact, Clone)]
pub struct BuildingRenderer {
    id: BuildingRendererID,
    wall_grouper: GrouperID,
    flat_roof_grouper: GrouperID,
    brick_roof_grouper: GrouperID,
}

impl BuildingRenderer {
    pub fn spawn(id: BuildingRendererID, world: &mut World) -> BuildingRenderer {
        BuildingRenderer {
            id,
            wall_grouper: GrouperID::spawn([0.95, 0.95, 0.95], 5000, false, world),
            flat_roof_grouper: GrouperID::spawn([0.5, 0.5, 0.5], 5100, false, world),
            brick_roof_grouper: GrouperID::spawn([0.8, 0.5, 0.2], 5200, false, world),
        }
    }

    pub fn add_geometry(
        &mut self,
        id: BuildingID,
        geometry: &architecture::BuildingGeometry,
        world: &mut World,
    ) {
        // TODO: ugly: Building is not really a GrouperIndividual
        self.wall_grouper.add_frozen(
            GrouperIndividualID { _raw_id: id._raw_id },
            geometry.wall.clone(),
            world,
        );
        self.flat_roof_grouper.add_frozen(
            GrouperIndividualID { _raw_id: id._raw_id },
            geometry.flat_roof.clone(),
            world,
        );
        self.brick_roof_grouper.add_frozen(
            GrouperIndividualID { _raw_id: id._raw_id },
            geometry.brick_roof.clone(),
            world,
        );
    }
}

use economy::households::grocery_shop::GroceryShopID;

impl Renderable for BuildingRenderer {
    fn setup_in_scene(&mut self, renderer_id: RendererID, scene_id: usize, world: &mut World) {
        // let band_path = CPath::new(vec![
        //     Segment::arc_with_direction(
        //         P2::new(5.0, 0.0),
        //         V2::new(0.0, 1.0),
        //         P2::new(-5.0, 0.0)
        //     ),
        //     Segment::arc_with_direction(
        //         P2::new(-5.0, 0.0),
        //         V2::new(0.0, -1.0),
        //         P2::new(5.0, 0.0)
        //     ),
        // ]);
        // let building_circle = band_to_geometry(&Band::new(band_path, 2.0), 0.0);
        // renderer_id.add_batch(scene_id, 11_111, building_circle, world);
        Into::<RenderableID>::into(self.wall_grouper).setup_in_scene(renderer_id, scene_id, world);
        Into::<RenderableID>::into(self.flat_roof_grouper)
            .setup_in_scene(renderer_id, scene_id, world);
        Into::<RenderableID>::into(self.brick_roof_grouper)
            .setup_in_scene(renderer_id, scene_id, world);
    }

    fn render_to_scene(
        &mut self,
        renderer_id: RendererID,
        scene_id: usize,
        frame: usize,
        world: &mut World,
    ) {
        // let renderable_buildings: RenderableID = BuildingID::local_broadcast(world).into();
        // renderable_buildings.render_to_scene(renderer_id, scene_id, frame, world);
        Into::<RenderableID>::into(self.wall_grouper)
            .render_to_scene(renderer_id, scene_id, frame, world);
        Into::<RenderableID>::into(self.flat_roof_grouper)
            .render_to_scene(renderer_id, scene_id, frame, world);
        Into::<RenderableID>::into(self.brick_roof_grouper)
            .render_to_scene(renderer_id, scene_id, frame, world);
    }
}

// impl Renderable for Building {
//     fn setup_in_scene(&mut self, _renderer_id: RendererID, _scene_id: usize, _: &mut World) {}

//     fn render_to_scene(
//         &mut self,
//         renderer_id: RendererID,
//         scene_id: usize,
//         frame: usize,
//         world: &mut World,
//     ) {
//         // TODO: this is super hacky
//         let is_shop = self.households[0]._raw_id.local_broadcast() ==
//             GroceryShopID::local_broadcast(world)._raw_id;
//         renderer_id.add_instance(
//             scene_id,
//             11_111,
//             frame,
//             Instance {
//                 instance_position: [self.lot.position.x, self.lot.position.y, 0.0],
//                 instance_direction: [1.0, 0.0],
//                 instance_color: if is_shop {
//                     [0.2, 0.2, 0.8]
//                 } else {
//                     [0.3, 0.8, 0.0]
//                 },
//             },
//             world,
//         );
//     }
// }

pub fn setup(system: &mut ActorSystem, user_interface: UserInterfaceID) -> BuildingRendererID {
    system.register::<BuildingInspector>();
    system.register::<BuildingRenderer>();
    auto_setup(system);

    BuildingInspectorID::spawn(user_interface, &mut system.world());

    BuildingRendererID::spawn(&mut system.world())
}

use rand::{XorShiftRng, SeedableRng};

pub fn on_add(building: &Building, world: &mut World) {
    // TODO: not sure if correct
    UserInterfaceID::local_first(world).add(
        building.id.into(),
        AnyShape::Circle(Circle {
            center: building.lot.position,
            radius: 5.0,
        }),
        10,
        world,
    );

    // TODO: this is super hacky
    let is_shop = building.households[0]._raw_id.local_broadcast() ==
        GroceryShopID::local_broadcast(world)._raw_id;
    BuildingRendererID::local_first(world).add_geometry(
        building.id,
        architecture::build_building(
            &building.lot,
            is_shop,
            &mut XorShiftRng::from_seed(
                [
                    building.id._raw_id.instance_id * 1000,
                    u32::from(building.id._raw_id.machine),
                    building.id._raw_id.instance_id,
                    42,
                ],
            ),
        ),
        world,
    )
}

mod kay_auto;
pub use self::kay_auto::*;
