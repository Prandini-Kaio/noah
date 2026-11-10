use eframe::egui::{self, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use noah_workspace::measurement::{self, Anchor, LengthUnit};
use noah_workspace::{Geometry, Placement, Scene, session};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::Duration,
};
mod animation;
mod blast;
mod browser;
mod jobs;
mod shell;
mod theme;
mod viewport_tools;
use browser::{Browser, Purpose};
use noah_workspace::{camera::Projection, view::ThemeMode};
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
    show_measurements: bool,
    measure_mode: bool,
    anchors: Vec<Anchor>,
    measure_name: String,
    known_distance: f64,
    length_unit: LengthUnit,
    scale_provenance: String,
    transition: Option<animation::ExplosionTransition>,
    show_visual: bool,
    hover_anchor: Option<Anchor>,
    hover_pixel: Option<[f64; 2]>,
    hover_updated: std::time::Instant,
    exact_a: usize,
    exact_b: usize,
    blast: Option<blast::Blast>,
    blast_mode: bool,
    blast_selected: bool,
    blast_strength: f64,
    module: shell::Module,
    tool: shell::Tool,
    left_visible: bool,
    right_visible: bool,
    maximized: bool,
    tree_search: String,
    tree_matches: std::collections::HashSet<String>,
    analysis_tab: bool,
    display_mode: shell::DisplayMode,
    show_mesh: bool,
    show_clouds: bool,
}
impl Desktop {
    fn new(cc: &eframe::CreationContext<'_>, scene: Scene) -> Self {
        theme::apply(&cc.egui_ctx, scene.view.theme);
        Self {
            scene,
            message: "Pronto · importe geometria ou explore seu workspace local.".into(),
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
            show_measurements: false,
            measure_mode: false,
            anchors: vec![],
            measure_name: "Distância 1".into(),
            known_distance: 100.,
            length_unit: LengthUnit::Millimetre,
            scale_provenance: "Referência informada pelo usuário".into(),
            transition: None,
            show_visual: false,
            hover_anchor: None,
            hover_pixel: None,
            hover_updated: std::time::Instant::now(),
            exact_a: 1,
            exact_b: 2,
            blast: None,
            blast_mode: true,
            blast_selected: false,
            blast_strength: 1.,
            module: shell::Module::Assembly,
            tool: shell::Tool::Orbit,
            left_visible: true,
            right_visible: true,
            maximized: false,
            tree_search: String::new(),
            tree_matches: Default::default(),
            analysis_tab: false,
            display_mode: shell::DisplayMode::Solid,
            show_mesh: true,
            show_clouds: true,
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
        self.restore_blast();
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
            self.blast = None;
            self.scene = Scene::default();
            self.anchors.clear();
            self.measure_mode = false;
            self.transition = None;
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
            Purpose::ExportMeasurements => {
                let result = measurement::export_report(&self.scene, &path);
                self.notice(result, "Relatório de medições exportado.");
            }
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
    fn tree(&mut self, ui: &mut egui::Ui, parent: Option<&str>, depth: usize) {
        if depth > 64 {
            return;
        }
        let groups: Vec<_> = self
            .scene
            .assemblies
            .iter()
            .filter(|g| g.parent.as_deref() == parent && self.tree_matches.contains(&g.id))
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
            if item.parent.as_deref() != parent || !self.tree_matches.contains(&item.id) {
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
    fn viewport(&mut self, ctx: &egui::Context) {
        let palette = self.palette();
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(palette.viewport)).show(ctx,|ui|{
            let (response,painter)=ui.allocate_painter(ui.available_size(),Sense::click_and_drag());let outer=response.rect;let rect=if self.scene.view.rulers{Rect::from_min_max(outer.min+Vec2::splat(viewport_tools::RULER),outer.max)}else{outer};let size=[rect.width() as f64,rect.height() as f64];
            if response.dragged() && self.pending.is_none(){self.stop_transition();let delta=ctx.input(|i|i.pointer.delta());let pan=self.tool==shell::Tool::Pan||response.dragged_by(egui::PointerButton::Secondary)||response.dragged_by(egui::PointerButton::Middle)||ctx.input(|i|i.modifiers.shift);if pan{self.scene.camera.pan([delta.x as f64,delta.y as f64],size[1]);}else if self.tool==shell::Tool::Orbit{self.scene.camera.orbit([delta.x as f64,delta.y as f64]);}self.dirty=true;}
            if response.hovered() && self.pending.is_none(){let scroll=ctx.input(|i|i.smooth_scroll_delta.y);if scroll!=0.{self.stop_transition();self.scene.camera.zoom(scroll as f64);self.dirty=true;}}
            if !ctx.wants_keyboard_input()&&ctx.input(|i|i.key_pressed(egui::Key::F)){self.fit_view();}
            let mut display_scene=self.scene.clone();
            if let Some(t)=&self.transition{display_scene.explosion=t.value();display_scene.camera=t.camera();}
            let display_painter=painter.with_clip_rect(rect);
            let to_screen=|p:[f64;2]|Pos2::new(rect.left()+p[0] as f32,rect.top()+p[1] as f32);
            if self.scene.grid&&display_scene.camera.projection==Projection::Orthographic{viewport_tools::ortho_grid(&display_painter,&display_scene,rect,palette);}
            if self.scene.grid&&display_scene.camera.projection==Projection::Perspective{
                let center=self.scene.camera.target;let step=10f64.powf((self.scene.camera.distance/8.).log10().floor());let range=10;
                for i in -range..=range{for axis in 0..2{let mut a=center;let mut b=center;a[2]=0.;b[2]=0.;a[axis]+=i as f64*step;b[axis]+=i as f64*step;a[1-axis]-=range as f64*step;b[1-axis]+=range as f64*step;
                    if let (Some(a),Some(b))=(display_scene.camera.project(a,size),display_scene.camera.project(b,size)){display_painter.line_segment([to_screen([a[0],a[1]]),to_screen([b[0],b[1]])],Stroke::new(1.0_f32,palette.border.gamma_multiply(0.55)));}
                }}
            }
            for item in &mut display_scene.items { if matches!(item.geometry.as_ref(),Geometry::Mesh(_)) && !self.show_mesh || matches!(item.geometry.as_ref(),Geometry::Points(_)) && !self.show_clouds {item.visible=false;} }
            display_scene.wireframe=self.display_mode==shell::DisplayMode::Wire || self.scene.wireframe;
            let mut primitives=noah_workspace::project(&display_scene,size);
            if self.display_mode==shell::DisplayMode::Points { primitives.retain(|p|p.pixels.len()==1); let transforms=display_scene.world_transforms(true).ok(); if let Some(ts)=transforms { let total:usize=display_scene.items.iter().filter(|i|i.visible && matches!(i.geometry.as_ref(),Geometry::Mesh(_))).map(|i|i.geometry.positions().len()).sum();let stride=total.div_ceil(noah_workspace::DISPLAY_POINTS).max(1);let mut serial=0;for (index,item) in display_scene.items.iter().enumerate().filter(|(_,i)|i.visible && matches!(i.geometry.as_ref(),Geometry::Mesh(_))) {for &p in item.geometry.positions(){let take=serial%stride==0;serial+=1;if take && let Ok(p)=ts[index].apply(p) && let Some(p)=display_scene.camera.project(p,size){primitives.push(noah_workspace::Primitive{item:index,pixels:vec![[p[0],p[1]]],depth:p[2],color:[142,187,210]});}}} } }
            if let Some(blast)=&self.blast {primitives.retain(|p|!blast.targets.contains(&p.item)); primitives.extend(blast.project(&display_scene.camera,size).into_iter().filter(|p|display_scene.items[p.item].visible));}
            primitives.sort_by(|a,b|b.depth.total_cmp(&a.depth));self.shown=primitives.len();
            if response.clicked() && self.pending.is_none()&& let Some(pos)=response.interact_pointer_pos() && rect.contains(pos){
                let point=[(pos.x-rect.left()) as f64,(pos.y-rect.top()) as f64];
                if self.blast.is_some(){self.notice(Err("Restaure o objeto antes de selecionar ou medir; os estilhaços são apenas visuais".into()),"");}
                else if self.measure_mode && self.transition.is_some(){self.notice(Err("Aguarde o fim da animação para selecionar vértices".into()),"");}
                else if self.measure_mode{
                    if let Some(anchor)=measurement::pick_anchor_controlled(&display_scene,point,size){
                        if self.anchors.last()==Some(&anchor){self.notice(Err("Escolha outro vértice para o segundo ponto".into()),"");}
                        else{self.anchors.push(anchor);if self.anchors.len()==2{self.measure_mode=false;self.show_measurements=true;self.notice(Ok(()),"Dois vértices selecionados. Abra Medir para registrar ou definir escala.");}else{self.notice(Ok(()),"Ponto A escolhido; selecione o ponto B.");}}
                    }else{let faces:usize=self.scene.items.iter().filter(|i|i.visible).map(|i|match i.geometry.as_ref(){Geometry::Mesh(m)=>m.triangles().len(),_=>0}).sum();let text=if self.scene.view.occlusion&&faces>200_000{"Proteção frontal limitada a 200 mil faces visíveis. Isole peças ou desative essa opção em Visual."}else if self.scene.view.selected_only&&self.scene.selected.is_none(){"Selecione uma peça/montagem ou desative o filtro em Visual."}else{"Nenhum vértice próximo. Aproxime a câmera e tente novamente."};self.notice(Err(text.into()),"");}
                }else{self.scene.selected=noah_workspace::pick(&primitives,point,7.).map(|i|self.scene.items[i].id.clone());self.dirty=true;}
            }
            let mut mesh=egui::Mesh::default();let mut edges=Vec::new();
            for primitive in &primitives{
                if primitive.pixels.iter().flatten().any(|v|v.abs()>1e7){continue;}
                let selected=Some(&self.scene.items[primitive.item].id)==self.scene.selected.as_ref()||self.scene.selected.as_ref().is_some_and(|id|self.scene.belongs_to(primitive.item,id));
                let original=primitive.color;let color=if selected{Color32::from_rgb(((original[0] as u16+palette.primary.r() as u16)/2) as u8,((original[1] as u16+palette.primary.g() as u16)/2) as u8,((original[2] as u16+palette.primary.b() as u16)/2) as u8)}else{Color32::from_rgb(original[0],original[1],original[2])};
                if primitive.pixels.len()==1{let pos=to_screen(primitive.pixels[0]);mesh.add_colored_rect(Rect::from_center_size(pos,Vec2::splat(self.scene.point_size)),color);}else{
                    let points:Vec<_>=primitive.pixels.iter().map(|&p|to_screen(p)).collect();
                    if self.display_mode!=shell::DisplayMode::Wire{let start=mesh.vertices.len() as u32;for &p in &points{mesh.colored_vertex(p,color);}mesh.indices.extend([start,start+1,start+2]);}
                    let line=if selected{palette.primary}else{palette.border};
                    if display_scene.wireframe {for i in 0..3{edges.push(egui::Shape::line_segment([points[i],points[(i+1)%3]],Stroke::new(1.0_f32,line)));}}
                }
            }
            display_painter.add(egui::Shape::mesh(mesh));display_painter.extend(edges);
            let visual_transforms=display_scene.world_transforms(true).ok();
            let anchor_screen=|a:&Anchor|self.scene.items.iter().position(|i|i.id==a.item_id).filter(|&index|self.scene.items[index].visible).and_then(|index|self.scene.items[index].geometry.positions().get(a.vertex).and_then(|&p|visual_transforms.as_ref().and_then(|ts|ts[index].apply(p).ok()))).and_then(|p|display_scene.camera.project(p,size)).map(|p|to_screen([p[0],p[1]]));
            for measure in self.scene.measurements.iter().filter(|_|self.scene.view.annotations && self.blast.is_none()){
                if let (Some(a),Some(b))=(anchor_screen(&measure.a),anchor_screen(&measure.b)){
                    display_painter.line_segment([a,b],Stroke::new(1.5_f32,palette.primary));display_painter.circle_filled(a,3.5,palette.primary);display_painter.circle_filled(b,3.5,palette.primary);
                    let label=match self.scene.distance(&measure.a,&measure.b){Ok(d)=>format!("{}: {:.*} {} (montado)",measure.name,self.scene.view.decimals,d.value,d.unit.map(|u|u.symbol()).unwrap_or("u. cena")),Err(_)=>format!("{}: recalibrar escala",measure.name)};
                    display_painter.text(a+(b-a)*0.5,egui::Align2::CENTER_BOTTOM,label,FontId::proportional(11.),palette.primary);
                }
            }
            for (i,a) in self.anchors.iter().filter(|_|self.blast.is_none()).enumerate(){if let Some(p)=anchor_screen(a){display_painter.circle_filled(p,6.,palette.warning);display_painter.text(p+Vec2::new(10.,-10.),egui::Align2::LEFT_BOTTOM,if i==0{"A"}else{"B"},FontId::proportional(14.),Color32::WHITE);}}
            if self.measure_mode{display_painter.text(rect.center_top()+Vec2::new(0.,25.),egui::Align2::CENTER_TOP,"MEDIÇÃO: clique nos vértices A e B",FontId::proportional(14.),palette.primary);}
            if self.measure_mode&&self.transition.is_none()&&self.blast.is_none(){ctx.request_repaint_after(Duration::from_millis(75));
                if let Some(pos)=response.hover_pos().filter(|pos|rect.contains(*pos)){
                    let pixel=[(pos.x-rect.left()) as f64,(pos.y-rect.top()) as f64];
                    if self.hover_pixel!=Some(pixel)&&self.hover_updated.elapsed()>Duration::from_millis(75){self.hover_anchor=measurement::pick_anchor_controlled(&display_scene,pixel,size);self.hover_pixel=Some(pixel);self.hover_updated=std::time::Instant::now();}
                    if let Some(a)=&self.hover_anchor&&let Some(p)=anchor_screen(a){display_painter.circle_stroke(p,9.,Stroke::new(1.5_f32,palette.primary));display_painter.line_segment([p-Vec2::new(13.,0.),p+Vec2::new(13.,0.)],Stroke::new(1.0_f32,palette.primary));display_painter.text(p+Vec2::new(15.,-15.),egui::Align2::LEFT_BOTTOM,format!("Vértice {}",a.vertex+1),FontId::proportional(11.),palette.primary);}
                }else{self.hover_anchor=None;self.hover_pixel=None;}
            }
            viewport_tools::draw(&painter,&display_scene,outer,rect,palette,response.hover_pos());
            let top=rect.left_top()+Vec2::new(22.,22.);display_painter.text(top,egui::Align2::LEFT_TOP,if display_scene.camera.projection==Projection::Orthographic{"ORTOGRÁFICA · U/V"}else{"PERSPECTIVA"},FontId::proportional(11.),palette.secondary);
            display_painter.text(top+Vec2::new(0.,20.),egui::Align2::LEFT_TOP,format!("{} primitivas na tela",self.shown),FontId::proportional(10.),palette.secondary);
            if self.scene.items.is_empty(){let center=rect.center();display_painter.text(center-Vec2::new(0.,25.),egui::Align2::CENTER_CENTER,"Seu espaço de geometria",FontId::proportional(27.),palette.text);display_painter.text(center+Vec2::new(0.,20.),egui::Align2::CENTER_CENTER,"Importe fotos, uma geometria ou abra seu projeto.",FontId::proportional(13.),palette.secondary);
                let area=Rect::from_center_size(center+Vec2::new(0.,90.),Vec2::new(350.,85.));
                ui.scope_builder(egui::UiBuilder::new().max_rect(area),|ui|{ui.vertical_centered(|ui|{if theme::action(ui,"Importar fotos",palette,true).clicked(){self.choose(Purpose::Images);self.module=shell::Module::Capture;self.right_visible=true;}ui.horizontal(|ui|{if ui.button("Abrir geometria").clicked(){self.choose(Purpose::Import);}
if ui.button("Abrir workspace").clicked(){self.choose(Purpose::OpenSession);}});});});
            }
            display_painter.text(rect.left_bottom()+Vec2::new(22.,-24.),egui::Align2::LEFT_BOTTOM,"Arrastar: órbita   •   Direito / Shift: pan   •   Roda: zoom   •   Clique: selecionar",FontId::proportional(10.),palette.secondary);
            display_painter.text(rect.right_bottom()-Vec2::new(20.,24.),egui::Align2::RIGHT_BOTTOM,"XYZ do arquivo",FontId::proportional(10.),palette.primary);
        });
    }
}
impl Desktop {
    fn reconstruction_window(&mut self, ctx: &egui::Context) {
        let palette = self.palette();
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
    if ui.button(RichText::new("Iniciar reconstrução").color(palette.primary)).clicked(){let result=self.reconstruction.start();self.notice(result,"Reconstrução iniciada. Você pode continuar explorando a cena.");}
   });
   ui.separator();if busy{ui.horizontal(|ui|{ui.spinner();ui.label("Processamento em andamento");if ui.button("Cancelar processamento").clicked(){let result=self.reconstruction.cancel();self.notice(result,"Cancelamento solicitado.");}});}
   ui.label(&self.reconstruction.status);
   if let Some(manifest)=&self.reconstruction.manifest{ui.label(format!("Estado: {} · {} imagens",jobs::status_label(&manifest.status),manifest.inputs.len()));for step in &manifest.steps{ui.horizontal(|ui|{ui.colored_label(match step.status{noah_pipeline::Status::Succeeded=>palette.success,noah_pipeline::Status::Failed=>palette.error,noah_pipeline::Status::Cancelled=>palette.warning,_=>palette.secondary},jobs::status_label(&step.status));ui.label(jobs::step_label(&step.name));});}}
   ui.horizontal(|ui|{
    if ui.button("Ver logs e diagnóstico").clicked(){match self.reconstruction.logs(){Ok(text)=>{self.log_text=text;self.show_logs=true;},Err(e)=>self.notice(Err(e),"")}}
    let output=self.reconstruction.output.clone();let available=!busy&&output.as_ref().is_some_and(|p|p.join("exports/sparse.ply").is_file());
    if ui.add_enabled(available,egui::Button::new("Abrir nuvem no workspace")).clicked()&&let Some(root)=output{self.load_source(root);}
   });ui.label(RichText::new("Falhas e resultados parciais são preservados. As fotos não são alteradas.").size(11.).color(palette.secondary));
  });
        self.show_reconstruction = open;
    }
}
impl eframe::App for Desktop {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let palette = self.palette();
        theme::apply(ctx, self.scene.view.theme);
        if self.scene.view.reduced_motion && self.transition.is_some() {
            self.transition = None;
            self.notice(Ok(()), "Vista aplicada sem animação (movimento reduzido).");
        }
        if let Some(t) = &mut self.transition {
            if t.tick(ctx.input(|i| i.stable_dt) as f64) {
                ctx.request_repaint_after(Duration::from_millis(16));
            } else {
                let exploded = t.to > 0.;
                self.transition = None;
                self.notice(
                    Ok(()),
                    if exploded {
                        "Vista explodida aplicada; geometria original preservada."
                    } else {
                        "Conjunto remontado."
                    },
                );
            }
        }
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
                            self.blast = None;
                            self.scene = scene;
                            ctx.request_repaint();
                            self.transition = None;
                            self.hover_anchor = None;
                            self.anchors.clear();
                            self.measure_mode = false;
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
        if let Some(blast) = &mut self.blast {
            if self.scene.view.reduced_motion && blast.playing {
                blast.progress = if blast.reverse { 0. } else { 1. };
                blast.playing = false;
            } else {
                blast.tick(ctx.input(|i| i.stable_dt) as f64);
            }
            if blast.playing {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }
        if self
            .blast
            .as_ref()
            .is_some_and(|b| b.reverse && b.progress == 0. && !b.playing)
        {
            self.restore_blast();
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
                    ui.label(
                        RichText::new(if self.reconstruction.worker.is_some() {
                            "Processando · acompanhe em Reconstrução"
                        } else {
                            &self.message
                        })
                        .size(11.)
                        .color(if self.error {
                            palette.error
                        } else {
                            palette.secondary
                        }),
                    )
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
                            .color(palette.primary),
                        );
                    });
                });
            });
        if !self.maximized && self.left_visible {
            self.left(ctx);
        }
        if !self.maximized && self.right_visible {
            self.inspector(ctx);
        }
        self.viewport_toolbar(ctx);
        self.viewport(ctx);
        if self.transition.is_some() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if self.show_measurements {
            self.restore_blast();
            self.measurement_window(ctx);
        }
        if self.show_visual {
            self.visual_window(ctx);
        }
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

impl Desktop {
    fn measurement_window(&mut self, ctx: &egui::Context) {
        let palette = self.palette();
        let mut open = self.show_measurements;
        let mut requested_pick = false;
        egui::Window::new("Medições e referência de escala").open(&mut open).default_width(440.).default_pos([300.,90.]).show(ctx,|ui|{
   egui::ScrollArea::vertical().id_salt("measurement_scroll").max_height((ctx.content_rect().height()-170.).max(240.)).show(ui,|ui|{
   ui.label("Distância entre vértices no estado montado.");ui.label(RichText::new("A vista explodida não altera o valor medido.").size(11.).color(palette.secondary));
   ui.horizontal(|ui|{if ui.button("Escolher pontos A e B").clicked(){self.anchors.clear();self.measure_mode=true;self.hover_anchor=None;requested_pick=true;self.notice(Ok(()),"Clique nos vértices da geometria: primeiro A, depois B.");}
if ui.button("Cancelar seleção").clicked(){self.measure_mode=false;self.anchors.clear();}});
   egui::CollapsingHeader::new("Seleção exata / coordenadas").show(ui,|ui|{
    ui.label("Selecione uma peça na árvore e informe os índices dos vértices (começam em 1).");
    ui.horizontal(|ui|{ui.label("A");ui.add(egui::DragValue::new(&mut self.exact_a).range(1..=1_000_000));ui.label("B");ui.add(egui::DragValue::new(&mut self.exact_b).range(1..=1_000_000));});
    if ui.button("Usar estes vértices").clicked(){
     if let Some(item)=self.scene.items.iter().find(|i|Some(&i.id)==self.scene.selected.as_ref()){
      let a=Anchor{item_id:item.id.clone(),vertex:self.exact_a.saturating_sub(1)};let b=Anchor{item_id:item.id.clone(),vertex:self.exact_b.saturating_sub(1)};
      match self.scene.raw_distance(&a,&b){Ok(_) if a!=b=>{self.anchors=vec![a,b];self.measure_mode=false;self.notice(Ok(()),"Vértices selecionados por índice original.");},_=>self.notice(Err("Índices ausentes ou iguais".into()),"")}
     }else{self.notice(Err("Selecione uma peça, não uma montagem, na árvore.".into()),"");}
    }
    ui.label(RichText::new("Para melhorar o controle, use vistas ortográficas e o filtro de seleção em Visual.").size(10.).color(palette.secondary));
   });
   for (i,anchor) in self.anchors.iter().enumerate(){let name=self.scene.items.iter().find(|item|item.id==anchor.item_id).map(|i|i.name.as_str()).unwrap_or("Fonte ausente");ui.label(format!("{}: {} / vértice {}",if i==0{"A"}else{"B"},name,anchor.vertex+1));}
   for (label,anchor) in ["A","B"].into_iter().zip(&self.anchors){if let Ok(p)=self.scene.anchor_position(anchor,false){ui.label(RichText::new(format!("{label} montado: [{:.*}, {:.*}, {:.*}] u. cena",self.scene.view.decimals,p[0],self.scene.view.decimals,p[1],self.scene.view.decimals,p[2])).size(10.).color(palette.secondary));}}
   if self.anchors.len()==2{
    let a=self.anchors[0].clone();let b=self.anchors[1].clone();
    match self.scene.distance(&a,&b){Ok(d)=>{ui.heading(format!("{:.*} {}",self.scene.view.decimals,d.value,d.unit.map(|u|u.symbol()).unwrap_or("unidades da cena")));},Err(e)=>{ui.colored_label(palette.error,e.to_string());}}
    if let (Ok(pa),Ok(pb))=(self.scene.anchor_position(&a,false),self.scene.anchor_position(&b,false)){
     let (factor,unit)=self.scene.scale_factor().ok().flatten().map(|(f,u)|(f,u.symbol())).unwrap_or((1.,"u. cena"));let delta=std::array::from_fn::<_,3,_>(|i|(pb[i]-pa[i])*factor);
     ui.label(format!("ΔX {:.*} · ΔY {:.*} · ΔZ {:.*} {}",self.scene.view.decimals,delta[0],self.scene.view.decimals,delta[1],self.scene.view.decimals,delta[2],unit));
    }
    ui.horizontal(|ui|{ui.label("Nome");ui.text_edit_singleline(&mut self.measure_name);});if ui.button("Registrar distância").clicked(){let result=self.scene.add_measurement(a.clone(),b.clone(),self.measure_name.clone());match result{Ok(_)=>{self.dirty=true;self.notice(Ok(()),"Medição registrada com incerteza desconhecida.");},Err(e)=>self.notice(Err(e),"")}}
    ui.separator();ui.label(RichText::new("Definir escala por distância conhecida").strong());ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut self.known_distance).speed(0.1).range(1e-9..=1e12));egui::ComboBox::from_id_salt("length_unit").selected_text(self.length_unit.symbol()).show_ui(ui,|ui|{ui.selectable_value(&mut self.length_unit,LengthUnit::Millimetre,"mm");ui.selectable_value(&mut self.length_unit,LengthUnit::Metre,"m");});});ui.label("Origem / método da referência");ui.text_edit_singleline(&mut self.scale_provenance);
    if ui.button("Aplicar referência de escala").clicked(){let result=self.scene.calibrate(a,b,self.known_distance,self.length_unit,self.scale_provenance.clone());if result.is_ok(){self.dirty=true;}self.notice(result,"Escala declarada. Geometria original preservada.");}
   }
   ui.separator();match self.scene.scale_factor(){Ok(Some((_,unit)))=>{ui.label(RichText::new(format!("Escala declarada em {}",unit.symbol())).color(palette.primary));},Ok(None)=>{ui.label("Escala física desconhecida; valores em unidades da cena.");},Err(e)=>{ui.colored_label(palette.warning,e.to_string());}}
   if self.scene.calibration.is_some()&&ui.button("Remover referência de escala").clicked(){self.scene.calibration=None;self.dirty=true;}
   ui.label(RichText::new("Incerteza desconhecida. Escala declarada não garante exatidão metrológica.").size(10.).color(palette.secondary));
   let mut remove=None;egui::ScrollArea::vertical().max_height(180.).show(ui,|ui|{for m in &self.scene.measurements{ui.horizontal(|ui|{let text=match self.scene.distance(&m.a,&m.b){Ok(d)=>format!("{}: {:.*} {}",m.name,self.scene.view.decimals,d.value,d.unit.map(|u|u.symbol()).unwrap_or("u. cena")),Err(_)=>format!("{}: recalibrar",m.name)};ui.label(text);if ui.small_button("Remover").clicked(){remove=Some(m.id.clone());}});}});
   if let Some(id)=remove{self.scene.measurements.retain(|m|m.id!=id);self.dirty=true;}
   if ui.add_enabled(!self.scene.measurements.is_empty(),egui::Button::new("Exportar relatório de medições")).clicked(){self.choose(Purpose::ExportMeasurements);}
   ui.label(RichText::new("Salvar revisão do workspace preserva medidas, âncoras e referência de escala.").size(10.).color(palette.secondary));
   });
  });
        self.show_measurements = open && !requested_pick;
        if !open {
            self.measure_mode = false;
        }
    }
}

impl Desktop {
    fn palette(&self) -> theme::Palette {
        theme::Palette::new(self.scene.view.theme)
    }
    fn stop_transition(&mut self) {
        self.hover_anchor = None;
        self.hover_pixel = None;
        if let Some(t) = self.transition.take() {
            self.scene.explosion = t.value();
            self.scene.camera = t.camera();
        }
    }
    fn animate_explosion(&mut self, target: f64) {
        self.blast = None;
        self.stop_transition();
        let from = self.scene.explosion;
        let camera_from = self.scene.camera.clone();
        self.scene.explosion = target;
        if let Err(e) = self
            .scene
            .validate_world()
            .and_then(|()| self.scene.fit(false))
        {
            self.scene.explosion = from;
            self.scene.camera = camera_from;
            self.notice(Err(e), "");
            return;
        }
        if !self.scene.view.reduced_motion && (from - target).abs() > 1e-9 {
            self.transition = Some(animation::ExplosionTransition {
                from,
                to: target,
                camera_from,
                camera_to: self.scene.camera.clone(),
                elapsed: 0.,
                duration: 0.9,
            });
        }
        self.hover_anchor = None;
        self.measure_mode = false;
        self.dirty = true;
        self.notice(
            Ok(()),
            if self.transition.is_some() {
                if target > 0. {
                    "Separando componentes · geometria original preservada"
                } else {
                    "Remontando o conjunto"
                }
            } else if target > 0. {
                "Vista explodida aplicada; geometria original preservada."
            } else {
                "Conjunto remontado."
            },
        );
    }
    fn preset(&mut self, view: &str) {
        self.stop_transition();
        self.scene.camera.projection = Projection::Orthographic;
        match view {
            "Frontal" => {
                self.scene.camera.yaw = -std::f64::consts::FRAC_PI_2;
                self.scene.camera.pitch = 0.;
            }
            "Superior" => {
                self.scene.camera.yaw = -std::f64::consts::FRAC_PI_2;
                self.scene.camera.pitch = std::f64::consts::FRAC_PI_2;
            }
            "Lateral" => {
                self.scene.camera.yaw = 0.;
                self.scene.camera.pitch = 0.;
            }
            _ => {
                self.scene.camera.yaw = -1.1;
                self.scene.camera.pitch = 0.45;
            }
        }
        let r = self.scene.fit(false);
        self.notice(r, "Vista ortográfica aplicada. Réguas uniformes na tela.");
        self.dirty = true;
    }
    fn visual_window(&mut self, ctx: &egui::Context) {
        let palette = self.palette();
        let mut open = self.show_visual;
        egui::Window::new("Visualização · Arctic").open(&mut open).default_width(390.).show(ctx,|ui|{
   ui.label(RichText::new("APARÊNCIA").size(11.).color(palette.secondary));
   ui.horizontal(|ui|{if ui.selectable_value(&mut self.scene.view.theme,ThemeMode::ArcticDark,"Arctic Dark").changed(){self.dirty=true;}
if ui.selectable_value(&mut self.scene.view.theme,ThemeMode::ArcticLight,"Arctic Light").changed(){self.dirty=true;}});
   ui.separator();ui.label("PROJEÇÃO");
   ui.horizontal(|ui|{if ui.selectable_value(&mut self.scene.camera.projection,Projection::Perspective,"Perspectiva").changed(){let projection=self.scene.camera.projection;self.stop_transition();self.scene.camera.projection=projection;self.dirty=true;}
if ui.selectable_value(&mut self.scene.camera.projection,Projection::Orthographic,"Ortográfica").changed(){let projection=self.scene.camera.projection;self.stop_transition();self.scene.camera.projection=projection;self.dirty=true;}});
   ui.horizontal(|ui|{for name in ["Frontal","Superior","Lateral"]{if ui.button(name).clicked(){self.preset(name);}}});
   ui.label(RichText::new("Ortográfica: escala uniforme. Perspectiva: régua referida ao plano do alvo, não a todas as profundidades.").size(11.).color(palette.secondary));
   ui.separator();
   for (value,label) in [(&mut self.scene.view.rulers,"Réguas e escala"),(&mut self.scene.view.axes,"Eixos de orientação"),(&mut self.scene.view.annotations,"Anotações de medida"),(&mut self.scene.grid,"Grade de navegação"),(&mut self.scene.wireframe,"Arame / arestas"),(&mut self.scene.view.reduced_motion,"Reduzir movimentos")]{if ui.checkbox(value,label).changed(){self.dirty=true;}}
   ui.label("E4 · SELEÇÃO DE VÉRTICES");if ui.add(egui::Slider::new(&mut self.scene.view.snap_radius,3.0..=24.0).text("Raio / px")).changed(){self.dirty=true;}
   if ui.checkbox(&mut self.scene.view.selected_only,"Somente peça/montagem selecionada").changed(){self.dirty=true;}
   if ui.checkbox(&mut self.scene.view.occlusion,"Evitar vértices atrás de faces").changed(){self.dirty=true;}
   if ui.add(egui::Slider::new(&mut self.scene.view.decimals,0..=9).text("Casas decimais")).changed(){self.dirty=true;}
   ui.label(RichText::new("Casas decimais alteram a exibição, não a exatidão física. Preferências acompanham o workspace.").size(10.).color(palette.secondary));
  });
        self.show_visual = open;
    }
}
