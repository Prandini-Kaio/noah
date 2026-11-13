//! Workspace layout and contextual controls; all actions delegate to existing services.
use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Reconstruction,
    Assembly,
    Inspection,
    Measures,
    Export,
}
impl Module {
    fn name(self) -> &'static str {
        match self {
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
                        if ui.button("Explorar armadura por camadas…").clicked(){self.choose(Purpose::ArmourDemo);ui.close();}
                        if ui.button("Explorar montagem de exemplo…").clicked() {
                            self.choose(Purpose::Demo);
                            ui.close();
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Configurações…").clicked(){self.show_general=true;}
                        if theme::action(ui, "Salvar", p, false).clicked() {
                            if self.cleaning_target().is_some(){self.notice(Err("Salve a revisão limpa ou descarte a edição antes de salvar o workspace".into()),"");}else{
                            self.choose(Purpose::SaveSession);
                            }
                        }
                        if ui.add(egui::Button::new(RichText::new("Reconstruir…").color(p.background)).fill(p.primary)).clicked(){self.show_reconstruction=true;self.pipeline_tab=if self.reconstruction.worker.is_some(){1}else if self.reconstruction.manifest.is_some(){2}else{0};}
                        ui.menu_button("Importar…", |ui| {
                            if ui.button("Fotografias…").clicked() {
                                self.choose(Purpose::Images);
                                self.module = Module::Reconstruction;
                                self.show_reconstruction=true;self.pipeline_tab=0;
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
                            if m == Module::Assembly{self.inspector_tab=2;}
                            if m == Module::Reconstruction {
                                self.pipeline_tab=if self.reconstruction.worker.is_some(){1}else if self.reconstruction.manifest.is_some(){2}else{0};
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
                        let clouds=self.scene.items.iter().filter(|i|matches!(i.geometry.as_ref(),Geometry::Points(_))).count();
                        ui.label(if self.scene.items.is_empty(){"Nenhum objeto 3D aberto".into()}else{format!("{} nuvens · {} malhas",clouds,self.scene.items.len()-clouds)});
                        if self.reconstruction.worker.is_some(){ui.colored_label(p.warning,"Reconstrução em andamento");}
                        else if self.cleaning_target().is_some(){ui.colored_label(p.warning,"Editando nuvem · revisão ainda não salva");}
                        else if self.scene.explosion>0.||self.blast.is_some(){ui.colored_label(p.warning,"Vista explodida · apresentação");}
                        ui.add_space(8.);
                        egui::CollapsingHeader::new("Pipeline · etapas").default_open(self.scene.items.is_empty()).show(ui,|ui|{for(tab,label)in["1 · Configurar entrada","2 · Acompanhar processamento","3 · Visualizar / editar resultados"].iter().enumerate(){if ui.button(*label).clicked(){self.pipeline_tab=tab;self.show_reconstruction=true;self.module=Module::Reconstruction;}}});
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
                                    if ui.button("Criar montagem na seleção").on_hover_text("Se uma montagem estiver selecionada, cria uma submontagem dentro dela; caso contrário, cria na raiz.").clicked() {
                                        let parent=self.scene.selected.clone().filter(|id|self.scene.assemblies.iter().any(|g|&g.id==id));
                                        match self.scene.add_assembly(self.new_group_name.clone()) {
                                            Ok(id) => {
                                                if let Some(parent)=parent{let r=self.scene.set_parent(&id,Some(parent));self.notice(r,"Submontagem criada dentro da seleção.");}else{self.notice(Ok(()),"Montagem criada na raiz.");}
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
        if self.analysis_tab {
            self.inspector_tab = 1;
            self.analysis_tab = false;
        }
        egui::SidePanel::right("context_inspector").default_width(320.).width_range(290.0..=440.).resizable(true).frame(egui::Frame::new().fill(p.surface).inner_margin(12)).show(ctx,|ui|{
            ui.horizontal(|ui|{ui.strong("Inspetor");if ui.small_button("×").on_hover_text("Ocultar inspetor").clicked(){self.right_visible=false;}});
            let item=self.scene.selected.as_ref().and_then(|id|self.scene.items.iter().find(|i|&i.id==id));
            let group=self.scene.selected.as_ref().and_then(|id|self.scene.assemblies.iter().find(|g|&g.id==id));
            egui::Frame::group(ui.style()).fill(p.elevated).show(ui,|ui|{ui.set_min_width(ui.available_width());if let Some(i)=item{ui.colored_label(p.primary,if matches!(i.geometry.as_ref(),Geometry::Points(_)){"NUVEM SELECIONADA"}else{"MALHA SELECIONADA"});ui.strong(&i.name);ui.label(if i.visible{"Visível no viewport"}else{"Oculta no viewport"});ui.label(RichText::new(self.selection_path()).size(10.).color(p.secondary));}else if let Some(g)=group{ui.colored_label(p.primary,"MONTAGEM SELECIONADA");ui.strong(&g.name);ui.label(RichText::new(self.selection_path()).size(10.).color(p.secondary));}else{ui.strong("Nenhum objeto selecionado");ui.label("Selecione na estrutura ou clique na geometria.");}});
            ui.horizontal(|ui|{for(tab,label)in["Objeto","Análise","Ações"].iter().enumerate(){ui.selectable_value(&mut self.inspector_tab,tab,*label);}});ui.separator();
            egui::ScrollArea::vertical().id_salt(("inspector_body",self.inspector_tab)).show(ui,|ui|{ui.add_enabled_ui(self.pending.is_none(),|ui|{match self.inspector_tab{
                0=>{if self.blast.is_some(){ui.label("Fragmentação ativa. Restaure para editar o objeto.");if ui.button("Abrir efeitos").clicked(){self.show_effects=true;}}else{ui.label("Somente o objeto identificado acima será alterado.");self.selection_properties(ui);}},
                1=>self.inspection_summary(ui),
                _=>{
                    let cloud=self.scene.selected.as_ref().and_then(|id|self.scene.items.iter().find(|i|&i.id==id)).is_some_and(|i|matches!(i.geometry.as_ref(),Geometry::Points(_)));
                    ui.strong("Geometria selecionada");
                    if ui.add_enabled(cloud,egui::Button::new("Editar / limpar nuvem…")).on_hover_text("Selecionar pontos, recortar, tratar outliers e salvar revisão.").clicked(){if self.cleaning_target().is_none(){let r=self.begin_cleaning();self.notice(r,"Edição de nuvem iniciada.");}self.cleaning.open=true;}
                    if ui.add_enabled(self.scene.selected.is_some(),egui::Button::new("Enquadrar seleção")).clicked(){let r=self.scene.fit(true);self.notice(r,"Seleção enquadrada.");}
                    if ui.add_enabled(self.selected_run().is_some(),egui::Button::new("Resultados desta reconstrução…")).clicked(){self.attach_selected_run();}
                    if ui.add_enabled(self.selected_run().is_some(),egui::Button::new("Gerar superfície desta origem…")).clicked(){self.configure_selected_run();}
                    if let Some(root)=self.selected_run().filter(|p|p.join("exports/sparse.ply").is_file())&&ui.button("Visualizar nuvem esparsa deste run").clicked(){self.open_result(root,true,false);}
                    ui.separator();ui.strong("Montagem e apresentação");ui.label("Organize componentes usando Montagem pai, na aba Objeto.");
                    if ui.button("Desmontar / explosão / estilhaços…").clicked(){self.show_effects=true;}
                    if ui.button("Alinhamento entre peças…").clicked(){self.open_alignment();}
                    if ui.button("Criar exemplo de armadura…").clicked(){self.choose(Purpose::ArmourDemo);}
                    ui.separator();ui.strong("Inspeção e saída");
                    if ui.button("Medir e anotar…").clicked(){self.show_measurements=true;}
                    if ui.add_enabled(self.scene.selected.as_ref().is_some_and(|id|self.scene.items.iter().any(|i|&i.id==id)),egui::Button::new("Exportar objeto montado…")).clicked(){self.choose(Purpose::Export);}
                    if ui.button("Exportar relatório…").clicked(){self.choose(Purpose::ExportMeasurements);}
                    ui.separator();if ui.button("Configurar reconstrução…").clicked(){self.show_reconstruction=true;self.pipeline_tab=0;}
if ui.button("Configurações gerais…").clicked(){self.show_general=true;}
                }
            }});});
        });
    }
    fn selection_path(&self) -> String {
        let mut names = Vec::new();
        let mut parent = self
            .scene
            .selected
            .as_ref()
            .and_then(|id| {
                self.scene
                    .items
                    .iter()
                    .find(|i| &i.id == id)
                    .map(|i| i.parent.as_deref())
                    .or_else(|| {
                        self.scene
                            .assemblies
                            .iter()
                            .find(|g| &g.id == id)
                            .map(|g| g.parent.as_deref())
                    })
            })
            .flatten();
        for _ in 0..64 {
            let Some(id) = parent else { break };
            let Some(g) = self.scene.assemblies.iter().find(|g| g.id == id) else {
                break;
            };
            names.push(g.name.clone());
            parent = g.parent.as_deref();
        }
        names.reverse();
        if names.is_empty() {
            "Workspace / raiz".into()
        } else {
            format!("Workspace / {}", names.join(" / "))
        }
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
                    egui::ComboBox::from_id_salt("viewport_tool")
                        .selected_text(match self.tool {
                            Tool::Select => "Selecionar",
                            Tool::Pan => "Pan",
                            Tool::Orbit => "Órbita",
                        })
                        .width(100.)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.tool, Tool::Select, "Selecionar");
                            ui.selectable_value(&mut self.tool, Tool::Pan, "Pan");
                            ui.selectable_value(&mut self.tool, Tool::Orbit, "Órbita");
                        });
                    ui.separator();
                    let cloud = self
                        .scene
                        .selected
                        .as_ref()
                        .and_then(|id| self.scene.items.iter().find(|i| &i.id == id))
                        .is_some_and(|i| matches!(i.geometry.as_ref(), Geometry::Points(_)));
                    if cloud
                        && ui
                            .button(if self.cleaning_target().is_some() {
                                "Retomar edição"
                            } else {
                                "Editar nuvem"
                            })
                            .clicked()
                    {
                        if self.cleaning_target().is_none() {
                            let r = self.begin_cleaning();
                            self.notice(r, "Edição da nuvem iniciada.");
                        }
                        self.cleaning.open = true;
                    }
                    if ui.button("Desmontagem…").clicked() {
                        self.show_effects = true;
                    }
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
                        if ui
                            .button(if self.maximized {
                                "Restaurar painéis"
                            } else {
                                "Maximizar viewport"
                            })
                            .clicked()
                        {
                            self.maximized = !self.maximized;
                            ui.close();
                        }
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
        match if self.panel_fragments {
            blast::Blast::with_panels(&self.scene, self.blast_selected, self.blast_strength)
        } else if self.blast_detail == 3 {
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
                    &format!(
                        "Prévia: {count} {} · dados originais preservados",
                        if self.panel_fragments {
                            "painéis"
                        } else {
                            "estilhaços"
                        }
                    ),
                );
            }
            Err(e) => self.notice(Err(e), ""),
        }
    }
    pub(super) fn blast_controls(&mut self, ui: &mut egui::Ui) {
        if self.blast_mode {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.panel_fragments, false, "Estilhaços pequenos");
                ui.selectable_value(&mut self.panel_fragments, true, "Painéis de superfície");
            });
            ui.label(if self.panel_fragments{"Agrupa faces originais em placas rígidas; não usa subdivisão fractal. Não cria interior volumétrico."}else{"Estilhaços subdivididos e dispersão radial. Apenas efeito visual."});
            ui.checkbox(&mut self.blast_selected, "Somente seleção");
            ui.add_enabled_ui(!self.panel_fragments, |ui| {
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
                ui.label(format!(
                    "{} {}",
                    b.count(),
                    if self.panel_fragments {
                        "painéis visuais"
                    } else {
                        "estilhaços visuais"
                    }
                ));
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
    pub(super) fn selection_properties(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        if let Some(id) = self.scene.selected.clone() {
            let item = self.scene.items.iter().position(|i| i.id == id);
            let group = self.scene.assemblies.iter().position(|g| g.id == id);
            let (mut placement, parent) = if let Some(index) = item {
                let i = &self.scene.items[index];
                ui.strong("Dados e transformação local");
                let count = match i.geometry.as_ref() {
                    Geometry::Mesh(m) => format!("{} triângulos", m.triangles().len()),
                    Geometry::Points(p) => format!("{} pontos", p.positions().len()),
                };
                ui.label(RichText::new(count).color(palette.primary));
                ui.label(format!(
                    "Unidade: {}",
                    match i.unit.as_str() {
                        "unknown" | "Unknown" => "não definida",
                        "Metre" => "m",
                        "Millimetre" => "mm",
                        other => other,
                    }
                ));
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
            ui.label("Montagem pai");
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
