//! Measurement UI; assembled-state calculations remain in workspace.
use super::*;
use noah_workspace::inspection::{Bounds, Dimensions};
use std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MeasurementTool {
    Distance,
    Angle,
    Dimensions,
}
impl MeasurementTool {
    pub fn points(self) -> usize {
        match self {
            Self::Distance => 2,
            Self::Angle => 3,
            Self::Dimensions => 0,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Distance => "Distância",
            Self::Angle => "Ângulo",
            Self::Dimensions => "Dimensões XYZ",
        }
    }
}
#[derive(Default)]
pub struct BoundsCache {
    stamp: Vec<(String, Option<String>, usize, noah_geometry::Transform)>,
    groups: Vec<(String, Option<String>)>,
    entries: HashMap<String, std::result::Result<Bounds, String>>,
}
impl BoundsCache {
    fn sync(&mut self, scene: &Scene) {
        let Ok(ts) = scene.world_transforms(false) else {
            self.entries.clear();
            self.stamp.clear();
            return;
        };
        let stamp: Vec<_> = scene
            .items
            .iter()
            .zip(ts)
            .map(|(i, t)| {
                (
                    i.id.clone(),
                    i.parent.clone(),
                    std::sync::Arc::as_ptr(&i.geometry) as usize,
                    t,
                )
            })
            .collect();
        let groups: Vec<_> = scene
            .assemblies
            .iter()
            .map(|g| (g.id.clone(), g.parent.clone()))
            .collect();
        if stamp != self.stamp || groups != self.groups {
            self.stamp = stamp;
            self.groups = groups;
            self.entries.clear();
        }
    }
    fn raw(&mut self, scene: &Scene, id: &str) -> std::result::Result<Bounds, String> {
        self.entries
            .entry(id.into())
            .or_insert_with(|| scene.raw_bounds(id).map_err(|e| e.to_string()))
            .clone()
    }
    fn dimensions(&mut self, scene: &Scene, id: &str) -> std::result::Result<Dimensions, String> {
        self.raw(scene, id)?
            .dimensions(scene.scale_factor().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
}
impl Desktop {
    pub(super) fn inspection_summary(&mut self, ui: &mut egui::Ui) {
        let p = self.palette();
        ui.heading("Medições");
        ui.label("Distâncias, ângulos e dimensões XYZ.");
        ui.label(format!(
            "{} distâncias · {} ângulos · {} caixas",
            self.scene.measurements.len(),
            self.scene.angles.len(),
            self.scene.dimension_measurements.len()
        ));
        match self.scene.scale_factor() {
            Ok(Some((_, u))) => {
                ui.label(format!("Escala declarada em {}", u.symbol()));
            }
            Ok(None) => {
                ui.colored_label(p.warning, "Escala física não definida");
            }
            Err(e) => {
                ui.colored_label(p.warning, e.to_string());
            }
        }
        ui.add_space(8.);
        if ui.button("Abrir ferramentas de medida").clicked() {
            self.restore_blast();
            self.show_measurements = true;
        }
        if ui.button("Exportar relatório…").clicked() {
            self.choose(Purpose::ExportMeasurements);
        }
        self.bounds_cache.sync(&self.scene);
        if let Some(id) = self.scene.selected.clone() {
            egui::CollapsingHeader::new("Dimensões da seleção")
                .default_open(true)
                .show(ui, |ui| {
                    match self.bounds_cache.dimensions(&self.scene, &id) {
                        Ok(d) => dimensions_text(ui, &d, self.scene.view.decimals),
                        Err(e) => {
                            ui.colored_label(p.warning, e);
                        }
                    }
                    ui.label(
                        RichText::new("Caixa XYZ mundial · montado · inclui filhos ocultos")
                            .size(10.)
                            .color(p.secondary),
                    );
                    if ui.button("Registrar dimensões…").clicked() {
                        self.measurement_tool = MeasurementTool::Dimensions;
                        self.measure_name = "Dimensões 1".into();
                        self.anchors.clear();
                        self.show_measurements = true;
                    }
                });
        } else {
            ui.label("Selecione uma peça ou montagem para ver suas dimensões.");
        }
        ui.add_space(12.);
        ui.label(RichText::new("Escala declarada não garante exatidão física. Incerteza desconhecida. Área e volume ainda não disponíveis.").size(11.).color(p.secondary));
    }
    pub(super) fn measurement_window(&mut self, ctx: &egui::Context) {
        self.bounds_cache.sync(&self.scene);
        let p = self.palette();
        let mut open = self.show_measurements;
        let mut pick = false;
        egui::Window::new("Medições e referência de escala").open(&mut open).default_width(510.).default_pos([300.,90.]).show(ctx,|ui|{
   egui::ScrollArea::vertical().id_salt("measurement_scroll").max_height((ctx.content_rect().height()-170.).max(240.)).show(ui,|ui|{
    ui.horizontal(|ui|{for tool in [MeasurementTool::Distance,MeasurementTool::Angle,MeasurementTool::Dimensions]{if ui.selectable_label(self.measurement_tool==tool,tool.name()).clicked(){self.measurement_tool=tool;self.measure_mode=false;self.anchors.clear();self.measure_name=format!("{} 1",tool.name());}}});
    ui.add_space(8.);ui.label(RichText::new("Cálculos no estado montado; explosão não altera valores.").size(11.).color(p.secondary));
    if self.measurement_tool==MeasurementTool::Dimensions{self.dimension_editor(ui);}else{
     if self.measurement_tool==MeasurementTool::Angle{ui.label("Ângulo ABC: B é o vértice. Resultado de 0 a 180 graus.");}
     ui.horizontal(|ui|{
      if ui.button(if self.measurement_tool==MeasurementTool::Angle{"Escolher A, B e C"}else{"Escolher A e B"}).clicked(){self.stop_transition();self.anchors.clear();self.measure_mode=true;self.hover_anchor=None;pick=true;self.notice(Ok(()),"Clique nos vértices na ordem indicada. B é o vértice do ângulo.");}
      if ui.button("Limpar seleção").clicked(){self.measure_mode=false;self.anchors.clear();}
     });
     egui::CollapsingHeader::new("Seleção exata / coordenadas").show(ui,|ui|{
      ui.label("Selecione uma peça e informe índices originais (começam em 1).");
      ui.horizontal(|ui|{
       for (label,index) in [("A",&mut self.exact_a),("B",&mut self.exact_b)]{ui.label(label);ui.add(egui::DragValue::new(index).range(1..=2_000_000));}
       if self.measurement_tool==MeasurementTool::Angle{ui.label("C");ui.add(egui::DragValue::new(&mut self.exact_c).range(1..=2_000_000));}
      });
      if ui.button("Usar estes vértices").clicked(){self.exact_vertices();}
      ui.label(RichText::new("Use câmera ortográfica e filtro de seleção para controlar o snap.").size(10.).color(p.secondary));
     });
     for (label,a) in ["A","B","C"].iter().zip(&self.anchors){
      let name=self.scene.items.iter().find(|i|i.id==a.item_id).map(|i|i.name.as_str()).unwrap_or("Fonte ausente");
      ui.label(format!("{label}: {name} / vértice {}",a.vertex+1));
      if let Ok(pos)=self.scene.anchor_position(a,false){ui.monospace(format!("[{value0:.n$}, {value1:.n$}, {value2:.n$}] u. cena",value0=pos[0],value1=pos[1],value2=pos[2],n=self.scene.view.decimals));}
     }
     if self.anchors.len()==self.measurement_tool.points(){
      let a=self.anchors[0].clone();let b=self.anchors[1].clone();
      if self.measurement_tool==MeasurementTool::Angle{
       let c=self.anchors[2].clone();match self.scene.angle_degrees(&a,&b,&c){Ok(value)=>{ui.heading(format!("{:.*}°",self.scene.view.decimals,value));ui.horizontal(|ui|{ui.label("Nome");ui.text_edit_singleline(&mut self.measure_name);});if ui.button("Registrar ângulo").clicked(){let r=self.scene.add_angle(a,b,c,self.measure_name.clone());self.register_result(r);}},Err(e)=>{ui.colored_label(p.error,e.to_string());}}
      }else{self.distance_editor(ui,a,b);}
     }
    }
    ui.separator();self.scale_status(ui);
    ui.label(RichText::new("Incerteza desconhecida. Casas decimais não representam exatidão física.").size(10.).color(p.secondary));
    self.measurement_list(ui);
    let has=!self.scene.measurements.is_empty()||!self.scene.angles.is_empty()||!self.scene.dimension_measurements.is_empty();
    if ui.add_enabled(has,egui::Button::new("Exportar relatório de medições")).clicked(){self.choose(Purpose::ExportMeasurements);}
    ui.label(RichText::new("Salvar revisão preserva âncoras, caixas e escala (sessão 5).").size(10.).color(p.secondary));
   });
  });
        self.show_measurements = open && !pick;
        if !open {
            self.measure_mode = false;
        }
    }
    fn exact_vertices(&mut self) {
        let Some(item) = self
            .scene
            .items
            .iter()
            .find(|i| Some(&i.id) == self.scene.selected.as_ref())
        else {
            self.notice(
                Err("Selecione uma peça, não uma montagem, na árvore.".into()),
                "",
            );
            return;
        };
        let indices = [self.exact_a, self.exact_b, self.exact_c];
        let anchors: Vec<_> = indices[..self.measurement_tool.points()]
            .iter()
            .map(|i| Anchor {
                item_id: item.id.clone(),
                vertex: i.saturating_sub(1),
            })
            .collect();
        let valid = if anchors.len() == 3 {
            self.scene
                .angle_degrees(&anchors[0], &anchors[1], &anchors[2])
                .map(|_| ())
        } else {
            self.scene
                .raw_distance(&anchors[0], &anchors[1])
                .and_then(|_| {
                    if anchors[0] != anchors[1] {
                        Ok(())
                    } else {
                        Err("Escolha vértices distintos".into())
                    }
                })
        };
        match valid {
            Ok(()) => {
                self.anchors = anchors;
                self.measure_mode = false;
                self.notice(Ok(()), "Vértices selecionados por índice original.");
            }
            Err(e) => self.notice(Err(e), ""),
        }
    }
    fn register_result(&mut self, result: noah_workspace::Result<String>) {
        match result {
            Ok(_) => {
                self.dirty = true;
                self.notice(Ok(()), "Medição registrada com incerteza desconhecida.");
            }
            Err(e) => self.notice(Err(e), ""),
        }
    }
    fn distance_editor(&mut self, ui: &mut egui::Ui, a: Anchor, b: Anchor) {
        let p = self.palette();
        match self.scene.distance(&a, &b) {
            Ok(d) => {
                ui.heading(format!(
                    "{:.*} {}",
                    self.scene.view.decimals,
                    d.value,
                    d.unit.map(|u| u.symbol()).unwrap_or("u. cena")
                ));
            }
            Err(e) => {
                ui.colored_label(p.error, e.to_string());
            }
        }
        if let (Ok(pa), Ok(pb)) = (
            self.scene.anchor_position(&a, false),
            self.scene.anchor_position(&b, false),
        ) {
            let (f, u) = self
                .scene
                .scale_factor()
                .ok()
                .flatten()
                .map(|(f, u)| (f, u.symbol()))
                .unwrap_or((1., "u. cena"));
            let d = std::array::from_fn::<_, 3, _>(|i| (pb[i] - pa[i]) * f);
            ui.monospace(format!(
                "ΔX {x:.n$} · ΔY {y:.n$} · ΔZ {z:.n$} {u}",
                x = d[0],
                y = d[1],
                z = d[2],
                n = self.scene.view.decimals
            ));
        }
        ui.horizontal(|ui| {
            ui.label("Nome");
            ui.text_edit_singleline(&mut self.measure_name);
        });
        if ui.button("Registrar distância").clicked() {
            let r = self
                .scene
                .add_measurement(a.clone(), b.clone(), self.measure_name.clone());
            self.register_result(r);
        }
        egui::CollapsingHeader::new("Definir escala por distância conhecida")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.known_distance)
                            .speed(0.1)
                            .range(1e-9..=1e12),
                    );
                    egui::ComboBox::from_id_salt("length_unit")
                        .selected_text(self.length_unit.symbol())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.length_unit,
                                LengthUnit::Millimetre,
                                "mm",
                            );
                            ui.selectable_value(&mut self.length_unit, LengthUnit::Metre, "m");
                        });
                });
                ui.label("Origem / método da referência");
                ui.text_edit_singleline(&mut self.scale_provenance);
                if ui.button("Aplicar referência de escala").clicked() {
                    let r = self.scene.calibrate(
                        a,
                        b,
                        self.known_distance,
                        self.length_unit,
                        self.scale_provenance.clone(),
                    );
                    if r.is_ok() {
                        self.dirty = true;
                    }
                    self.notice(r, "Escala declarada; geometria preservada.");
                }
            });
    }
    fn dimension_editor(&mut self, ui: &mut egui::Ui) {
        let p = self.palette();
        ui.label("Caixa alinhada aos eixos mundiais XYZ. Inclui todos os filhos, mesmo ocultos.");
        if let Some(id) = self.scene.selected.clone() {
            match self.bounds_cache.dimensions(&self.scene, &id) {
                Ok(d) => {
                    dimensions_text(ui, &d, self.scene.view.decimals);
                    ui.label(format!(
                        "{} vértices · {} peças",
                        d.bounds.vertex_count, d.bounds.item_count
                    ));
                    ui.horizontal(|ui| {
                        ui.label("Nome");
                        ui.text_edit_singleline(&mut self.measure_name);
                    });
                    if ui.button("Registrar dimensões XYZ").clicked() {
                        let r = self.scene.add_dimensions(id, self.measure_name.clone());
                        self.register_result(r);
                    }
                }
                Err(e) => {
                    ui.colored_label(p.warning, e);
                }
            }
        } else {
            ui.label("Selecione uma peça ou montagem na árvore.");
        }
        ui.label(
            RichText::new(
                "A caixa muda com a rotação. Não é a menor caixa orientada e não mede volume.",
            )
            .size(10.)
            .color(p.secondary),
        );
    }
    fn scale_status(&mut self, ui: &mut egui::Ui) {
        let p = self.palette();
        match self.scene.scale_factor() {
            Ok(Some((_, u))) => {
                ui.label(format!("Escala declarada em {}", u.symbol()));
            }
            Ok(None) => {
                ui.label("Escala desconhecida: comprimentos em unidades da cena.");
            }
            Err(e) => {
                ui.colored_label(p.warning, e.to_string());
            }
        }
        if self.scene.calibration.is_some() && ui.button("Remover referência de escala").clicked()
        {
            self.scene.calibration = None;
            self.dirty = true;
        }
    }
    fn measurement_list(&mut self, ui: &mut egui::Ui) {
        let p = self.palette();
        let mut remove = None;
        egui::CollapsingHeader::new("Medições registradas")
            .default_open(true)
            .show(ui, |ui| {
                for m in &self.scene.measurements {
                    ui.push_id(&m.id, |ui| {
                        ui.horizontal(|ui| {
                            let label = match self.scene.distance(&m.a, &m.b) {
                                Ok(d) => format!(
                                    "{}: {:.*} {}",
                                    m.name,
                                    self.scene.view.decimals,
                                    d.value,
                                    d.unit.map(|u| u.symbol()).unwrap_or("u. cena")
                                ),
                                Err(_) => format!("{}: recalibrar escala", m.name),
                            };
                            ui.label(label);
                            if ui.small_button("Remover").clicked() {
                                remove = Some(m.id.clone());
                            }
                        });
                    });
                }
                for m in &self.scene.angles {
                    ui.push_id(&m.id, |ui| {
                        ui.horizontal(|ui| {
                            match self.scene.angle_degrees(&m.a, &m.b, &m.c) {
                                Ok(v) => {
                                    ui.label(format!(
                                        "{}: {:.*}°",
                                        m.name, self.scene.view.decimals, v
                                    ));
                                }
                                Err(e) => {
                                    ui.colored_label(p.warning, e.to_string());
                                }
                            }
                            if ui.small_button("Remover").clicked() {
                                remove = Some(m.id.clone());
                            }
                        });
                    });
                }
                for m in &self.scene.dimension_measurements {
                    ui.push_id(&m.id, |ui| {
                        ui.label(&m.name);
                        match self.bounds_cache.dimensions(&self.scene, &m.target_id) {
                            Ok(d) => dimensions_text(ui, &d, self.scene.view.decimals),
                            Err(e) => {
                                ui.colored_label(p.warning, e);
                            }
                        }
                        if ui.small_button("Remover").clicked() {
                            remove = Some(m.id.clone());
                        }
                        ui.separator();
                    });
                }
            });
        if let Some(id) = remove {
            self.scene.measurements.retain(|m| m.id != id);
            self.scene.angles.retain(|m| m.id != id);
            self.scene.dimension_measurements.retain(|m| m.id != id);
            self.dirty = true;
        }
    }
    pub(super) fn draw_inspection(&mut self, display: &Scene, painter: &egui::Painter, rect: Rect) {
        let p = self.palette();
        let size = [rect.width() as f64, rect.height() as f64];
        let Ok(transforms) = display.world_transforms(true) else {
            return;
        };
        let project = |point: [f64; 3]| {
            display
                .camera
                .project(point, size)
                .map(|q| Pos2::new(rect.left() + q[0] as f32, rect.top() + q[1] as f32))
        };
        let anchor = |a: &Anchor| {
            display
                .items
                .iter()
                .position(|i| i.id == a.item_id && i.visible)
                .and_then(|i| {
                    display.items[i]
                        .geometry
                        .positions()
                        .get(a.vertex)
                        .and_then(|&v| transforms[i].apply(v).ok())
                })
                .and_then(project)
        };
        for m in &self.scene.angles {
            if let (Some(a), Some(b), Some(c)) = (anchor(&m.a), anchor(&m.b), anchor(&m.c)) {
                painter.line_segment([a, b], Stroke::new(1.5_f32, p.warning));
                painter.line_segment([b, c], Stroke::new(1.5_f32, p.warning));
                painter.circle_filled(b, 4., p.warning);
                let label = match self.scene.angle_degrees(&m.a, &m.b, &m.c) {
                    Ok(v) => format!("{}: {:.*}° (montado)", m.name, self.scene.view.decimals, v),
                    Err(_) => format!("{}: revisar vértices", m.name),
                };
                painter.text(
                    b + Vec2::new(10., -12.),
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    FontId::proportional(11.),
                    p.warning,
                );
            }
        }
        if display.explosion > 1e-9 {
            return;
        }
        self.bounds_cache.sync(&self.scene);
        for m in &self.scene.dimension_measurements {
            if !display.items.iter().enumerate().any(|(i, item)| {
                item.visible && (item.id == m.target_id || display.belongs_to(i, &m.target_id))
            }) {
                continue;
            }
            let Ok(d) = self.bounds_cache.dimensions(&self.scene, &m.target_id) else {
                continue;
            };
            let b = &d.bounds;
            let vertices: Vec<_> = (0..8)
                .map(|mask| {
                    std::array::from_fn(|k| {
                        if mask & (1 << k) == 0 {
                            b.min[k]
                        } else {
                            b.max[k]
                        }
                    })
                })
                .collect();
            for mask in 0..8 {
                for axis in 0..3 {
                    if mask & (1 << axis) == 0
                        && let (Some(a), Some(b)) = (
                            project(vertices[mask]),
                            project(vertices[mask | (1 << axis)]),
                        )
                    {
                        painter.line_segment(
                            [a, b],
                            Stroke::new(1.0_f32, p.primary.gamma_multiply(0.6)),
                        );
                    }
                }
            }
            if let Some(q) = project(b.max) {
                painter.text(
                    q + Vec2::new(8., -10.),
                    egui::Align2::LEFT_BOTTOM,
                    &m.name,
                    FontId::proportional(11.),
                    p.primary,
                );
            }
            for axis in 0..3 {
                let middle = std::array::from_fn(|k| {
                    if k == axis {
                        (b.min[k] + b.max[k]) * 0.5
                    } else {
                        b.min[k]
                    }
                });
                if let Some(q) = project(middle) {
                    painter.text(
                        q - Vec2::new(0., 14.),
                        egui::Align2::CENTER_BOTTOM,
                        format!(
                            "{}: {:.*} {}",
                            ["X", "Y", "Z"][axis],
                            self.scene.view.decimals,
                            d.size[axis],
                            d.unit.map(|u| u.symbol()).unwrap_or("u. cena")
                        ),
                        FontId::proportional(11.),
                        p.primary,
                    );
                }
            }
        }
    }
}
fn dimensions_text(ui: &mut egui::Ui, d: &Dimensions, decimals: usize) {
    for (axis, value) in ["X", "Y", "Z"].iter().zip(d.size) {
        ui.monospace(format!(
            "{axis}  {value:.decimals$} {}",
            d.unit.map(|u| u.symbol()).unwrap_or("u. cena")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_cache_tracks_geometry_transforms_hierarchy_and_scale() {
        let mut s = Scene::default();
        s.add_source(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/datasets/inspection-box.obj"),
        )
        .unwrap();
        let id = s.items[0].id.clone();
        let a = s.add_assembly("A".into()).unwrap();
        let b = s.add_assembly("B".into()).unwrap();
        s.set_parent(&id, Some(a.clone())).unwrap();
        let mut cache = BoundsCache::default();
        cache.sync(&s);
        assert_eq!(cache.dimensions(&s, &a).unwrap().size, [2., 3., 4.]);
        s.set_placement(
            0,
            Placement {
                scale: 2.,
                ..Default::default()
            },
        )
        .unwrap();
        cache.sync(&s);
        assert_eq!(cache.dimensions(&s, &a).unwrap().size, [4., 6., 8.]);
        assert!(cache.dimensions(&s, &b).is_err());
        s.set_parent(&a, Some(b.clone())).unwrap();
        cache.sync(&s);
        assert_eq!(cache.dimensions(&s, &b).unwrap().size, [4., 6., 8.]);
        s.set_parent(&id, Some(b.clone())).unwrap();
        cache.sync(&s);
        assert!(cache.dimensions(&s, &a).is_err());
        assert!(cache.dimensions(&s, &b).is_ok());
        s.calibrate(
            Anchor {
                item_id: id.clone(),
                vertex: 0,
            },
            Anchor {
                item_id: id,
                vertex: 1,
            },
            200.,
            LengthUnit::Millimetre,
            "Teste".into(),
        )
        .unwrap();
        assert_eq!(cache.dimensions(&s, &b).unwrap().size, [200., 300., 400.]);
        s.calibration = None;
        s.items[0].geometry = std::sync::Arc::new(Geometry::Points(
            noah_geometry::PointCloud::new(vec![[0., 0., 0.], [5., 0., 0.]], None).unwrap(),
        ));
        cache.sync(&s);
        assert_eq!(cache.dimensions(&s, &b).unwrap().size, [10., 0., 0.]);
    }
}
