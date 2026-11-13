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
    pub fn ready_inputs(&self) -> bool {
        self.worker.is_none()
            && self.engine.is_file()
            && self.parent.is_dir()
            && if self.source_run.is_some() {
                self.mesh
            } else {
                self.images.is_dir()
            }
    }
    pub fn available_name(&self, prefix: &str) -> String {
        for i in 1..=1000 {
            let s = format!("{prefix}-{i:03}");
            if !self.parent.join(&s).exists() {
                return s;
            }
        }
        format!("{prefix}-novo")
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
    pub fn logs(&self) -> Result<String> {
        use std::io::{Read, Seek, SeekFrom};
        let root = self.output.as_ref().ok_or("sem resultado")?;
        let manifest = self.manifest.as_ref().ok_or("sem etapas registradas")?;
        let mut text = String::new();
        for step in &manifest.steps {
            for stream in [&step.stdout, &step.stderr] {
                let path = noah_pipeline::controlled(root, stream)?;
                let mut file = std::fs::File::open(path)?;
                let len = file.metadata()?.len();
                file.seek(SeekFrom::Start(len.saturating_sub(4096)))?;
                let mut bytes = Vec::new();
                file.take(4096).read_to_end(&mut bytes)?;
                text.push_str(&format!(
                    "\n--- {} / {} ---\n{}",
                    step.name,
                    stream,
                    String::from_utf8_lossy(&bytes)
                ));
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
