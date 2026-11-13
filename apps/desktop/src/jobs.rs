use noah_pipeline::Result;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
pub struct ReconstructionJob {
    pub images: PathBuf,
    pub engine: PathBuf,
    pub parent: PathBuf,
    pub name: String,
    pub dense: bool,
    pub mesh: bool,
    pub source_run: Option<PathBuf>,
    pub clean_revision: Option<PathBuf>,
    pub timeout: u64,
    pub output: Option<PathBuf>,
    pub worker: Option<Receiver<Result<()>>>,
    pub status: String,
    pub manifest: Option<noah_pipeline::Manifest>,
    last_refresh: std::time::Instant,
}
impl Default for ReconstructionJob {
    fn default() -> Self {
        Self {
            images: PathBuf::new(),
            engine: PathBuf::new(),
            parent: std::env::current_dir().unwrap_or_default(),
            name: "reconstrucao-001".into(),
            dense: false,
            mesh: true,
            source_run: None,
            clean_revision: None,
            timeout: 600,
            output: None,
            worker: None,
            status: "Escolha fotografias, motor local e pasta de resultados.".into(),
            manifest: None,
            last_refresh: std::time::Instant::now(),
        }
    }
}
impl ReconstructionJob {
    pub fn configured(engine: PathBuf, images: PathBuf, parent: PathBuf) -> Self {
        let name = available_name(&parent, "reconstrucao");
        Self {
            engine,
            images,
            parent,
            name,
            ..Self::default()
        }
    }
    pub fn validation_error(&self) -> Option<String> {
        if self.worker.is_some() {
            return Some("Execução em andamento".into());
        }
        if !self.engine.is_file() {
            return Some("Selecione COLMAP em Configurações gerais".into());
        }
        if !self.parent.is_dir() {
            return Some("Escolha uma pasta de resultados existente".into());
        }
        if self.name.trim().is_empty()
            || std::path::Path::new(&self.name).components().count() != 1
            || matches!(self.name.as_str(), "." | "..")
        {
            return Some("Digite um nome de execução válido".into());
        }
        if self.parent.join(&self.name).exists() {
            return Some("Esse nome já existe. Escolha uma nova revisão".into());
        }
        if let Some(source) = &self.source_run {
            if !source.is_dir() || !self.mesh {
                return Some("Continuar um run exige origem válida e modo de superfície".into());
            }
        } else if !self.images.is_dir() {
            return Some("Selecione a pasta de fotografias".into());
        }
        if self.clean_revision.is_some() && self.dense {
            return Some("Revisão sparse limpa exige superfície rápida CPU".into());
        }
        None
    }
    pub fn ready_inputs(&self) -> bool {
        self.validation_error().is_none()
    }
    pub fn attach(&mut self, root: PathBuf) -> Result<()> {
        if self.worker.is_some() {
            return Err("aguarde a execução atual".into());
        }
        let m = noah_pipeline::inspect(&root)?;
        if m.status == noah_pipeline::Status::Running {
            return Err("run registrado em andamento; aguarde ou verifique seu diagnóstico".into());
        }
        self.status = format!("Execução anterior: {}", status_label(&m.status));
        self.output = Some(root);
        self.manifest = Some(m);
        Ok(())
    }
    pub fn available_name(&self, prefix: &str) -> String {
        available_name(&self.parent, prefix)
    }
    pub fn start(&mut self) -> Result<()> {
        if self.worker.is_some() {
            return Err("reconstrução já em andamento".into());
        }
        if (!self.images.is_dir() && self.source_run.is_none())
            || !self.engine.is_file()
            || !self.parent.is_dir()
        {
            return Err("escolha fotos, executável COLMAP e pasta de resultados".into());
        }
        if std::path::Path::new(&self.name).components().count() != 1
            || self.name.trim().is_empty()
            || matches!(self.name.as_str(), "." | "..")
        {
            return Err("nome de reconstrução inválido".into());
        }
        let output = self.parent.join(&self.name);
        if output.exists() {
            return Err("escolha um novo nome; resultados existentes serão preservados".into());
        }
        let config = noah_photogrammetry::Config {
            images: self.images.clone(),
            engine: self.engine.clone(),
            output: output.clone(),
            dense: self.dense,
            mesh: self.mesh,
            timeout: Duration::from_secs(self.timeout),
        };
        if self.source_run.is_some() && !self.mesh {
            return Err("escolha um modo de malha para continuar um run".into());
        }
        let source = self.source_run.clone();
        let clean = self.clean_revision.clone();
        if clean.is_some() && self.dense {
            return Err("revisão esparsa limpa exige malha CPU".into());
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = if let Some(source) = source {
                let config = noah_photogrammetry::surface::Config {
                    source,
                    engine: config.engine,
                    output: config.output,
                    timeout: config.timeout,
                    method: if config.dense {
                        noah_photogrammetry::surface::Method::PoissonDense
                    } else {
                        noah_photogrammetry::surface::Method::DelaunaySparse
                    },
                };
                if let Some(path) = clean {
                    noah_photogrammetry::surface::from_clean_revision(config, &path)
                } else {
                    noah_photogrammetry::surface::from_run(config)
                }
            } else {
                noah_photogrammetry::reconstruct(config)
            };
            let _ = tx.send(result);
        });
        self.worker = Some(rx);
        self.manifest = None;
        self.output = Some(output);
        self.status = "Reconstrução em andamento. Acompanhe as etapas abaixo.".into();
        Ok(())
    }
    pub fn has_artifact(&self, relative: &str) -> bool {
        self.worker.is_none()
            && self
                .output
                .as_ref()
                .is_some_and(|p| p.join(relative).is_file())
            && self
                .manifest
                .as_ref()
                .is_some_and(|m| m.artifacts.iter().any(|a| a.path == relative))
    }
    pub fn poll(&mut self) {
        if let Some(rx) = &self.worker {
            match rx.try_recv() {
                Ok(result) => {
                    self.worker = None;
                    self.status = match result {
                        Ok(()) => {
                            "Processamento concluído. Abra a malha ou a nuvem nos botões abaixo."
                                .into()
                        }
                        Err(e) => format!("Reconstrução encerrada: {e}"),
                    };
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.worker = None;
                    self.status =
                        "O processamento foi interrompido; consulte os logs preservados.".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
    }
    pub fn cancel(&mut self) -> Result<()> {
        let root = self.output.as_ref().ok_or("nenhuma reconstrução ativa")?;
        noah_pipeline::cancel(root)?;
        self.status = "Cancelamento solicitado. Aguardando encerramento do motor...".into();
        Ok(())
    }
    pub fn refresh(&mut self) {
        let was_running = self.worker.is_some();
        self.poll();
        let finished = was_running && self.worker.is_none();
        if !finished && self.last_refresh.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_refresh = std::time::Instant::now();
        if let Some(root) = &self.output
            && let Ok(bytes) = noah_pipeline::read_bounded(&root.join("run.json"), 16 * 1024 * 1024)
            && let Ok(manifest) = serde_json::from_slice::<noah_pipeline::Manifest>(&bytes)
        {
            self.manifest = Some(manifest);
        }
    }
    pub fn logs_filtered(&self, selected: Option<usize>, stream: usize) -> Result<String> {
        use std::io::{Read, Seek, SeekFrom};
        let root = self.output.as_ref().ok_or("nenhuma execução disponível")?;
        let manifest = self
            .manifest
            .as_ref()
            .ok_or("manifesto ainda indisponível")?;
        let mut text = format!(
            "NOAH · DIAGNÓSTICO DE RECONSTRUÇÃO\nRun: {}\nPasta: {}\nEstado: {}\nMotor: {}\nImagens: {} · Etapas: {}\n",
            manifest.run_id,
            root.display(),
            status_label(&manifest.status),
            manifest
                .engine_version
                .as_deref()
                .unwrap_or("não verificado"),
            manifest.inputs.len(),
            manifest.steps.len()
        );
        if let Some(d) = &manifest.diagnostic {
            text.push_str(&format!("Diagnóstico: {d}\n"));
        }
        for (i, step) in manifest.steps.iter().enumerate() {
            if selected.is_some_and(|s| s != i) {
                continue;
            }
            let duration = step
                .finished_ms
                .map(|end| {
                    format!(
                        "{:.2} s",
                        end.saturating_sub(step.started_ms) as f64 / 1000.
                    )
                })
                .unwrap_or_else(|| "em andamento".into());
            text.push_str(&format!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nETAPA {} · {}\nEstado: {} · Duração: {} · Código: {}\n",i+1,step_label(&step.name),status_label(&step.status),duration,step.exit_code.map(|n|n.to_string()).unwrap_or_else(||"—".into())));
            if let Some(d) = &step.diagnostic {
                text.push_str(&format!("Diagnóstico: {d}\n"));
            }
            for (kind, relative) in [(1, &step.stdout), (2, &step.stderr)] {
                if stream != 0 && stream != kind {
                    continue;
                }
                let path = noah_pipeline::controlled(root, relative)?;
                let mut file = std::fs::File::open(path)?;
                let len = file.metadata()?.len();
                file.seek(SeekFrom::Start(len.saturating_sub(128 * 1024)))?;
                let mut bytes = Vec::new();
                file.take(128 * 1024).read_to_end(&mut bytes)?;
                text.push_str(&format!(
                    "\n{} · {}{}\n",
                    if kind == 1 {
                        "SAÍDA"
                    } else {
                        "DIAGNÓSTICO DO MOTOR"
                    },
                    relative,
                    if len > 128 * 1024 {
                        " [trecho final]"
                    } else {
                        ""
                    }
                ));
                if bytes.is_empty() {
                    text.push_str("(sem mensagens)\n");
                } else {
                    text.push_str(&plain_log(&String::from_utf8_lossy(&bytes)));
                    text.push('\n');
                }
            }
            if text.len() > 2 * 1024 * 1024 {
                text.truncate(text.floor_char_boundary(2 * 1024 * 1024));
                text.push_str("\n[limite de exibição; consulte os arquivos completos]\n");
                break;
            }
        }
        Ok(text)
    }
}

pub fn status_label(status: &noah_pipeline::Status) -> &'static str {
    match status {
        noah_pipeline::Status::Prepared => "Preparando",
        noah_pipeline::Status::Running => "Em execução",
        noah_pipeline::Status::Succeeded => "Concluído",
        noah_pipeline::Status::Failed => "Falhou",
        noah_pipeline::Status::Cancelled => "Cancelado",
    }
}
pub fn step_label(name: &str) -> &'static str {
    match name {
        "version" => "Verificar motor",
        "features" => "Extrair características",
        "matching" => "Encontrar correspondências",
        "mapping" => "Reconstruir câmeras e pontos",
        "undistort" => "Corrigir distorções",
        "patch_match" => "Calcular profundidades",
        "fusion" => "Fundir nuvem densa",
        "meshing" => "Gerar superfície",
        "restore_model" => "Reutilizar câmeras e pontos",
        _ => "Exportar modelo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_refreshes_final_manifest_without_waiting_for_next_frame() {
        let root =
            std::env::temp_dir().join(format!("noah-ui-job-{:032x}", noah_storage::new_id()));
        std::fs::create_dir(&root).unwrap();
        let manifest = noah_pipeline::Manifest {
            schema_version: 1,
            run_id: "test".into(),
            started_ms: 0,
            finished_ms: Some(1),
            status: noah_pipeline::Status::Succeeded,
            engine: "test".into(),
            engine_version: None,
            parameters: Default::default(),
            inputs: vec![],
            steps: vec![],
            artifacts: vec![],
            diagnostic: None,
        };
        std::fs::write(
            root.join("run.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let mut job = ReconstructionJob {
            output: Some(root.clone()),
            worker: Some(rx),
            ..Default::default()
        };
        tx.send(Ok(())).unwrap();
        job.refresh();
        assert!(job.worker.is_none());
        assert_eq!(
            job.manifest.unwrap().status,
            noah_pipeline::Status::Succeeded
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

fn plain_log(text: &str) -> String {
    let mut out = String::new();
    let mut escape = false;
    let mut csi = false;
    for ch in text.chars() {
        if escape {
            if ch == '[' && !csi {
                csi = true;
                continue;
            }
            if !csi || ch.is_ascii_alphabetic() {
                escape = false;
                csi = false;
            }
            continue;
        }
        if ch == '\u{1b}' {
            escape = true;
            continue;
        }
        if ch == '\r' {
            out.push('\n');
        } else if ch == '\n' || ch == '\t' || !ch.is_control() {
            out.push(ch);
        }
    }
    out
}

pub fn available_name(parent: &std::path::Path, prefix: &str) -> String {
    for i in 1..=1000 {
        let name = format!("{prefix}-{i:03}");
        if !parent.join(&name).exists() {
            return name;
        }
    }
    format!("{prefix}-novo")
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    #[test]
    fn logs_format_select_steps_streams_normalize_and_mark_limits() {
        let root = std::env::temp_dir().join(format!("noah-logs-{:032x}", noah_storage::new_id()));
        std::fs::create_dir(&root).unwrap();
        let step = |name: &str| noah_pipeline::Step {
            name: name.into(),
            arguments: vec![],
            started_ms: 1000,
            finished_ms: Some(2500),
            exit_code: Some(0),
            status: noah_pipeline::Status::Succeeded,
            diagnostic: None,
            stdout: format!("{name}.out"),
            stderr: format!("{name}.err"),
        };
        for name in ["features", "mapping"] {
            std::fs::write(
                root.join(format!("{name}.out")),
                format!("\u{1b}[32m{name}\u{1b}[0m\rprogress"),
            )
            .unwrap();
            std::fs::write(root.join(format!("{name}.err")), "aviso útil").unwrap();
        }
        let mut job = ReconstructionJob {
            output: Some(root.clone()),
            manifest: Some(noah_pipeline::Manifest {
                schema_version: 1,
                run_id: "test".into(),
                started_ms: 0,
                finished_ms: Some(3000),
                status: noah_pipeline::Status::Succeeded,
                engine: "colmap".into(),
                engine_version: Some("3.11.1".into()),
                parameters: Default::default(),
                inputs: vec![],
                steps: vec![step("features"), step("mapping")],
                artifacts: vec![],
                diagnostic: None,
            }),
            ..Default::default()
        };
        let full = job.logs_filtered(None, 0).unwrap();
        assert!(full.contains("ETAPA 1 · Extrair características"));
        assert!(full.contains("1.50 s"));
        assert!(full.contains("aviso útil"));
        assert!(!full.contains('\u{1b}'));
        assert!(!full.contains('\r'));
        let filtered = job.logs_filtered(Some(1), 1).unwrap();
        assert!(!filtered.contains("ETAPA 1"));
        assert!(!filtered.contains("aviso útil"));
        assert!(filtered.contains("mapping"));
        std::fs::write(root.join("mapping.out"), "x".repeat(150000)).unwrap();
        assert!(
            job.logs_filtered(Some(1), 1)
                .unwrap()
                .contains("[trecho final]")
        );
        job.manifest.as_mut().unwrap().steps[0].stdout = "../escape".into();
        assert!(job.logs_filtered(None, 0).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn preflight_explains_conflicting_destination_and_missing_inputs() {
        let mut j = ReconstructionJob::default();
        assert!(j.validation_error().unwrap().contains("COLMAP"));
        j.engine = std::env::current_exe().unwrap();
        j.images = std::env::temp_dir();
        j.parent = std::env::temp_dir();
        j.name = format!("noah-not-created-{:032x}", noah_storage::new_id());
        assert!(j.ready_inputs());
        std::fs::create_dir(j.parent.join(&j.name)).unwrap();
        assert!(j.validation_error().unwrap().contains("já existe"));
        std::fs::remove_dir(j.parent.join(&j.name)).unwrap();
        j.clean_revision = Some("clean".into());
        j.dense = true;
        assert!(j.validation_error().unwrap().contains("CPU"));
    }
}
