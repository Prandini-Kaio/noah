use eframe::egui::{self, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use noah_workspace::{Geometry, Placement, Scene, session};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
const ACCENT: Color32 = Color32::from_rgb(92, 220, 196);
const MUTED: Color32 = Color32::from_rgb(143, 156, 176);
struct Desktop {
    scene: Scene,
    open_path: String,
    save_path: String,
    message: String,
    error: bool,
    dirty: bool,
    pending: Option<Receiver<noah_workspace::Result<Scene>>>,
    shown: usize,
    pending_session: bool,
    confirm_open: bool,
    confirm_close: bool,
    allow_close: bool,
}
impl Desktop {
    fn new(cc: &eframe::CreationContext<'_>, scene: Scene) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(20, 26, 36);
        style.visuals.window_fill = Color32::from_rgb(25, 33, 45);
        style.visuals.selection.bg_fill = Color32::from_rgb(32, 78, 82);
        style.visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(31, 41, 55);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 61, 76);
        style.spacing.item_spacing = Vec2::new(10., 10.);
        style.spacing.button_padding = Vec2::new(12., 7.);
        cc.egui_ctx.set_style(style);
        Self {
            scene,
            open_path: String::new(),
            save_path: "sessao-001.noah.json".into(),
            message: "Pronto. Abra uma fonte ou selecione um objeto.".into(),
            error: false,
            dirty: false,
            pending: None,
            shown: 0,
            pending_session: false,
            confirm_open: false,
            confirm_close: false,
            allow_close: false,
        }
    }
    fn notice(&mut self, result: noah_workspace::Result<()>, success: &str) {
        match result {
            Ok(()) => {
                self.message = success.into();
                self.error = false;
            }
            Err(e) => {
                self.message = e.to_string();
                self.error = true;
            }
        }
    }
    fn open(&mut self, as_session: bool) {
        let path = PathBuf::from(self.open_path.trim());
        if path.as_os_str().is_empty() {
            self.message = "Informe o caminho da fonte ou sessão.".into();
            self.error = true;
            return;
        }
        let (tx, rx) = mpsc::channel();
        let mut scene = self.scene.clone();
        std::thread::spawn(move || {
            let result = if as_session {
                session::load(&path)
            } else {
                scene.add_source(&path).map(|()| scene)
            };
            let _ = tx.send(result);
        });
        self.pending = Some(rx);
        self.pending_session = as_session;
        self.message = "Abrindo e verificando a origem...".into();
        self.error = false;
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .exact_height(74.)
            .show(ctx, |ui| {
                ui.add_space(13.);
                ui.horizontal(|ui| {
                    ui.add_space(12.);
                    ui.label(RichText::new("N").size(30.).strong().color(ACCENT));
                    ui.add_space(8.);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("NOAH 2.0").size(21.).strong());
                        ui.label(RichText::new("WORKSPACE  /  E3").size(10.).color(MUTED));
                    });
                    ui.add_space(28.);
                    ui.separator();
                    ui.add_space(16.);
                    ui.label(RichText::new("Explorar geometria").size(18.));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(14.);
                        ui.label(
                            RichText::new("LOCAL  •  SEM UPLOAD")
                                .size(10.)
                                .color(ACCENT),
                        );
                        if ui.button("Enquadrar tudo  [F]").clicked() {
                            let result = self.scene.fit(false);
                            self.notice(result, "Cena enquadrada.");
                            self.dirty = true;
                        }
                    });
                });
            });
    }
    fn left(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sources")
            .exact_width(252.)
            .resizable(false)
            .show(ctx, |ui| {
                if self.pending.is_some() {
                    ui.disable();
                }
                ui.add_space(16.);
                ui.label(RichText::new("CENA").color(MUTED).size(11.).strong());
                ui.label(RichText::new(format!("{} objetos", self.scene.items.len())).size(22.));
                ui.add_space(6.);
                ui.separator();
                ui.label(RichText::new("Abrir fonte local").strong());
                ui.add(
                    egui::TextEdit::singleline(&mut self.open_path)
                        .hint_text("Caminho do arquivo ou pasta")
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    let enabled = self.pending.is_none();
                    if ui
                        .add_enabled(enabled, egui::Button::new("Adicionar"))
                        .clicked()
                    {
                        self.open(false);
                    }
                    if ui
                        .add_enabled(enabled, egui::Button::new("Abrir sessão"))
                        .clicked()
                    {
                        if self.dirty {
                            self.confirm_open = true;
                        } else {
                            self.open(true);
                        }
                    }
                });
                ui.label(
                    RichText::new("PLY ASCII, OBJ triangular, pasta E1 ou run E2.")
                        .size(11.)
                        .color(MUTED),
                );
                ui.add_space(10.);
                ui.separator();
                ui.label(RichText::new("OBJETOS").size(10.).color(MUTED));
                egui::ScrollArea::vertical()
                    .max_height((ui.available_height() - 185.).max(100.))
                    .show(ui, |ui| {
                        let mut selected = self.scene.selected.clone();
                        for item in &mut self.scene.items {
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut item.visible, "").changed() {
                                    self.dirty = true;
                                }
                                let kind = match item.geometry.as_ref() {
                                    Geometry::Points(_) => "PTS",
                                    Geometry::Mesh(_) => "OBJ",
                                };
                                if ui
                                    .selectable_label(
                                        selected.as_ref() == Some(&item.id),
                                        format!("{kind}  {}", item.name),
                                    )
                                    .on_hover_text(&item.id)
                                    .clicked()
                                {
                                    selected = Some(item.id.clone());
                                    self.dirty = true;
                                }
                            });
                        }
                        self.scene.selected = selected;
                    });
                ui.add_space(12.);
                ui.separator();
                ui.label(RichText::new("SESSÃO").size(10.).color(MUTED));
                ui.add(
                    egui::TextEdit::singleline(&mut self.save_path)
                        .hint_text("Nova revisão .json")
                        .desired_width(f32::INFINITY),
                );
                if ui
                    .add_enabled(
                        self.pending.is_none(),
                        egui::Button::new("Salvar nova revisão"),
                    )
                    .clicked()
                {
                    let result = session::save(&self.scene, &PathBuf::from(self.save_path.trim()));
                    if result.is_ok() {
                        self.dirty = false;
                    }
                    self.notice(result, "Sessão salva. As fontes foram preservadas.");
                }
                ui.label(
                    RichText::new("Escolha um nome novo para cada revisão.")
                        .size(10.)
                        .color(MUTED),
                );
            });
    }
    fn right(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector").exact_width(274.).resizable(false).show(ctx,|ui|{
            if self.pending.is_some(){ui.disable();}
            egui::ScrollArea::vertical().id_salt("inspector_scroll").show(ui,|ui|{
            ui.add_space(16.);ui.label(RichText::new("PROPRIEDADES").size(11.).color(MUTED).strong());ui.add_space(8.);
            if let Some(index)=self.scene.items.iter().position(|i|Some(&i.id)==self.scene.selected.as_ref()){
                let item=&self.scene.items[index];ui.label(RichText::new(&item.name).size(19.).strong());
                let (kind,count)=match item.geometry.as_ref(){Geometry::Points(p)=>("Nuvem de pontos",p.positions().len()),Geometry::Mesh(m)=>("Malha triangular",m.triangles().len())};
                ui.label(RichText::new(kind).color(ACCENT));ui.label(format!("{count} elementos"));
                ui.add_space(8.);ui.separator();ui.label(format!("Unidade: {}",item.unit));ui.label(RichText::new(&item.frame).size(11.).color(MUTED));
                ui.add_space(12.);ui.label(RichText::new("Ajuste da sessão").strong());
                ui.label(RichText::new("Aplicado após a transformação de origem.").size(10.).color(MUTED));
                let mut p=item.placement.clone();let mut changed=false;
                ui.add_space(4.);ui.label("Translação  X / Y / Z");
                ui.horizontal(|ui|{for value in &mut p.translation{changed|=ui.add(egui::DragValue::new(value).speed(0.01).range(-1e12..=1e12).max_decimals(4)).changed();}});
                ui.label("Rotação  X / Y / Z  (graus)");
                ui.horizontal(|ui|{for angle in &mut p.angles{let mut degrees=angle.to_degrees();if ui.add(egui::DragValue::new(&mut degrees).speed(0.5).range(-3600.0..=3600.0).max_decimals(2)).changed(){*angle=degrees.to_radians();changed=true;}}});
                ui.label("Escala uniforme");changed|=ui.add(egui::DragValue::new(&mut p.scale).speed(0.01).range(1e-6..=1e6).max_decimals(4)).changed();
                if changed{let result=self.scene.set_placement(index,p);if result.is_ok(){self.dirty=true;}self.notice(result,"Ajuste aplicado à instância.");}
                ui.add_space(10.);ui.horizontal(|ui|{if ui.button("Restaurar").clicked(){let result=self.scene.set_placement(index,Placement::default());self.notice(result,"Ajuste restaurado.");self.dirty=true;}
                if ui.button("Enquadrar").clicked(){let result=self.scene.fit(true);self.notice(result,"Seleção enquadrada.");self.dirty=true;}});
                ui.add_space(14.);ui.separator();
                let source=&self.scene.sources[self.scene.items[index].source];ui.label(RichText::new("ORIGEM PRESERVADA").size(10.).color(ACCENT));ui.label(RichText::new(source.path.display().to_string()).size(10.).color(MUTED));
            }else{ui.label(RichText::new("Nenhum objeto selecionado").size(18.));ui.label(RichText::new("Clique na geometria ou escolha um objeto na lista.").color(MUTED));}
            ui.add_space(20.);ui.separator();ui.label(RichText::new("VISUALIZAÇÃO").size(10.).color(MUTED));
            if ui.checkbox(&mut self.scene.grid,"Grade de navegação").changed(){self.dirty=true;}
            if ui.checkbox(&mut self.scene.wireframe,"Malha em arame").changed(){self.dirty=true;}
            ui.label("Tamanho dos pontos");if ui.add(egui::Slider::new(&mut self.scene.point_size,1.0..=8.0).suffix(" px")).changed(){self.dirty=true;}
            ui.add_space(8.);ui.label(RichText::new("Exibição limitada a 50 mil pontos e 20 mil faces. A geometria original permanece completa.").size(10.).color(MUTED));
            });
        });
    }
    fn viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::from_rgb(12,18,27))).show(ctx,|ui|{
            let (response,painter)=ui.allocate_painter(ui.available_size(),Sense::click_and_drag());let rect=response.rect;let size=[rect.width() as f64,rect.height() as f64];
            if response.dragged(){let delta=ctx.input(|i|i.pointer.delta());let pan=response.dragged_by(egui::PointerButton::Secondary)||response.dragged_by(egui::PointerButton::Middle)||ctx.input(|i|i.modifiers.shift);if pan{self.scene.camera.pan([delta.x as f64,delta.y as f64],size[1]);}else{self.scene.camera.orbit([delta.x as f64,delta.y as f64]);}self.dirty=true;}
            if response.hovered(){let scroll=ctx.input(|i|i.smooth_scroll_delta.y);if scroll!=0.{self.scene.camera.zoom(scroll as f64);self.dirty=true;}}
            if !ctx.wants_keyboard_input()&&ctx.input(|i|i.key_pressed(egui::Key::F)){let result=self.scene.fit(false);self.notice(result,"Cena enquadrada.");self.dirty=true;}
            let to_screen=|p:[f64;2]|Pos2::new(rect.left()+p[0] as f32,rect.top()+p[1] as f32);
            if self.scene.grid{
                let center=self.scene.camera.target;let step=10f64.powf((self.scene.camera.distance/8.).log10().floor());let range=10;
                for i in -range..=range{for axis in 0..2{let mut a=center;let mut b=center;a[2]=0.;b[2]=0.;a[axis]+=i as f64*step;b[axis]+=i as f64*step;a[1-axis]-=range as f64*step;b[1-axis]+=range as f64*step;
                    if let (Some(a),Some(b))=(self.scene.camera.project(a,size),self.scene.camera.project(b,size)){painter.line_segment([to_screen([a[0],a[1]]),to_screen([b[0],b[1]])],Stroke::new(1.0_f32,Color32::from_rgb(28,39,53)));}
                }}
            }
            let primitives=noah_workspace::project(&self.scene,size);self.shown=primitives.len();
            if response.clicked() && self.pending.is_none()&& let Some(pos)=response.interact_pointer_pos(){self.scene.selected=noah_workspace::pick(&primitives,[(pos.x-rect.left()) as f64,(pos.y-rect.top()) as f64],7.).map(|i|self.scene.items[i].id.clone());self.dirty=true;}
            let mut mesh=egui::Mesh::default();let mut edges=Vec::new();
            for primitive in &primitives{
                if primitive.pixels.iter().flatten().any(|v|v.abs()>1e7){continue;}
                let selected=Some(&self.scene.items[primitive.item].id)==self.scene.selected.as_ref();
                let color=if selected{Color32::from_rgb(246,187,94)}else{Color32::from_rgb(primitive.color[0],primitive.color[1],primitive.color[2])};
                if primitive.pixels.len()==1{let pos=to_screen(primitive.pixels[0]);mesh.add_colored_rect(Rect::from_center_size(pos,Vec2::splat(self.scene.point_size)),color);}else{
                    let points:Vec<_>=primitive.pixels.iter().map(|&p|to_screen(p)).collect();
                    if !self.scene.wireframe{let start=mesh.vertices.len() as u32;for &p in &points{mesh.colored_vertex(p,color);}mesh.indices.extend([start,start+1,start+2]);}
                    let line=if selected{Color32::from_rgb(162,103,45)}else{Color32::from_rgb(31,73,87)};
                    for i in 0..3{edges.push(egui::Shape::line_segment([points[i],points[(i+1)%3]],Stroke::new(1.0_f32,line)));}
                }
            }
            painter.add(egui::Shape::mesh(mesh));painter.extend(edges);
            let top=rect.left_top()+Vec2::new(22.,22.);painter.text(top,egui::Align2::LEFT_TOP,"PERSPECTIVA",FontId::proportional(11.),MUTED);
            painter.text(top+Vec2::new(0.,20.),egui::Align2::LEFT_TOP,format!("{} primitivas na tela",self.shown),FontId::proportional(10.),Color32::from_rgb(93,112,133));
            if self.scene.items.is_empty(){let center=rect.center();painter.text(center-Vec2::new(0.,25.),egui::Align2::CENTER_CENTER,"Seu espaço de geometria",FontId::proportional(27.),Color32::from_rgb(190,204,220));painter.text(center+Vec2::new(0.,20.),egui::Align2::CENTER_CENTER,"Abra uma nuvem, malha, projeto ou run para começar.",FontId::proportional(13.),MUTED);}
            painter.text(rect.left_bottom()+Vec2::new(22.,-24.),egui::Align2::LEFT_BOTTOM,"Arrastar: órbita   •   Direito / Shift: pan   •   Roda: zoom   •   Clique: selecionar",FontId::proportional(10.),MUTED);
            painter.text(rect.right_bottom()-Vec2::new(20.,24.),egui::Align2::RIGHT_BOTTOM,"XYZ do arquivo",FontId::proportional(10.),ACCENT);
        });
    }
}
impl eframe::App for Desktop {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    match result {
                        Ok(scene) => {
                            self.scene = scene;
                            self.message = "Fonte aberta e verificada.".into();
                            self.error = false;
                            self.dirty = !self.pending_session;
                        }
                        Err(error) => {
                            self.message = error.to_string();
                            self.error = true;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.message = "A abertura foi interrompida; cena preservada.".into();
                    self.error = true;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(50));
                }
            }
        }
        self.header(ctx);
        egui::TopBottomPanel::bottom("status")
            .exact_height(34.)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.);
                    if self.pending.is_some() {
                        ui.spinner();
                    }
                    ui.label(RichText::new(&self.message).size(11.).color(if self.error {
                        Color32::from_rgb(250, 143, 123)
                    } else {
                        MUTED
                    }));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.);
                        ui.label(
                            RichText::new(if self.dirty {
                                "ALTERAÇÕES NÃO SALVAS"
                            } else {
                                "SESSÃO LOCAL"
                            })
                            .size(10.)
                            .color(ACCENT),
                        );
                    });
                });
            });
        self.left(ctx);
        self.right(ctx);
        self.viewport(ctx);
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        if self.confirm_close {
            egui::Window::new("Alterações não salvas")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Salve uma nova revisão para preservar os ajustes.");
                    ui.horizontal(|ui| {
                        if ui.button("Continuar editando").clicked() {
                            self.confirm_close = false;
                        }
                        if ui.button("Sair sem salvar").clicked() {
                            self.allow_close = true;
                            self.confirm_close = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
        if self.confirm_open {
            egui::Window::new("Abrir outra sessão?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Os ajustes não salvos serão descartados após a abertura.");
                    ui.horizontal(|ui| {
                        if ui.button("Cancelar").clicked() {
                            self.confirm_open = false;
                        }
                        if ui.button("Abrir e descartar ajustes").clicked() {
                            self.confirm_open = false;
                            self.open(true);
                        }
                    });
                });
        }
    }
}
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--help") {
        println!(
            "NOAH 2.0 Workspace E3\nUso: noah-desktop [fonte ... | sessao.json]\nFontes: pasta E1, run E2, PLY ASCII XYZ/RGB ou OBJ triangular.\nÓrbita: arrastar; pan: direito/Shift; zoom: roda; enquadrar: F."
        );
        return Ok(());
    }
    let mut scene = Scene::default();
    for path in std::env::args().skip(1) {
        let path = PathBuf::from(path);
        let result = if path.extension().is_some_and(|e| e == "json") {
            session::load(&path).map(|loaded| scene = loaded)
        } else {
            scene.add_source(&path)
        };
        if let Err(error) = result {
            eprintln!("NOAH Workspace: {error}");
            std::process::exit(1);
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320., 850.])
            .with_min_inner_size([1080., 700.])
            .with_position([0., 0.]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "NOAH 2.0 — Workspace",
        options,
        Box::new(move |cc| Ok(Box::new(Desktop::new(cc, scene)))),
    )
}
