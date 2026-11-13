//! Workspace layout and contextual controls; all actions delegate to existing services.
use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Capture,
    Reconstruction,
    Assembly,
    Inspection,
    Measures,
    Export,
}
impl Module {
    fn name(self) -> &'static str {
        match self {
            Self::Capture => "Captura",
            Self::Reconstruction => "Reconstrução",
            Self::Assembly => "Montagem",
            Self::Inspection => "Inspeção",
            Self::Measures => "Medidas",
            Self::Export => "Exportação",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Pan,
    Orbit,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Solid,
    Wire,
    Points,
}
impl DisplayMode {
    pub fn mesh_style(self, legacy_wire: bool, overlay: bool) -> (bool, bool) {
        let wire = legacy_wire || self == Self::Wire;
        (!wire, wire || overlay)
    }
}
impl Desktop {
    pub(super) fn header(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        egui::TopBottomPanel::top("app_shell_header")
            .exact_height(92.)
            .frame(
                egui::Frame::new()
                    .fill(p.background)
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                if self.pending.is_some() {
                    ui.disable();
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new("N").size(28.).color(p.primary));
                    ui.label(RichText::new("NOAH 2.0").size(19.).strong());
                    ui.add_space(12.);
                    ui.menu_button("Workspace…", |ui| {
                        if ui.button("Novo workspace").clicked() {
                            self.request_replace(Replace::New);
                            ui.close();
                        }
                        if ui.button("Abrir workspace…").clicked() {
                            self.choose(Purpose::OpenSession);
                            ui.close();
                        }
                        if ui.button("Abrir projeto / run…").clicked() {
                            self.choose(Purpose::Project);
                            ui.close();
                        }
                        if ui.button("Abrir peças de alinhamento do kit…").clicked() {
                            self.open_alignment_kit();
                            ui.close();
                        }
                        if ui.button("Explorar montagem de exemplo…").clicked() {
                            self.choose(Purpose::Demo);
                            ui.close();
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button("Preferências…", |ui| {
                            if ui.button("Tema e opções visuais…").clicked() {
                                self.show_visual = true;
                                ui.close();
                            }
                            ui.checkbox(&mut self.left_visible, "Painel de estrutura");
                            ui.checkbox(&mut self.right_visible, "Inspetor");
                            ui.checkbox(&mut self.maximized, "Maximizar viewport");
                        });
                        if theme::action(ui, "Salvar", p, false).clicked() {
                            if self.cleaning_target().is_some(){self.notice(Err("Salve a revisão limpa ou descarte a edição antes de salvar o workspace".into()),"");}else{
                            self.choose(Purpose::SaveSession);
                            }
                        }
                        let ready = self.reconstruction.ready_inputs();
                        if ui
                            .add_enabled(
                                ready,
                                egui::Button::new(
                                    RichText::new("Rodar pipeline").color(p.background),
                                )
                                .fill(p.primary),
                            )
                            .on_hover_text("Configure fotos, COLMAP e destino em Reconstrução")
                            .clicked()
                        {
                            let r = self.reconstruction.start();
                            self.notice(r, "Pipeline iniciado.");
                            self.show_reconstruction = true;
                        }
                        ui.menu_button("Importar…", |ui| {
                            if ui.button("Fotografias…").clicked() {
                                self.choose(Purpose::Images);
                                self.module = Module::Capture;
                                self.right_visible = true;
                                ui.close();
                            }
                            if ui.button("Geometria PLY / OBJ…").clicked() {
                                self.choose(Purpose::Import);
                                ui.close();
                            }
                        });
                    });
                });
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    for m in [
                        Module::Capture,
                        Module::Reconstruction,
                        Module::Assembly,
                        Module::Inspection,
                        Module::Measures,
                        Module::Export,
                    ] {
                        if ui.selectable_label(self.module == m, m.name()).clicked() {
                            self.module = m;
                            self.right_visible = true;
                            self.maximized = false;
                            self.analysis_tab = m == Module::Measures;
                            if m == Module::Reconstruction {
                                self.show_reconstruction = true;
                            }
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("LOCAL · SEM UPLOAD")
                                .size(10.)
                                .color(p.secondary),
                        );
                    });
                });
            });
    }
    pub(super) fn left(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        egui::SidePanel::left("workspace_structure")
            .default_width(250.)
            .width_range(210.0..=380.)
            .resizable(true)
            .frame(egui::Frame::new().fill(p.surface).inner_margin(12))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("workspace_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("WORKSPACE LOCAL").size(11.).strong());
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("Ocultar").clicked() {
                                        self.left_visible = false;
                                    }
                                },
                            );
                        });
                        ui.label(
                            RichText::new(self.last_dir.display().to_string())
                                .size(10.)
                                .color(p.secondary),
                        );
                        ui.add_space(16.);
                        egui::CollapsingHeader::new("Fluxo de reconstrução")
                            .default_open(self.scene.items.is_empty())
                            .show(ui, |ui| {
                                let captured = self.reconstruction.images.is_dir();
                                let processing = self.reconstruction.worker.is_some();
                                let pipeline_state = if processing {
                                    "Processando"
                                } else {
                                    match self.reconstruction.manifest.as_ref().map(|m| &m.status) {
                                        Some(noah_pipeline::Status::Succeeded) => "Concluído",
                                        Some(noah_pipeline::Status::Failed) => {
                                            "Erro · consulte diagnóstico"
                                        }
                                        Some(noah_pipeline::Status::Cancelled) => "Cancelado",
                                        Some(noah_pipeline::Status::Prepared) => "Preparado",
                                        Some(noah_pipeline::Status::Running) => {
                                            "Execução registrada"
                                        }
                                        None => "A configurar",
                                    }
                                };
                                for (n, title, detail, state, module) in [
                                    (
                                        1,
                                        "Captura",
                                        "Importar e validar fotos",
                                        if captured {
                                            "Fotos escolhidas"
                                        } else {
                                            "Não iniciado"
                                        },
                                        Module::Capture,
                                    ),
                                    (
                                        2,
                                        "Reconstrução",
                                        "Nuvem e malha",
                                        pipeline_state,
                                        Module::Reconstruction,
                                    ),
                                    (
                                        3,
                                        "Montagem",
                                        "Organizar componentes",
                                        if self.scene.items.is_empty() {
                                            "Sem geometria"
                                        } else {
                                            "Geometria aberta"
                                        },
                                        Module::Assembly,
                                    ),
                                ] {
                                    if ui
                                        .selectable_label(
                                            self.module == module,
                                            format!("{n}   {title}"),
                                        )
                                        .clicked()
                                    {
                                        self.module = module;
                                        if module != Module::Assembly {
                                            self.show_reconstruction = true;
                                        }
                                    }
                                    ui.label(RichText::new(detail).size(10.).color(p.secondary));
                                    ui.label(RichText::new(state).size(10.).color(
                                        if processing && n == 2 {
                                            p.warning
                                        } else {
                                            p.secondary
                                        },
                                    ));
                                    ui.add_space(8.);
                                }
                            });
                        ui.add_space(12.);
                        ui.separator();
                        ui.label(RichText::new("ESTRUTURA DO CONJUNTO").size(11.).strong());
                        ui.add_space(8.);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.tree_search)
                                .hint_text("Buscar peça ou montagem…")
                                .desired_width(f32::INFINITY),
                        );
                        ui.add_space(8.);
                        self.tree_matches = matching_ids(&self.scene, &self.tree_search);
                        if self.tree_matches.is_empty() && !self.tree_search.is_empty() {
                            ui.label("Nenhum resultado para a busca.");
                        }
                        ui.add_enabled_ui(self.pending.is_none() && self.blast.is_none(), |ui| {
                            self.tree(ui, None, 0)
                        });
                        if self.scene.items.is_empty() {
                            ui.label(
                                RichText::new(
                                    "Nenhum objeto. Importe uma geometria ou abra um workspace.",
                                )
                                .color(p.secondary),
                            );
                            if ui.button("Importar geometria…").clicked() {
                                self.choose(Purpose::Import);
                            }
                        }
                        ui.add_space(12.);
                        ui.separator();
                        egui::CollapsingHeader::new("Nova montagem").show(ui, |ui| {
                            ui.add_enabled_ui(
                                self.pending.is_none() && self.blast.is_none(),
                                |ui| {
                                    ui.text_edit_singleline(&mut self.new_group_name);
                                    if ui.button("Criar montagem").clicked() {
                                        match self.scene.add_assembly(self.new_group_name.clone()) {
                                            Ok(id) => {
                                                self.scene.selected = Some(id);
                                                self.dirty = true;
                                            }
                                            Err(e) => self.notice(Err(e), ""),
                                        }
                                    }
                                },
                            );
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} peças · {} montagens",
                                self.scene.items.len(),
                                self.scene.assemblies.len()
                            ))
                            .size(11.)
                            .color(p.secondary),
                        );
                    });
            });
    }
    pub(super) fn inspector(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        egui::SidePanel::right("context_inspector").default_width(300.).width_range(260.0..=420.).resizable(true).frame(egui::Frame::new().fill(p.surface).inner_margin(12)).show(ctx,|ui|{
   ui.horizontal(|ui|{ui.selectable_value(&mut self.analysis_tab,false,"Propriedades");ui.selectable_value(&mut self.analysis_tab,true,"Análise");if ui.small_button("×").on_hover_text("Ocultar inspetor").clicked(){self.right_visible=false;}});ui.separator();
   egui::ScrollArea::vertical().id_salt("context_scroll").show(ui,|ui|{
    ui.add_enabled_ui(self.pending.is_none(),|ui|{
     if self.analysis_tab{
      self.inspection_summary(ui);
     }else{
      match self.module{
       Module::Capture=>{ui.heading("Captura");ui.label("Escolha a pasta local de fotografias para o pipeline.");if ui.button("Importar fotos…").clicked(){self.choose(Purpose::Images);}ui.label(self.reconstruction.images.display().to_string());if ui.button("Preparar fotografias…").clicked(){self.preparation.open=true;}
if ui.button("Configurar reconstrução").clicked(){self.show_reconstruction=true;}ui.separator();},
       Module::Reconstruction=>{ui.heading("Reconstrução");ui.label(&self.reconstruction.status);if ui.button("Configuração e acompanhamento").clicked(){self.show_reconstruction=true;}ui.separator();},
       Module::Export=>{ui.heading("Exportação");ui.label("Exporta a peça no estado montado e preserva os dados originais.");let selected=self.scene.items.iter().any(|i|Some(&i.id)==self.scene.selected.as_ref());if ui.add_enabled(selected,egui::Button::new("Exportar peça montada…")).clicked(){self.choose(Purpose::Export);}
if ui.button("Relatório de medições…").clicked(){self.choose(Purpose::ExportMeasurements);}ui.separator();},_=>{}
      }
      if self.module==Module::Assembly && ui.button("Alinhar peças…").clicked(){self.open_alignment();}
      egui::CollapsingHeader::new("Seleção e transformações").default_open(true).show(ui,|ui|{
       if self.blast.is_some(){ui.label("Prévia temporária ativa. Restaure para editar a seleção.");}else{self.selection_properties(ui);}
      });
      ui.add_space(12.);
      if ui.button("Limpar nuvem…").clicked(){self.cleaning.open=true;}
      if matches!(self.module,Module::Assembly|Module::Inspection){egui::CollapsingHeader::new("Explosão e estilhaços").default_open(true).show(ui,|ui|self.blast_controls(ui));ui.add_space(12.);}
      egui::CollapsingHeader::new("Visualização").default_open(self.module==Module::Inspection).show(ui,|ui|{
       ui.horizontal(|ui|{for (mode,label) in [(DisplayMode::Solid,"Sólido"),(DisplayMode::Wire,"Arame"),(DisplayMode::Points,"Pontos")]{if ui.selectable_value(&mut self.display_mode,mode,label).changed(){self.scene.wireframe=mode==DisplayMode::Wire;self.dirty=true;}}});
       ui.checkbox(&mut self.show_mesh,"Exibir malhas");ui.checkbox(&mut self.show_clouds,"Exibir nuvens");
       if ui.checkbox(&mut self.scene.grid,"Grade de navegação").changed(){self.dirty=true;}
       ui.checkbox(&mut self.wire_overlay,"Sobrepor arestas ao sólido");
       if ui.add(egui::Slider::new(&mut self.scene.point_size,1.0..=8.0).text("Ponto / px")).changed(){self.dirty=true;}
       if ui.button("Tema, réguas e opções…").clicked(){self.show_visual=true;}
       ui.label(RichText::new("Exibição amostrada; profundidade aproximada.").size(10.).color(p.secondary));
      });
     }
    });
   });
  });
    }
    pub(super) fn viewport_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("viewport_tools")
            .frame(
                egui::Frame::new()
                    .fill(self.palette().elevated)
                    .inner_margin(8),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if (!self.left_visible || self.maximized)
                        && ui.small_button("Estrutura").clicked()
                    {
                        self.left_visible = true;
                        self.maximized = false;
                    }
                    ui.selectable_value(&mut self.tool, Tool::Select, "Selecionar");
                    ui.selectable_value(&mut self.tool, Tool::Pan, "Pan");
                    ui.selectable_value(&mut self.tool, Tool::Orbit, "Órbita");
                    ui.separator();
                    if ui.button("Medir").clicked() {
                        self.restore_blast();
                        self.show_measurements = true;
                        self.module = Module::Measures;
                        self.analysis_tab = true;
                    }
                    ui.menu_button("Câmera…", |ui| {
                        for (projection, name) in [
                            (Projection::Perspective, "Perspectiva"),
                            (Projection::Orthographic, "Ortográfica"),
                        ] {
                            if ui
                                .selectable_label(self.scene.camera.projection == projection, name)
                                .clicked()
                            {
                                self.stop_transition();
                                self.scene.camera.projection = projection;
                                self.dirty = true;
                                ui.close();
                            }
                        }
                        for name in ["Frontal", "Superior", "Lateral", "Isométrica"] {
                            if ui.button(name).clicked() {
                                self.preset(name);
                                ui.close();
                            }
                        }
                    });
                    if ui.button("Enquadrar").clicked() {
                        self.fit_view();
                    }
                    ui.menu_button("Exibição…", |ui| {
                        if ui.checkbox(&mut self.scene.grid, "Grade").changed() {
                            self.dirty = true;
                        }
                        if ui.checkbox(&mut self.scene.view.rulers, "Réguas").changed() {
                            self.dirty = true;
                        }
                        if ui.checkbox(&mut self.scene.view.axes, "Eixos").changed() {
                            self.dirty = true;
                        }
                        if ui.button("Todas as opções…").clicked() {
                            self.show_visual = true;
                            ui.close();
                        }
                    });
                    if ui
                        .button(if self.maximized {
                            "Restaurar painéis"
                        } else {
                            "Maximizar"
                        })
                        .clicked()
                    {
                        self.maximized = !self.maximized;
                    }
                    if !self.right_visible
                        && !self.maximized
                        && ui.small_button("Inspetor").clicked()
                    {
                        self.right_visible = true;
                    }
                });
            });
    }
    pub(super) fn fit_view(&mut self) {
        self.stop_transition();
        let r = if let Some(b) = &self.blast {
            b.fit_camera(&mut self.scene.camera)
        } else {
            self.scene.fit(false)
        };
        self.notice(r, "Vista enquadrada.");
        self.dirty = true;
    }
    pub(super) fn restore_blast(&mut self) {
        if self.blast.take().is_some() {
            self.message = "Objeto restaurado; geometria original preservada.".into();
        }
    }
    fn start_blast(&mut self) {
        self.alignment.preview = false;
        self.alignment.open = false;
        self.stop_transition();
        self.measure_mode = false;
        self.hover_anchor = None;
        match if self.blast_detail == 3 {
            blast::Blast::new(&self.scene, self.blast_selected, self.blast_strength)
        } else {
            blast::Blast::with_detail(
                &self.scene,
                self.blast_selected,
                self.blast_strength,
                self.blast_detail,
            )
        } {
            Ok(mut b) => {
                if self.blast_follow {
                    b.follow_camera(&self.scene.camera);
                }
                if self.scene.view.reduced_motion {
                    b.progress = 1.;
                    b.playing = false;
                }
                let count = b.count();
                self.blast = Some(b);
                self.notice(
                    Ok(()),
                    &format!("Prévia: {count} estilhaços · dados originais preservados"),
                );
            }
            Err(e) => self.notice(Err(e), ""),
        }
    }
    fn blast_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.selectable_label(self.blast_mode, "Estilhaçar").clicked() {
                self.blast_mode = true;
            }
            if ui
                .selectable_label(!self.blast_mode, "Componentes")
                .clicked()
            {
                self.blast_mode = false;
                self.restore_blast();
            }
        });
        if self.blast_mode {
            ui.checkbox(&mut self.blast_selected, "Somente seleção");
            egui::ComboBox::from_id_salt("shard_size")
                .selected_text(match self.blast_detail {
                    2 => "Pedaços médios",
                    4 => "Pedaços muito pequenos",
                    _ => "Pedaços pequenos",
                })
                .show_ui(ui, |ui| {
                    for (n, label) in [(2, "Médios"), (3, "Pequenos"), (4, "Muito pequenos")] {
                        ui.selectable_value(&mut self.blast_detail, n, label);
                    }
                });
            ui.label(
                RichText::new(
                    "O tamanho é aplicado ao clicar Explodir; ajusta-se ao limite de exibição.",
                )
                .size(10.)
                .color(self.palette().secondary),
            );
            ui.checkbox(&mut self.blast_follow, "Câmera acompanha a explosão");
            if ui
                .add(egui::Slider::new(&mut self.blast_strength, 0.2..=6.0).text("Força"))
                .changed()
                && let Some(b) = &mut self.blast
            {
                b.strength = self.blast_strength;
                if let Some((from, _)) = &b.frame {
                    let from = from.clone();
                    b.follow_camera(&from);
                }
            }
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.scene.items.iter().any(|i| i.visible),
                        egui::Button::new("Explodir"),
                    )
                    .clicked()
                {
                    self.start_blast();
                }
                if ui
                    .add_enabled(self.blast.is_some(), egui::Button::new("Restaurar"))
                    .clicked()
                {
                    self.restore_blast();
                }
            });
            if let Some(b) = &mut self.blast {
                ui.horizontal(|ui| {
                    if ui
                        .button(if b.playing { "Pausar" } else { "Reproduzir" })
                        .clicked()
                    {
                        if b.progress >= 1. && !b.reverse {
                            b.progress = 0.;
                        }
                        if b.progress <= 0. && b.reverse {
                            b.progress = 1.;
                        }
                        b.playing = !b.playing;
                    }
                    if ui.button("Remontar").clicked() {
                        b.reverse = true;
                        b.playing = true;
                    }
                });
                if ui
                    .add(egui::Slider::new(&mut b.progress, 0.0..=1.0).text("Tempo"))
                    .changed()
                {
                    b.playing = false;
                }
                ui.label(format!("{} fragmentos visuais", b.count()));
            }
            ui.label(RichText::new("Malhas: estilhaços de superfície. Nuvens: dispersão de pontos. Efeito temporário, sem colisões.").size(10.).color(self.palette().secondary));
        } else {
            ui.horizontal(|ui| {
                if ui.button("Separar peças").clicked() {
                    self.animate_explosion(1.);
                }
                if ui.button("Montada").clicked() {
                    self.animate_explosion(0.);
                }
            });
            let before = (self.scene.explosion, self.scene.explosion_distance);
            let mut edited = false;
            let mut value = self.scene.explosion;
            if ui
                .add(egui::Slider::new(&mut value, 0.0..=1.0).text("Intensidade"))
                .changed()
            {
                self.stop_transition();
                self.scene.explosion = value;
                edited = true;
            }
            if ui
                .add(
                    egui::DragValue::new(&mut self.scene.explosion_distance)
                        .range(0.0..=1e6)
                        .speed(0.05)
                        .prefix("Afastamento "),
                )
                .changed()
            {
                self.stop_transition();
                edited = true;
            }
            if edited {
                if let Err(e) = self.scene.validate_world() {
                    self.scene.explosion = before.0;
                    self.scene.explosion_distance = before.1;
                    self.notice(Err(e), "");
                } else {
                    self.dirty = true;
                }
            }
        }
    }
    fn selection_properties(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        if let Some(id) = self.scene.selected.clone() {
            let item = self.scene.items.iter().position(|i| i.id == id);
            let group = self.scene.assemblies.iter().position(|g| g.id == id);
            let (mut placement, parent) = if let Some(index) = item {
                let i = &self.scene.items[index];
                ui.heading(&i.name);
                let count = match i.geometry.as_ref() {
                    Geometry::Mesh(m) => format!("{} triângulos", m.triangles().len()),
                    Geometry::Points(p) => format!("{} pontos", p.positions().len()),
                };
                ui.label(RichText::new(count).color(palette.primary));
                ui.label(format!("Unidade: {}", i.unit));
                ui.label(RichText::new(&i.frame).size(10.).color(palette.secondary));
                (i.placement.clone(), i.parent.clone())
            } else if let Some(index) = group {
                let group = &mut self.scene.assemblies[index];
                ui.heading("Montagem");
                if ui.text_edit_singleline(&mut group.name).changed() {
                    self.dirty = true;
                }
                let count = self
                    .scene
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| self.scene.belongs_to(*i, &id))
                    .count();
                ui.label(format!("{count} componentes descendentes"));
                (
                    self.scene.assemblies[index].placement.clone(),
                    self.scene.assemblies[index].parent.clone(),
                )
            } else {
                return;
            };
            ui.separator();
            let groups: Vec<_> = self
                .scene
                .assemblies
                .iter()
                .filter(|g| g.id != id)
                .map(|g| (g.id.clone(), g.name.clone()))
                .collect();
            let mut chosen = parent.clone();
            egui::ComboBox::from_id_salt("parent")
                .selected_text(
                    parent
                        .as_ref()
                        .and_then(|id| {
                            groups
                                .iter()
                                .find(|(g, _)| g == id)
                                .map(|(_, name)| name.as_str())
                        })
                        .unwrap_or("Raiz do workspace"),
                )
                .width(240.)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut chosen, None, "Raiz do workspace");
                    for (id, name) in &groups {
                        ui.selectable_value(&mut chosen, Some(id.clone()), name);
                    }
                });
            if chosen != parent {
                let result = self.scene.set_parent(&id, chosen);
                if result.is_ok() {
                    self.dirty = true;
                }
                self.notice(
                    result,
                    "Montagem pai atualizada; valores locais preservados.",
                );
            }
            ui.label(
                RichText::new("A transformação abaixo é local à montagem pai.")
                    .size(10.)
                    .color(palette.secondary),
            );
            ui.add_space(8.);
            let mut changed = false;
            ui.label("Translação X / Y / Z");
            ui.horizontal(|ui| {
                for value in &mut placement.translation {
                    changed |= ui
                        .add(
                            egui::DragValue::new(value)
                                .speed(0.01)
                                .range(-1e9..=1e9)
                                .max_decimals(3),
                        )
                        .changed();
                }
            });
            ui.label("Rotação X / Y / Z (graus)");
            ui.horizontal(|ui| {
                for angle in &mut placement.angles {
                    let mut degrees = angle.to_degrees();
                    if ui
                        .add(
                            egui::DragValue::new(&mut degrees)
                                .speed(0.5)
                                .range(-3600.0..=3600.0)
                                .max_decimals(1),
                        )
                        .changed()
                    {
                        *angle = degrees.to_radians();
                        changed = true;
                    }
                }
            });
            ui.label("Escala uniforme");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut placement.scale)
                        .speed(0.01)
                        .range(1e-6..=1e6),
                )
                .changed();
            if changed {
                let result = if let Some(index) = item {
                    self.scene.set_placement(index, placement)
                } else {
                    self.scene.set_group_placement(&id, placement)
                };
                if result.is_ok() {
                    self.dirty = true;
                }
                self.notice(result, "Transformação aplicada à seleção.");
            }
            ui.horizontal(|ui| {
                if ui.button("Restaurar ajuste").clicked() {
                    let result = if let Some(index) = item {
                        self.scene.set_placement(index, Placement::default())
                    } else {
                        self.scene.set_group_placement(&id, Placement::default())
                    };
                    self.notice(result, "Ajuste local restaurado.");
                    self.dirty = true;
                }
                if ui.button("Enquadrar").clicked() {
                    let r = self.scene.fit(true);
                    self.notice(r, "Seleção enquadrada.");
                    self.dirty = true;
                }
            });
            if let Some(index) = item {
                ui.add_space(5.);
                if ui.button("Exportar peça montada").clicked() {
                    self.choose(Purpose::Export);
                }
                let source = &self.scene.sources[self.scene.items[index].source];
                ui.label(
                    RichText::new(source.path.display().to_string())
                        .size(9.)
                        .color(palette.secondary),
                );
            }
        } else {
            ui.heading("Explore seu conjunto");
            ui.label("Selecione uma peça ou montagem na cena ou na estrutura à esquerda.");
        }
    }
}

/// Keep matching descendants and their ancestor path; group hits reveal their subtree.
fn matching_ids(scene: &Scene, search: &str) -> std::collections::HashSet<String> {
    let query = search.trim().to_lowercase();
    let mut result = std::collections::HashSet::new();
    let ancestors = |parent: Option<&str>| {
        let mut ids = Vec::new();
        let mut current = parent;
        for _ in 0..64 {
            let Some(id) = current else { break };
            let Some(g) = scene.assemblies.iter().find(|g| g.id == id) else {
                break;
            };
            ids.push(g);
            current = g.parent.as_deref();
        }
        ids
    };
    for group in &scene.assemblies {
        let chain = ancestors(group.parent.as_deref());
        if group.name.to_lowercase().contains(&query)
            || chain.iter().any(|g| g.name.to_lowercase().contains(&query))
        {
            result.insert(group.id.clone());
            result.extend(chain.iter().map(|g| g.id.clone()));
        }
    }
    for item in &scene.items {
        let chain = ancestors(item.parent.as_deref());
        if item.name.to_lowercase().contains(&query)
            || chain.iter().any(|g| g.name.to_lowercase().contains(&query))
        {
            result.insert(item.id.clone());
            result.extend(chain.iter().map(|g| g.id.clone()));
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_keeps_parent_path_and_group_descendants() {
        let mut s = Scene::default();
        let a = s.add_assembly("Conjunto".into()).unwrap();
        let b = s.add_assembly("Motor".into()).unwrap();
        s.set_parent(&b, Some(a.clone())).unwrap();
        s.add_source(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/datasets/reference-mesh.obj"),
        )
        .unwrap();
        let id = s.items[0].id.clone();
        s.set_parent(&id, Some(b.clone())).unwrap();
        assert_eq!(
            matching_ids(&s, "motor"),
            [a.clone(), b.clone(), id.clone()].into_iter().collect()
        );
        assert_eq!(
            matching_ids(&s, "REFERENCE"),
            [a, b, id].into_iter().collect()
        );
        assert!(matching_ids(&s, "inexistente").is_empty());
        assert_eq!(matching_ids(&s, "").len(), 3);
    }
}

#[cfg(test)]
mod wire_tests {
    use super::*;
    #[test]
    fn legacy_wire_removes_faces_overlay_keeps_faces() {
        assert_eq!(DisplayMode::Solid.mesh_style(true, false), (false, true));
        assert_eq!(DisplayMode::Wire.mesh_style(false, false), (false, true));
        assert_eq!(DisplayMode::Solid.mesh_style(false, true), (true, true));
        assert_eq!(DisplayMode::Solid.mesh_style(false, false), (true, false));
    }
}
