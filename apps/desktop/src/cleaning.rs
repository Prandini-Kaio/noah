use super::*;
use noah_geometry::{PointCloud, cleaning};
use noah_storage::cloud_revision::{self, SourceRef};
#[derive(Clone)]
struct Edit {
    id: String,
    cloud: PointCloud,
    source: SourceRef,
    active: Vec<bool>,
    selection: Vec<bool>,
    candidate: Option<Vec<bool>>,
    recipe: Vec<serde_json::Value>,
    history: Vec<(Vec<bool>, Vec<serde_json::Value>)>,
    min: [f64; 3],
    max: [f64; 3],
    candidate_recipe: serde_json::Value,
}
#[derive(Default)]
pub(super) struct CleaningUi {
    pub parent: PathBuf,
    pub open: bool,
    pub rect_mode: bool,
    start: Option<Pos2>,
    edit: Option<Edit>,
    worker: Option<Receiver<noah_pipeline::Result<Vec<bool>>>>,
    save_worker: Option<Receiver<noah_pipeline::Result<PathBuf>>>,
    k: usize,
    sigma: f64,
    radius: f64,
    min_neighbours: usize,
    name: String,
    status: String,
    show_removed: bool,
}
impl Edit {
    fn checkpoint(&mut self) {
        if self.history.len() == 8 {
            self.history.remove(0);
        }
        self.history
            .push((self.active.clone(), self.recipe.clone()));
    }
    fn apply(
        &mut self,
        removed: Vec<bool>,
        recipe: serde_json::Value,
    ) -> noah_pipeline::Result<()> {
        if removed.len() != self.active.len() || self.recipe.len() >= 64 {
            return Err("limite de 64 operações; salve uma revisão".into());
        }
        if !self.active.iter().zip(&removed).any(|(&a, &r)| a && !r) {
            return Err("a operação removeria todos os pontos".into());
        }
        self.checkpoint();
        for (i, r) in removed.into_iter().enumerate() {
            if r {
                self.active[i] = false;
            }
        }
        self.recipe.push(recipe);
        self.candidate = None;
        self.selection.fill(false);
        Ok(())
    }
}
impl CleaningUi {
    pub fn with_parent(parent: PathBuf) -> Self {
        Self {
            parent,
            ..Default::default()
        }
    }
}
impl Desktop {
    pub(super) fn begin_cleaning(&mut self) -> noah_pipeline::Result<()> {
        let item = self
            .scene
            .items
            .iter()
            .find(|i| Some(&i.id) == self.scene.selected.as_ref())
            .ok_or("selecione uma nuvem")?;
        let Geometry::Points(cloud) = item.geometry.as_ref() else {
            return Err("limpeza atua em nuvens, não malhas".into());
        };
        cleaning::validate(cloud, &vec![true; cloud.positions().len()])?;
        let source = &self.scene.sources[item.source];
        let kind = serde_json::to_value(&source.kind)?
            .as_str()
            .ok_or("tipo de fonte inválido")?
            .to_string();
        let source = SourceRef {
            path: source.path.clone(),
            kind,
            sha256: source.sha256.clone(),
            item_id: item.id.clone(),
            unit: item.unit.clone(),
            frame: item.frame.clone(),
        };
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for p in cloud.positions() {
            for a in 0..3 {
                min[a] = min[a].min(p[a]);
                max[a] = max[a].max(p[a]);
            }
        }
        let n = cloud.positions().len();
        self.cleaning.edit = Some(Edit {
            id: item.id.clone(),
            cloud: cloud.clone(),
            source,
            active: vec![true; n],
            selection: vec![false; n],
            candidate: None,
            recipe: vec![],
            history: vec![],
            min,
            max,
            candidate_recipe: serde_json::Value::Null,
        });
        self.cleaning.k = 16;
        self.cleaning.sigma = 2.;
        self.cleaning.radius =
            ((0..3).map(|a| (max[a] - min[a]).powi(2)).sum::<f64>()).sqrt() / 100.;
        self.cleaning.min_neighbours = 3;
        self.cleaning.name = jobs::available_name(&self.cleaning.parent, "nuvem-limpa");
        self.cleaning.status =
            "Edição temporária iniciada. Salve uma nova revisão para persistir.".into();
        self.restore_blast();
        self.scene.explosion = 0.;
        self.transition = None;
        self.measure_mode = false;
        self.alignment.preview = false;
        Ok(())
    }
    pub(super) fn cleaning_window(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.cleaning.worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.cleaning.worker = None;
                    match result {
                        Ok(mask) => {
                            if let Some(e) = &mut self.cleaning.edit {
                                self.cleaning.status = format!(
                                    "Prévia: {} candidatos à remoção. Revise antes de aplicar.",
                                    mask.iter().filter(|&&v| v).count()
                                );
                                e.candidate = Some(mask);
                            }
                        }
                        Err(e) => self.cleaning.status = e.to_string(),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.cleaning.worker = None;
                    self.cleaning.status = "Filtro interrompido".into();
                }
                _ => ctx.request_repaint_after(Duration::from_millis(75)),
            }
        }
        if let Some(rx) = &self.cleaning.save_worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.cleaning.save_worker = None;
                    match result {
                        Ok(path) => {
                            if let Some(edit) = &self.cleaning.edit
                                && let Some(item) =
                                    self.scene.items.iter_mut().find(|i| i.id == edit.id)
                            {
                                item.visible = false;
                            }
                            self.cleaning.edit = None;
                            self.cleaning.rect_mode = false;
                            self.load_source(path);
                            self.cleaning.status =
                                "Revisão salva e aberta; original preservado.".into();
                        }
                        Err(e) => self.cleaning.status = e.to_string(),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.cleaning.save_worker = None;
                    self.cleaning.status = "Gravação interrompida".into();
                }
                _ => ctx.request_repaint_after(Duration::from_millis(75)),
            }
        }
        if !self.cleaning.open {
            return;
        }
        let mut open = true;
        let mut start = false;
        let mut rect = false;
        let mut close = false;
        let mut choose_output = false;
        let mut discard = false;
        egui::Window::new("Limpeza da nuvem").open(&mut open).vscroll(true).max_height((ctx.content_rect().height()-160.).max(350.)).default_height(660.).default_width(530.).default_pos([330.,120.]).show(ctx,|ui|{
   let busy=self.cleaning.worker.is_some()||self.cleaning.save_worker.is_some();
   ui.add_enabled_ui(!busy,|ui|{
    if ui.button(if self.cleaning.edit.is_some(){"Reiniciar da nuvem selecionada"}else{"Editar nuvem selecionada"}).clicked(){start=true;}
    if let Some(e)=&mut self.cleaning.edit{
     ui.strong(format!("Editando: {}",self.scene.items.iter().find(|i|i.id==e.id).map(|i|i.name.as_str()).unwrap_or("nuvem de origem")));
     ui.label(format!("Original: {} · Atual: {} · Seleção: {}",e.active.len(),e.active.iter().filter(|&&v|v).count(),e.selection.iter().filter(|&&v|v).count()));
     ui.label("Coordenadas locais da fonte; unidades originais. Não altera medidas antigas.");
     ui.checkbox(&mut self.cleaning.show_removed,"Comparar: mostrar removidos em vermelho");
     ui.horizontal(|ui|{if ui.button("Selecionar retângulo no viewport").clicked(){rect=true;}
if ui.button("Limpar seleção").clicked(){e.selection.fill(false);}});
     ui.collapsing("Selecionar caixa XYZ local",|ui|{for a in 0..3{ui.horizontal(|ui|{ui.label(["X","Y","Z"][a]);ui.add(egui::DragValue::new(&mut e.min[a]).speed(0.1));ui.add(egui::DragValue::new(&mut e.max[a]).speed(0.1));});}
if ui.button("Selecionar dentro da caixa").clicked(){match cleaning::box_selection(&e.cloud,&e.active,e.min,e.max){Ok(v)=>e.selection=v,Err(s)=>self.cleaning.status=s.into()}}});
     ui.horizontal(|ui|{if ui.button("Remover seleção").clicked(){let r=e.apply(e.selection.clone(),serde_json::json!({"operation":"remove_selection","count":e.selection.iter().filter(|&&v|v).count()}));if let Err(s)=r{self.cleaning.status=s.to_string();}}
if ui.button("Isolar seleção").clicked(){let remove=e.active.iter().zip(&e.selection).map(|(&a,&s)|a&&!s).collect();if let Err(s)=e.apply(remove,serde_json::json!({"operation":"isolate_selection"})){self.cleaning.status=s.to_string();}}});
     ui.separator();
     ui.horizontal(|ui|{ui.label("SOR: vizinhos");ui.add(egui::DragValue::new(&mut self.cleaning.k).range(1..=64));ui.label("σ");ui.add(egui::DragValue::new(&mut self.cleaning.sigma).range(0.1..=10.).speed(0.1));});
     ui.horizontal(|ui|{ui.label("Raio (u. da fonte)");ui.add(egui::DragValue::new(&mut self.cleaning.radius).speed(0.01));ui.label("mín. vizinhos");ui.add(egui::DragValue::new(&mut self.cleaning.min_neighbours).range(1..=64));});
     let sor=ui.button("Prévia de outliers estatísticos").clicked();let radius=ui.button("Prévia de outliers por raio").clicked();
     if sor||radius{let cloud=e.cloud.clone();let active=e.active.clone();let k=self.cleaning.k;let sigma=self.cleaning.sigma;let r=self.cleaning.radius;let n=self.cleaning.min_neighbours;e.candidate=None;e.candidate_recipe=if sor{serde_json::json!({"operation":"statistical","k":k,"sigma":sigma,"deviation":"population"})}else{serde_json::json!({"operation":"radius","radius":r,"min_neighbours":n})};let(tx,rx)=mpsc::channel();std::thread::spawn(move||{let result=if sor{cleaning::statistical(&cloud,&active,k,sigma)}else{cleaning::radius(&cloud,&active,r,n)}.map_err(|s|s.into());let _=tx.send(result);});self.cleaning.worker=Some(rx);}
     if let Some(mask)=e.candidate.clone(){ui.label(format!("Candidatos: {} (vermelho)",mask.iter().filter(|&&v|v).count()));ui.horizontal(|ui|{if ui.button("Aplicar remoção prevista").clicked()&&let Err(s)=e.apply(mask,e.candidate_recipe.clone()){self.cleaning.status=s.to_string();}
if ui.button("Cancelar prévia").clicked(){e.candidate=None;}});}
     ui.horizontal(|ui|{if ui.add_enabled(!e.history.is_empty(),egui::Button::new("Desfazer limpeza")).clicked()&&let Some((active,recipe))=e.history.pop(){e.active=active;e.recipe=recipe;e.candidate=None;e.selection.fill(false);}
if ui.button("Ver nuvem no viewport").clicked(){close=true;}});
     if ui.button("Descartar edição temporária").clicked(){discard=true;}
     ui.separator();ui.horizontal(|ui|{if ui.button("Pasta de resultados").clicked(){choose_output=true;}ui.label(self.cleaning.parent.display().to_string());});ui.horizontal(|ui|{ui.label("Novo nome");ui.text_edit_singleline(&mut self.cleaning.name);});
     if ui.button("Salvar e abrir revisão limpa").clicked(){let name=&self.cleaning.name;if Path::new(name).components().count()!=1||matches!(name.as_str(),"."|"..")||name.trim().is_empty(){self.cleaning.status="Nome inválido".into();}else{let output=self.cleaning.parent.join(name);let e=e.clone();let(tx,rx)=mpsc::channel();std::thread::spawn(move||{let indices=e.active.iter().enumerate().filter_map(|(i,&a)|a.then_some(i)).collect();let result=cloud_revision::create(&output,e.source,&e.cloud,indices,e.recipe).map(|()|output);let _=tx.send(result);});self.cleaning.save_worker=Some(rx);}}
    }
   });if busy{ui.spinner();ctx.request_repaint_after(Duration::from_millis(75));}ui.label(&self.cleaning.status);ui.label("Até 200 mil pontos. Filtros podem remover detalhes reais; prévia não é classificação garantida. Desfazer guarda 8 operações; fechar mantém a edição temporária.");
  });
        self.cleaning.open = open && !close && !rect;
        if rect {
            self.cleaning.rect_mode = true;
            self.cleaning.start = None;
        }
        if discard {
            self.cleaning.edit = None;
            self.cleaning.rect_mode = false;
        }
        if choose_output {
            self.choose(Purpose::CleaningOutput);
        }
        if start {
            let r = self.begin_cleaning();
            self.notice(r, "Limpeza iniciada. Original preservado.");
        }
    }
    pub(super) fn cleaning_display_count(&self) -> usize {
        self.cleaning
            .edit
            .as_ref()
            .map(|e| {
                let stride = e
                    .active
                    .len()
                    .div_ceil(noah_workspace::DISPLAY_POINTS)
                    .max(1);
                (0..e.active.len())
                    .step_by(stride)
                    .filter(|&i| e.active[i] || self.cleaning.show_removed)
                    .count()
            })
            .unwrap_or(0)
    }
    pub(super) fn cleaning_target(&self) -> Option<&str> {
        self.cleaning.edit.as_ref().map(|e| e.id.as_str())
    }
    pub(super) fn cleaning_viewport(
        &mut self,
        ctx: &egui::Context,
        response: &egui::Response,
        painter: &egui::Painter,
        rect: Rect,
        scene: &Scene,
    ) {
        let primary = self.palette().primary;
        let Some(e) = &mut self.cleaning.edit else {
            return;
        };
        let Some(index) = scene.items.iter().position(|i| i.id == e.id) else {
            return;
        };
        if !scene.items[index].visible {
            return;
        }
        let Ok(transforms) = scene.world_transforms(false) else {
            return;
        };
        let t = transforms[index];
        let size = [rect.width() as f64, rect.height() as f64];
        if self.cleaning.rect_mode {
            if response.drag_started()
                && let Some(pos) = ctx.input(|i| i.pointer.press_origin())
                && rect.contains(pos)
            {
                self.cleaning.start = Some(pos);
            }
            if let (Some(start), Some(end)) = (self.cleaning.start, response.interact_pointer_pos())
            {
                let region = Rect::from_two_pos(start, end).intersect(rect);
                painter.rect_stroke(
                    region,
                    0.,
                    Stroke::new(1.5_f32, primary),
                    egui::StrokeKind::Inside,
                );
                if response.drag_stopped() {
                    for (i, &p) in e.cloud.positions().iter().enumerate() {
                        e.selection[i] = e.active[i]
                            && t.apply(p)
                                .ok()
                                .and_then(|p| scene.camera.project(p, size))
                                .is_some_and(|p| {
                                    region.contains(Pos2::new(
                                        rect.left() + p[0] as f32,
                                        rect.top() + p[1] as f32,
                                    ))
                                });
                    }
                    self.cleaning.rect_mode = false;
                    self.cleaning.start = None;
                    self.cleaning.status = format!(
                        "{} pontos selecionados. Abra Limpar nuvem para remover/isolar.",
                        e.selection.iter().filter(|&&v| v).count()
                    );
                }
            }
            painter.text(
                rect.center_top() + Vec2::new(0., 45.),
                egui::Align2::CENTER_TOP,
                "Arraste para selecionar pontos em toda a profundidade · Esc cancela",
                FontId::proportional(13.),
                Color32::YELLOW,
            );
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.cleaning.rect_mode = false;
                self.cleaning.start = None;
            }
        }
        let stride = e
            .active
            .len()
            .div_ceil(noah_workspace::DISPLAY_POINTS)
            .max(1);
        for i in (0..e.active.len()).step_by(stride) {
            let candidate = e.candidate.as_ref().is_some_and(|m| m[i]);
            if !e.active[i] && !self.cleaning.show_removed {
                continue;
            }
            if let Some(p) = t
                .apply(e.cloud.positions()[i])
                .ok()
                .and_then(|p| scene.camera.project(p, size))
            {
                let color = if !e.active[i] || candidate {
                    Color32::from_rgb(212, 143, 143)
                } else if e.selection[i] {
                    Color32::from_rgb(228, 183, 121)
                } else {
                    e.cloud
                        .colors()
                        .map(|c| Color32::from_rgb(c[i][0], c[i][1], c[i][2]))
                        .unwrap_or(Color32::from_rgb(142, 187, 210))
                };
                painter.circle_filled(
                    Pos2::new(rect.left() + p[0] as f32, rect.top() + p[1] as f32),
                    scene.point_size.max(2.) / 2.,
                    color,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_preview_commit_and_undo_keep_original_indices_and_reject_empty() {
        let c = PointCloud::new(vec![[0.; 3], [1.; 3], [99.; 3]], None).unwrap();
        let mut e = Edit {
            id: "id".into(),
            cloud: c.clone(),
            source: SourceRef {
                path: PathBuf::from("/source.ply"),
                kind: "ply".into(),
                sha256: "a".repeat(64),
                item_id: "id".into(),
                unit: "unknown".into(),
                frame: "file".into(),
            },
            active: vec![true; 3],
            selection: vec![false; 3],
            candidate: Some(vec![false, false, true]),
            recipe: vec![],
            history: vec![],
            min: [0.; 3],
            max: [99.; 3],
            candidate_recipe: serde_json::json!({"operation":"radius"}),
        };
        assert_eq!(e.active, vec![true; 3]);
        e.apply(e.candidate.clone().unwrap(), e.candidate_recipe.clone())
            .unwrap();
        assert_eq!(e.active, vec![true, true, false]);
        assert_eq!(e.cloud, c);
        assert!(e.candidate.is_none());
        assert!(e.apply(vec![true; 3], serde_json::Value::Null).is_err());
        let (active, recipe) = e.history.pop().unwrap();
        e.active = active;
        e.recipe = recipe;
        assert_eq!(e.active, vec![true; 3]);
        assert!(e.recipe.is_empty());
    }
}
