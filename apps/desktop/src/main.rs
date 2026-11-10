use eframe::egui::{self, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use noah_workspace::{Geometry, Placement, Scene, session};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::Duration,
};
mod browser;
mod jobs;
use browser::{Browser, Purpose};
const ACCENT: Color32 = Color32::from_rgb(92, 220, 196);
const MUTED: Color32 = Color32::from_rgb(143, 156, 176);
#[derive(Clone)]
enum Replace {
    New,
    Session(PathBuf),
    Demo(PathBuf),
}
struct Desktop {
    scene: Scene,
    message: String,
    error: bool,
    dirty: bool,
    pending: Option<Receiver<noah_workspace::Result<Scene>>>,
    pending_session: bool,
    shown: usize,
    browser: Option<Browser>,
    last_dir: PathBuf,
    replace: Option<Replace>,
    confirm_close: bool,
    allow_close: bool,
    reconstruction: jobs::ReconstructionJob,
    show_reconstruction: bool,
    show_logs: bool,
    log_text: String,
    new_group_name: String,
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
            message: "Abra um workspace, importe peças ou explore a montagem de exemplo.".into(),
            error: false,
            dirty: false,
            pending: None,
            pending_session: false,
            shown: 0,
            browser: None,
            last_dir: std::env::current_dir().unwrap_or_default(),
            replace: None,
            confirm_close: false,
            allow_close: false,
            reconstruction: Default::default(),
            show_reconstruction: false,
            show_logs: false,
            log_text: String::new(),
            new_group_name: "Nova montagem".into(),
        }
    }
    fn notice(&mut self, result: noah_workspace::Result<()>, success: &str) {
        match result {
            Ok(()) => {
                self.message = success.into();
                self.error = false;
            }
            Err(error) => {
                self.message = error.to_string();
                self.error = true;
            }
        }
    }
    fn choose(&mut self, purpose: Purpose) {
        let mut browser = Browser::new(purpose, &self.last_dir);
        if matches!(purpose, Purpose::Export) {
            let mesh = self
                .scene
                .items
                .iter()
                .find(|i| Some(&i.id) == self.scene.selected.as_ref())
                .is_some_and(|i| matches!(i.geometry.as_ref(), Geometry::Mesh(_)));
            browser.export_extension(mesh);
        }
        self.browser = Some(browser);
    }
    fn load_source(&mut self, path: PathBuf) {
        let (tx, rx) = mpsc::channel();
        let mut scene = self.scene.clone();
        std::thread::spawn(move || {
            let _ = tx.send(scene.add_source(&path).map(|()| scene));
        });
        self.pending = Some(rx);
        self.pending_session = false;
        self.message = "Abrindo e verificando a origem...".into();
        self.error = false;
    }
    fn replace_scene(&mut self, action: Replace) {
        if matches!(action, Replace::New) {
            self.scene = Scene::default();
            self.dirty = false;
            self.message = "Novo workspace. Importe componentes para começar.".into();
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.pending_session = matches!(action, Replace::Session(_));
        std::thread::spawn(move || {
            let result = match action {
                Replace::Session(p) => session::load(&p),
                Replace::Demo(p) => noah_workspace::demo::create(&p),
                Replace::New => Ok(Scene::default()),
            };
            let _ = tx.send(result);
        });
        self.pending = Some(rx);
        self.message = "Preparando workspace...".into();
        self.error = false;
    }
    fn request_replace(&mut self, action: Replace) {
        if self.dirty {
            self.replace = Some(action);
        } else {
            self.replace_scene(action);
        }
    }
    fn browser_result(&mut self, purpose: Purpose, path: PathBuf) {
        self.last_dir = if path.is_dir() {
            path.clone()
        } else {
            path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        match purpose {
            Purpose::Import | Purpose::Project => self.load_source(path),
            Purpose::OpenSession => self.request_replace(Replace::Session(path)),
            Purpose::SaveSession => {
                let result = session::save(&self.scene, &path);
                if result.is_ok() {
                    self.dirty = false;
                }
                self.notice(result, "Workspace salvo. Fontes e montagem preservadas.");
            }
            Purpose::Demo => self.request_replace(Replace::Demo(path)),
            Purpose::Images => self.reconstruction.images = path,
            Purpose::Engine => self.reconstruction.engine = path,
            Purpose::Output => self.reconstruction.parent = path,
            Purpose::Export => {
                let result = self
                    .scene
                    .items
                    .iter()
                    .position(|i| Some(&i.id) == self.scene.selected.as_ref())
                    .ok_or_else(|| "Selecione uma peça, não uma montagem".into())
                    .and_then(|i| noah_workspace::export_item(&self.scene, i, &path));
                self.notice(
                    result,
                    "Peça montada e metadados exportados; explosão não aplicada.",
                );
            }
        }
    }
    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .exact_height(70.)
            .show(ctx, |ui| {
                ui.add_space(12.);
                ui.horizontal(|ui| {
                    ui.add_space(10.);
                    ui.label(RichText::new("N").size(30.).color(ACCENT));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("NOAH 2.0").size(20.).strong());
                        ui.label(
                            RichText::new("WORKSPACE / MONTAGENS")
                                .size(10.)
                                .color(MUTED),
                        );
                    });
                    ui.add_space(28.);
                    ui.label(RichText::new("Da captura ao conjunto").size(17.));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.);
                        ui.label(RichText::new("LOCAL • SEM UPLOAD").size(10.).color(ACCENT));
                        if ui.button("Enquadrar [F]").clicked() {
                            let result = self.scene.fit(false);
                            self.notice(result, "Cena enquadrada.");
                            self.dirty = true;
                        }
                        if ui.button("Montada").clicked() {
                            self.scene.explosion = 0.;
                            let _ = self.scene.fit(false);
                            self.dirty = true;
                        }
                        if ui.button("Explodida").clicked() {
                            let old = self.scene.explosion;
                            self.scene.explosion = 1.;
                            let result = self
                                .scene
                                .validate_world()
                                .and_then(|()| self.scene.fit(false));
                            if result.is_err() {
                                self.scene.explosion = old;
                            }
                            self.notice(
                                result,
                                "Vista explodida: geometria e montagem originais preservadas.",
                            );
                            self.dirty = true;
                        }
                    });
                });
            });
    }
    fn tree(&mut self, ui: &mut egui::Ui, parent: Option<&str>, depth: usize) {
        if depth > 64 {
            return;
        }
        let groups: Vec<_> = self
            .scene
            .assemblies
            .iter()
            .filter(|g| g.parent.as_deref() == parent)
            .map(|g| (g.id.clone(), g.name.clone()))
            .collect();
        for (id, name) in groups {
            let response = egui::CollapsingHeader::new(format!(
                "M  {}",
                name.chars().take(24).collect::<String>()
            ))
            .id_salt(&id)
            .default_open(true)
            .show(ui, |ui| self.tree(ui, Some(&id), depth + 1));
            if response.header_response.clicked() {
                self.scene.selected = Some(id);
                self.dirty = true;
            }
        }
        for item in &mut self.scene.items {
            if item.parent.as_deref() != parent {
                continue;
            }
            ui.horizontal(|ui| {
                if ui.checkbox(&mut item.visible, "").changed() {
                    self.dirty = true;
                }
                let kind = if matches!(item.geometry.as_ref(), Geometry::Mesh(_)) {
                    "OBJ"
                } else {
                    "PTS"
                };
                if ui
                    .selectable_label(
                        self.scene.selected.as_ref() == Some(&item.id),
                        format!("{kind}  {}", item.name),
                    )
                    .clicked()
                {
                    self.scene.selected = Some(item.id.clone());
                    self.dirty = true;
                }
            });
        }
    }
    fn left(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("workflow")
            .exact_width(270.)
            .resizable(false)
            .show(ctx, |ui| {
                if self.pending.is_some() {
                    ui.disable();
                }
                ui.add_space(12.);
                ui.label(RichText::new("SEU WORKSPACE").size(10.).color(MUTED));
                ui.label(
                    RichText::new(format!(
                        "{} peças · {} montagens",
                        self.scene.items.len(),
                        self.scene.assemblies.len()
                    ))
                    .size(19.),
                );
                ui.add_space(8.);
                egui::Grid::new("workspace_actions")
                    .num_columns(2)
                    .spacing([6., 7.])
                    .show(ui, |ui| {
                        if ui.button("Novo workspace").clicked() {
                            self.request_replace(Replace::New);
                        }
                        if ui.button("Abrir workspace").clicked() {
                            self.choose(Purpose::OpenSession);
                        }
                        ui.end_row();
                        if ui.button("Importar peças").clicked() {
                            self.choose(Purpose::Import);
                        }
                        if ui.button("Projeto / run").clicked() {
                            self.choose(Purpose::Project);
                        }
                        ui.end_row();
                        if ui.button("Salvar revisão").clicked() {
                            self.choose(Purpose::SaveSession);
                        }
                        if ui.button("Reconstruir fotos").clicked() {
                            self.show_reconstruction = true;
                        }
                        ui.end_row();
                    });
                ui.add_space(8.);
                if ui.button("Explorar montagem de exemplo").clicked() {
                    self.choose(Purpose::Demo);
                }
                ui.separator();
                ui.label(
                    RichText::new("ESTRUTURA DO CONJUNTO")
                        .size(10.)
                        .color(MUTED),
                );
                egui::ScrollArea::vertical()
                    .id_salt("scene_tree")
                    .max_height((ui.available_height() - 120.).max(150.))
                    .show(ui, |ui| self.tree(ui, None, 0));
                ui.separator();
                ui.text_edit_singleline(&mut self.new_group_name);
                if ui.button("+ Criar montagem").clicked() {
                    let result = self.scene.add_assembly(self.new_group_name.clone());
                    match result {
                        Ok(id) => {
                            self.scene.selected = Some(id);
                            self.dirty = true;
                        }
                        Err(e) => self.notice(Err(e), ""),
                    }
                }
                ui.label(
                    RichText::new(
                        "Escolha a montagem pai no inspetor para organizar cada componente.",
                    )
                    .size(10.)
                    .color(MUTED),
                );
            });
    }
    fn inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector").exact_width(286.).resizable(false).show(ctx,|ui|{if self.pending.is_some(){ui.disable();}egui::ScrollArea::vertical().id_salt("inspector_scroll").show(ui,|ui|{
   ui.add_space(12.);ui.label(RichText::new("SELEÇÃO E MONTAGEM").size(10.).color(MUTED));
   if let Some(id)=self.scene.selected.clone(){
    let item=self.scene.items.iter().position(|i|i.id==id);let group=self.scene.assemblies.iter().position(|g|g.id==id);
    let (mut placement,parent)=if let Some(index)=item{let i=&self.scene.items[index];ui.heading(&i.name);let count=match i.geometry.as_ref(){Geometry::Mesh(m)=>format!("{} triângulos",m.triangles().len()),Geometry::Points(p)=>format!("{} pontos",p.positions().len())};ui.label(RichText::new(count).color(ACCENT));ui.label(format!("Unidade: {}",i.unit));ui.label(RichText::new(&i.frame).size(10.).color(MUTED));(i.placement.clone(),i.parent.clone())}else if let Some(index)=group{let group=&mut self.scene.assemblies[index];ui.heading("Montagem");if ui.text_edit_singleline(&mut group.name).changed(){self.dirty=true;}let count=self.scene.items.iter().enumerate().filter(|(i,_)|self.scene.belongs_to(*i,&id)).count();ui.label(format!("{count} componentes descendentes"));(self.scene.assemblies[index].placement.clone(),self.scene.assemblies[index].parent.clone())}else{return;};
    ui.separator();let groups:Vec<_>=self.scene.assemblies.iter().filter(|g|g.id!=id).map(|g|(g.id.clone(),g.name.clone())).collect();let mut chosen=parent.clone();
    egui::ComboBox::from_id_salt("parent").selected_text(parent.as_ref().and_then(|id|groups.iter().find(|(g,_)|g==id).map(|(_,name)|name.as_str())).unwrap_or("Raiz do workspace")).width(240.).show_ui(ui,|ui|{ui.selectable_value(&mut chosen,None,"Raiz do workspace");for (id,name) in &groups{ui.selectable_value(&mut chosen,Some(id.clone()),name);}});
    if chosen!=parent{let result=self.scene.set_parent(&id,chosen);if result.is_ok(){self.dirty=true;}self.notice(result,"Montagem pai atualizada; valores locais preservados.");}
    ui.label(RichText::new("A transformação abaixo é local à montagem pai.").size(10.).color(MUTED));ui.add_space(8.);
    let mut changed=false;ui.label("Translação X / Y / Z");ui.horizontal(|ui|{for value in &mut placement.translation{changed|=ui.add(egui::DragValue::new(value).speed(0.01).range(-1e9..=1e9).max_decimals(3)).changed();}});
    ui.label("Rotação X / Y / Z (graus)");ui.horizontal(|ui|{for angle in &mut placement.angles{let mut degrees=angle.to_degrees();if ui.add(egui::DragValue::new(&mut degrees).speed(0.5).range(-3600.0..=3600.0).max_decimals(1)).changed(){*angle=degrees.to_radians();changed=true;}}});
    ui.label("Escala uniforme");changed|=ui.add(egui::DragValue::new(&mut placement.scale).speed(0.01).range(1e-6..=1e6)).changed();
    if changed{let result=if let Some(index)=item{self.scene.set_placement(index,placement)}else{self.scene.set_group_placement(&id,placement)};if result.is_ok(){self.dirty=true;}self.notice(result,"Transformação aplicada à seleção.");}
    ui.horizontal(|ui|{if ui.button("Restaurar ajuste").clicked(){let result=if let Some(index)=item{self.scene.set_placement(index,Placement::default())}else{self.scene.set_group_placement(&id,Placement::default())};self.notice(result,"Ajuste local restaurado.");self.dirty=true;}
if ui.button("Enquadrar").clicked(){let r=self.scene.fit(true);self.notice(r,"Seleção enquadrada.");self.dirty=true;}});
    if let Some(index)=item{ui.add_space(5.);if ui.button("Exportar peça montada").clicked(){self.choose(Purpose::Export);}let source=&self.scene.sources[self.scene.items[index].source];ui.label(RichText::new(source.path.display().to_string()).size(9.).color(MUTED));}
   }else{ui.heading("Explore seu conjunto");ui.label("Selecione uma peça ou montagem na cena ou na estrutura à esquerda.");}
   ui.add_space(12.);ui.separator();ui.label(RichText::new("VISTA EXPLODIDA").color(ACCENT).strong());let old=(self.scene.explosion,self.scene.explosion_distance);
   let mut changed=ui.add(egui::Slider::new(&mut self.scene.explosion,0.0..=1.0).text("Intensidade")).changed();ui.label("Afastamento em unidades da cena");changed|=ui.add(egui::DragValue::new(&mut self.scene.explosion_distance).speed(0.05).range(0.0..=1e6)).changed();
   if changed{if let Err(e)=self.scene.validate_world(){self.scene.explosion=old.0;self.scene.explosion_distance=old.1;self.notice(Err(e),"");}else{self.dirty=true;}}
   ui.label(RichText::new("A explosão é apenas visual. Montada restaura as posições do conjunto.").size(10.).color(MUTED));ui.add_space(10.);ui.separator();ui.label("VISUALIZAÇÃO");
   if ui.checkbox(&mut self.scene.grid,"Grade de navegação").changed(){self.dirty=true;}
if ui.checkbox(&mut self.scene.wireframe,"Malha em arame").changed(){self.dirty=true;}
if ui.add(egui::Slider::new(&mut self.scene.point_size,1.0..=8.0).text("Pontos / px")).changed(){self.dirty=true;}
   ui.label(RichText::new("Até 50 mil pontos / 20 mil faces na tela. Oclusão aproximada nesta versão.").size(10.).color(MUTED));
  });});
    }
    fn viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::from_rgb(12,18,27))).show(ctx,|ui|{
            let (response,painter)=ui.allocate_painter(ui.available_size(),Sense::click_and_drag());let rect=response.rect;let size=[rect.width() as f64,rect.height() as f64];
            if response.dragged() && self.pending.is_none(){let delta=ctx.input(|i|i.pointer.delta());let pan=response.dragged_by(egui::PointerButton::Secondary)||response.dragged_by(egui::PointerButton::Middle)||ctx.input(|i|i.modifiers.shift);if pan{self.scene.camera.pan([delta.x as f64,delta.y as f64],size[1]);}else{self.scene.camera.orbit([delta.x as f64,delta.y as f64]);}self.dirty=true;}
            if response.hovered() && self.pending.is_none(){let scroll=ctx.input(|i|i.smooth_scroll_delta.y);if scroll!=0.{self.scene.camera.zoom(scroll as f64);self.dirty=true;}}
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
                let selected=Some(&self.scene.items[primitive.item].id)==self.scene.selected.as_ref()||self.scene.selected.as_ref().is_some_and(|id|self.scene.belongs_to(primitive.item,id));
                let color=if selected{Color32::from_rgb(246,187,94)}else{Color32::from_rgb(primitive.color[0],primitive.color[1],primitive.color[2])};
                if primitive.pixels.len()==1{let pos=to_screen(primitive.pixels[0]);mesh.add_colored_rect(Rect::from_center_size(pos,Vec2::splat(self.scene.point_size)),color);}else{
                    let points:Vec<_>=primitive.pixels.iter().map(|&p|to_screen(p)).collect();
                    if !self.scene.wireframe{let start=mesh.vertices.len() as u32;for &p in &points{mesh.colored_vertex(p,color);}mesh.indices.extend([start,start+1,start+2]);}
                    let line=if selected{Color32::from_rgb(162,103,45)}else{Color32::from_rgb(31,73,87)};
                    if self.scene.wireframe {for i in 0..3{edges.push(egui::Shape::line_segment([points[i],points[(i+1)%3]],Stroke::new(1.0_f32,line)));}}
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
impl Desktop {
    fn reconstruction_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_reconstruction;
        egui::Window::new("Fotogrametria local").open(&mut open).default_width(660.).collapsible(false).show(ctx,|ui|{
   ui.label(RichText::new("Fotografias / Câmeras / Nuvem de pontos").size(18.).strong());ui.label("Escolha um COLMAP 3.11.x instalado localmente. Seus dados permanecem nesta máquina.");ui.separator();
   let busy=self.reconstruction.worker.is_some();
   ui.add_enabled_ui(!busy,|ui|{
    ui.horizontal(|ui|{if ui.button("1. Escolher fotografias").clicked(){self.choose(Purpose::Images);}ui.label(self.reconstruction.images.display().to_string());});
    ui.horizontal(|ui|{if ui.button("2. Escolher COLMAP").clicked(){self.choose(Purpose::Engine);}ui.label(self.reconstruction.engine.display().to_string());});
    ui.horizontal(|ui|{if ui.button("3. Pasta de resultados").clicked(){self.choose(Purpose::Output);}ui.label(self.reconstruction.parent.display().to_string());});
    ui.horizontal(|ui|{ui.label("Nome da reconstrução");ui.text_edit_singleline(&mut self.reconstruction.name);});
    ui.checkbox(&mut self.reconstruction.dense,"Incluir densificação (requer CUDA e GPU compatível)");ui.horizontal(|ui|{ui.label("Tempo máximo por etapa / segundos");ui.add(egui::DragValue::new(&mut self.reconstruction.timeout).range(1..=86400));});
    if ui.button(RichText::new("Iniciar reconstrução").color(ACCENT)).clicked(){let result=self.reconstruction.start();self.notice(result,"Reconstrução iniciada. Você pode continuar explorando a cena.");}
   });
   ui.separator();if busy{ui.horizontal(|ui|{ui.spinner();ui.label("Processamento em andamento");if ui.button("Cancelar processamento").clicked(){let result=self.reconstruction.cancel();self.notice(result,"Cancelamento solicitado.");}});}
   ui.label(&self.reconstruction.status);
   if let Some(manifest)=&self.reconstruction.manifest{ui.label(format!("Estado: {} · {} imagens",jobs::status_label(&manifest.status),manifest.inputs.len()));for step in &manifest.steps{ui.horizontal(|ui|{ui.label(jobs::status_label(&step.status));ui.label(jobs::step_label(&step.name));});}}
   ui.horizontal(|ui|{
    if ui.button("Ver logs e diagnóstico").clicked(){match self.reconstruction.logs(){Ok(text)=>{self.log_text=text;self.show_logs=true;},Err(e)=>self.notice(Err(e),"")}}
    let output=self.reconstruction.output.clone();let available=!busy&&output.as_ref().is_some_and(|p|p.join("exports/sparse.ply").is_file());
    if ui.add_enabled(available,egui::Button::new("Abrir nuvem no workspace")).clicked()&&let Some(root)=output{self.load_source(root);}
   });ui.label(RichText::new("Falhas e resultados parciais são preservados. As fotos não são alteradas.").size(11.).color(MUTED));
  });
        self.show_reconstruction = open;
    }
}
impl eframe::App for Desktop {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.reconstruction.refresh();
        if self.reconstruction.worker.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    match result {
                        Ok(scene) => {
                            self.scene = scene;
                            self.dirty = !self.pending_session;
                            self.notice(Ok(()), "Workspace aberto e verificado.");
                        }
                        Err(e) => self.notice(Err(e), ""),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.notice(
                        Err("Abertura interrompida; cena anterior preservada".into()),
                        "",
                    );
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(50))
                }
            }
        }
        self.header(ctx);
        egui::TopBottomPanel::bottom("status")
            .exact_height(36.)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(10.);
                    if self.pending.is_some() {
                        ui.spinner();
                    }
                    ui.label(RichText::new(&self.message).size(11.).color(if self.error {
                        Color32::LIGHT_RED
                    } else {
                        MUTED
                    }))
                    .on_hover_text(&self.message);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.);
                        ui.label(
                            RichText::new(if self.dirty {
                                "ALTERAÇÕES NÃO SALVAS"
                            } else {
                                "WORKSPACE LOCAL"
                            })
                            .size(10.)
                            .color(ACCENT),
                        );
                    });
                });
            });
        self.left(ctx);
        self.inspector(ctx);
        self.viewport(ctx);
        if self.show_reconstruction {
            self.reconstruction_window(ctx);
        }
        if self.show_logs {
            egui::Window::new("Logs locais da reconstrução")
                .open(&mut self.show_logs)
                .default_size([760., 450.])
                .show(ctx, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log_text)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(720.)
                                .interactive(false),
                        );
                    });
                });
        }
        let mut choice = None;
        if let Some(browser) = &mut self.browser {
            let purpose = browser.purpose;
            if let Some(path) = browser.show(ctx) {
                choice = Some((purpose, path));
            }
            if browser.closed {
                self.browser = None;
            }
        }
        if let Some((purpose, path)) = choice {
            self.browser_result(purpose, path);
        }
        if let Some(action) = self.replace.clone() {
            egui::Window::new("Substituir workspace?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Existem ajustes não salvos. Salve uma revisão antes de continuar.");
                    ui.horizontal(|ui| {
                        if ui.button("Continuar editando").clicked() {
                            self.replace = None;
                        }
                        if ui.button("Descartar e continuar").clicked() {
                            self.replace = None;
                            self.replace_scene(action);
                        }
                    });
                });
        }
        if ctx.input(|i| i.viewport().close_requested())
            && (self.dirty || self.reconstruction.worker.is_some())
            && !self.allow_close
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        if self.confirm_close {
            egui::Window::new("Antes de sair")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    let busy = self.reconstruction.worker.is_some();
                    ui.label(if busy {
                        "Há uma reconstrução em andamento. Cancele e aguarde o encerramento."
                    } else {
                        "Existem ajustes não salvos. Salve uma revisão para preservá-los."
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Continuar no workspace").clicked() {
                            self.confirm_close = false;
                        }
                        if busy {
                            if ui.button("Cancelar reconstrução").clicked() {
                                let r = self.reconstruction.cancel();
                                self.notice(r, "Cancelamento solicitado.");
                            }
                        } else if ui.button("Sair sem salvar").clicked() {
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
    }
}
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--help") {
        println!(
            "NOAH 2.0 — Workspace e montagens\nInicie sem argumentos para operar pela interface.\nOpcional: noah-desktop [fonte ... | workspace.json]"
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
            eprintln!("NOAH: {error}");
            std::process::exit(1);
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1420., 900.])
            .with_min_inner_size([1180., 760.])
            .with_position([0., 0.]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "NOAH 2.0 — Workspace e montagens",
        options,
        Box::new(move |cc| Ok(Box::new(Desktop::new(cc, scene)))),
    )
}
