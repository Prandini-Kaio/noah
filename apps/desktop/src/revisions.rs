//! Capture revisions and disposable comparison preview, separate from saved scene geometry.
use super::*;
use noah_photogrammetry::comparison::{self, Comparison};
#[derive(Default)]
pub(super) struct RevisionsUi {
    pub open: bool,
    pub reference: PathBuf,
    pub updated: PathBuf,
    pub comparison: Option<Comparison>,
    pub preview: Option<Scene>,
    before_camera: Option<noah_workspace::Camera>,
    worker: Option<Receiver<noah_pipeline::Result<Comparison>>>,
    threshold: f64,
    mode: usize,
    status: String,
}
impl Desktop {
    pub(super) fn compare_latest_revision(&mut self) {
        self.revisions.open = true;
        if let Some(root) = &self.reconstruction.output {
            self.revisions.updated = root.clone();
            let p = root.join("refinement.json");
            if let Ok(bytes) = noah_pipeline::read_bounded(&p, 4 * 1024 * 1024)
                && let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&bytes)
                && let Some(parent) = meta["parent_path"].as_str()
            {
                self.revisions.reference = PathBuf::from(parent);
            }
        }
    }
    pub(super) fn configure_refinement(&mut self) {
        let result = (|| -> noah_pipeline::Result<()> {
            let path = self
                .selected_run()
                .ok_or("selecione um run de reconstrução")?;
            let root = if path.join("cloud-revision.json").is_file() {
                noah_storage::cloud_revision::run_selection(&path)?.0.path
            } else {
                path
            };
            self.reconstruction.refine_parent = Some(root);
            self.reconstruction.source_run = None;
            self.reconstruction.clean_revision = None;
            self.reconstruction.additional_images = PathBuf::new();
            self.reconstruction.name = self.reconstruction.available_name("revisao");
            Ok(())
        })();
        if result.is_ok() {
            self.show_reconstruction = true;
            self.pipeline_tab = 0;
            self.revisions.open = false;
        }
        self.notice(
            result,
            "Run pai selecionado. Escolha novas fotos no passo 1; o pai será preservado.",
        );
    }
    pub(super) fn end_comparison(&mut self) {
        if let Some(camera) = self.revisions.before_camera.take() {
            self.scene.camera = camera;
        }
        self.revisions.preview = None;
    }
    pub(super) fn reopen_comparison(&mut self, path: PathBuf) {
        self.end_comparison();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(comparison::reopen(&path));
        });
        self.revisions.worker = Some(rx);
        self.revisions.open = true;
        self.revisions.status = "Verificando fontes e recalculando o relatório…".into();
    }
    fn preview_comparison(&mut self) {
        if self.cleaning_target().is_some() {
            self.notice(
                Err("conclua a edição da nuvem antes da comparação visual".into()),
                "",
            );
            return;
        }
        self.restore_blast();
        self.stop_transition();
        let Some(c) = &self.revisions.comparison else {
            return;
        };
        let mode = self.revisions.mode;
        let camera = self.scene.camera.clone();
        let mut scene = Scene {
            camera,
            point_size: 3.,
            ..Default::default()
        };
        scene.view = self.scene.view.clone();
        scene.view.annotations = false;
        scene.selected = None;
        for (index, cloud) in [&c.reference, &c.updated_aligned].into_iter().enumerate() {
            if (mode == 1 && index == 1) || ((mode == 2 || mode == 3) && index == 0) {
                continue;
            }
            let colors: Vec<_> = cloud
                .positions()
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    if index == 0 {
                        [142, 187, 210]
                    } else if mode == 3 {
                        let v = (c.updated_distances[i] / c.report.threshold).clamp(0., 2.) / 2.;
                        [
                            (138. + 74. * v) as u8,
                            (193. - 50. * v) as u8,
                            (169. - 26. * v) as u8,
                        ]
                    } else {
                        [228, 183, 121]
                    }
                })
                .collect();
            let cloud = noah_geometry::PointCloud::new(cloud.positions().to_vec(), Some(colors));
            let Ok(cloud) = cloud else { continue };
            scene.items.push(noah_workspace::Item {
                id: format!("comparison:{index}"),
                name: if index == 0 {
                    "Anterior"
                } else {
                    "Atual alinhada"
                }
                .into(),
                source: 0,
                geometry: std::sync::Arc::new(Geometry::Points(cloud)),
                base: Default::default(),
                placement: Default::default(),
                visible: true,
                unit: "unknown".into(),
                frame: "comparison_reference_sfm".into(),
                parent: None,
                explode_direction: [0.; 3],
            });
        }
        if self.revisions.before_camera.is_none() {
            self.revisions.before_camera = Some(self.scene.camera.clone());
            if let Err(e) = scene.fit(false) {
                self.notice(Err(e), "");
                return;
            }
            self.scene.camera = scene.camera.clone();
        }
        self.revisions.preview = Some(scene);
        self.measure_mode = false;
        self.alignment.preview = false;
        self.show_reconstruction = false;
    }
    pub(super) fn revisions_window(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.revisions.worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.revisions.worker = None;
                    match result {
                        Ok(c) => {
                            self.revisions.reference = c.report.reference.path.clone();
                            self.revisions.updated = c.report.updated.path.clone();
                            self.revisions.status="Comparação calculada em referencial alinhado. Nenhuma garantia de ganho ou precisão física.".into();
                            self.revisions.comparison = Some(c);
                        }
                        Err(e) => self.revisions.status = e.to_string(),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.revisions.worker = None;
                    self.revisions.status = "Comparação interrompida".into();
                }
                _ => ctx.request_repaint_after(Duration::from_millis(75)),
            }
        }
        if !self.revisions.open {
            return;
        }
        let mut open = true;
        let mut go_to_capture = false;
        let p = self.palette();
        egui::Window::new("E6 · capturas e comparação de revisões").open(&mut open).default_pos([330.,110.]).default_width(740.).vscroll(true).max_height((ctx.content_rect().height()-140.).max(400.)).show(ctx,|ui|{
   ui.heading("Revisões sem alterar o original");if ui.button("Usar comparação pronta do kit").clicked() && let (Some(a),Some(b))=(kit_path("refinamento/pronto/anterior"),kit_path("refinamento/pronto/atual")){self.revisions.reference=a;self.revisions.updated=b;}ui.horizontal(|ui|{if ui.add_enabled(self.selected_run().is_some()&&self.reconstruction.worker.is_none(),egui::Button::new("Adicionar fotos ao run selecionado…")).clicked(){self.configure_refinement();go_to_capture=self.show_reconstruction;}
if ui.button("Reabrir comparação salva…").clicked(){self.choose(Purpose::ComparisonOpen);}});
   let busy=self.revisions.worker.is_some();ui.add_enabled_ui(!busy,|ui|{
    for(reference,label,purpose)in[(true,"Anterior / referência",Purpose::CompareReference),(false,"Atual / nova revisão",Purpose::CompareUpdated)]{ui.separator();ui.strong(label);ui.horizontal(|ui|{if ui.button(if reference{"Escolher run anterior…"}else{"Escolher run atualizado…"}).clicked(){self.choose(purpose);}
if ui.add_enabled(self.selected_run().is_some(),egui::Button::new(if reference{"Usar seleção como anterior"}else{"Usar seleção como atual"})).clicked()&&let Some(path)=self.selected_run(){if reference{self.revisions.reference=path;}else{self.revisions.updated=path;}}});ui.label(if reference{self.revisions.reference.display().to_string()}else{self.revisions.updated.display().to_string()});}
    ui.horizontal(|ui|{ui.label("Limiar visual / u. do run anterior (0 = 1% da diagonal)");ui.add(egui::DragValue::new(&mut self.revisions.threshold).speed(0.01).range(0.0..=1e12));});
    if ui.add_enabled(self.revisions.reference.is_dir()&&self.revisions.updated.is_dir(),egui::Button::new("Alinhar câmeras e comparar")).clicked(){self.end_comparison();self.revisions.comparison=None;let a=self.revisions.reference.clone();let b=self.revisions.updated.clone();let threshold=self.revisions.threshold;let(tx,rx)=mpsc::channel();std::thread::spawn(move||{let _=tx.send(comparison::compare(&a,&b,threshold));});self.revisions.worker=Some(rx);self.revisions.status="Verificando hashes, alinhando câmeras e calculando distâncias…".into();}
   });if busy{ui.spinner();ctx.request_repaint_after(Duration::from_millis(75));}ui.label(&self.revisions.status);
   if let Some(c)=&self.revisions.comparison{let r=&c.report;ui.separator();ui.heading("Resultado da comparação");ui.label(format!("Fotos de entrada: {} -> {} · registradas: {} -> {} · pontos: {} -> {}",r.reference.input_images,r.updated.input_images,r.reference.registered_images,r.updated.registered_images,r.reference.points,r.updated.points));ui.label(format!("{} câmeras comuns · escala relativa {:.6} · residual normalizado {:.3}%",r.registration.common_images,r.registration.new_to_reference_scale,r.registration.normalized_camera_rmse*100.));ui.label(format!("Limiar {:.6} u. de referência; não corresponde a mm ou precisão validada",r.threshold));
    for(label,d)in[("Anterior -> Atual",&r.reference_to_updated),("Atual -> Anterior",&r.updated_to_reference)]{ui.label(format!("{}: média {:.6}, RMS {:.6}, P95 {:.6}, máx {:.6} · {:.1}% dentro do limiar",label,d.mean,d.rms,d.p95,d.max,d.within_threshold_fraction*100.));}ui.label(RichText::new(&r.sampling).size(10.).color(p.secondary));
    let before=self.revisions.mode;ui.horizontal_wrapped(|ui|{for(i,label)in["Sobrepor","Anterior","Atual alinhada","Diferenças na atual"].iter().enumerate(){ui.selectable_value(&mut self.revisions.mode,i,*label);}});if before!=self.revisions.mode&&self.revisions.preview.is_some(){self.preview_comparison();}
    ui.horizontal(|ui|{if ui.button("Visualizar comparação").clicked(){self.preview_comparison();}
if ui.button("Voltar ao workspace original").clicked(){self.end_comparison();}
if ui.button("Salvar relatório de comparação…").clicked(){self.choose(Purpose::ComparisonReport);}});
   }
   ui.separator();ui.label("Azul = anterior; laranja = atual alinhada. Diferenças: verde próximo, vermelho distante. Distâncias entre pontos amostrados, não detecção de defeito, área nova ou erro metrológico. Exige ≥4 fotos únicas em comum e câmeras não colineares; rejeita residual >5% do raio das câmeras.");
  });
        self.revisions.open = open && !go_to_capture;
    }
}
