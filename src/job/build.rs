use crate::cmd::Cmd;
use crate::image::{Image, ImageState, ImagesState};
use crate::job::JobCtx;
use crate::recipe::Recipe;
use crate::Config;
use crate::Result;

use futures::StreamExt;
use log::{debug, error, info};
use moby::{
    image::ImageBuildChunk, tty::TtyChunk, BuildOptions, Container, ContainerOptions, Docker,
    ExecContainerOptions, RmContainerOptions,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::str;
use std::time::SystemTime;

#[allow(dead_code)]
pub struct BuildCtx<'j> {
    id: String,
    config: &'j Config,
    image: &'j Image,
    recipe: &'j Recipe,
    docker: &'j Docker,
    image_state: &'j RefCell<ImagesState>,
    bld_dir: PathBuf,
    verbose: bool,
}
impl<'j> BuildCtx<'j> {
    pub fn new(
        config: &'j Config,
        image: &'j Image,
        recipe: &'j Recipe,
        docker: &'j Docker,
        image_state: &'j RefCell<ImagesState>,
        verbose: bool,
    ) -> Self {
        let id = format!(
            "pkger-{}-{}-{}",
            &recipe.metadata.name,
            &image.name,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        debug!("{}", id);
        let bld_dir = PathBuf::from(format!(
            "/tmp/{}-{}",
            &recipe.metadata.name,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));

        BuildCtx {
            id,
            config,
            image,
            recipe,
            docker,
            image_state,
            bld_dir,
            verbose,
        }
    }

    // If successful returns id of the container
    async fn container_spawn(&self, image_state: &ImageState) -> Result<String> {
        let mut env = vec![
            format!("PKGER_BLD_DIR={}", self.bld_dir.display()),
            format!("PKGER_OS={}", image_state.os.as_ref()),
            format!("PKGER_OS_VERSION={}", &image_state.os.os_ver()),
        ];
        if let Some(_env) = &self.recipe.env {
            _env.iter()
                .for_each(|(k, v)| env.push(format!("{}={}", k, v.to_string())));
        }
        debug!("{:?}", env);
        Ok(self
            .docker
            .containers()
            .create(
                &ContainerOptions::builder(&image_state.image)
                    .name(&self.id)
                    .cmd(vec!["sleep infinity"])
                    .entrypoint(vec!["/bin/sh", "-c"])
                    .env(env)
                    .build(),
            )
            .await
            .map(|info| info.id)?)
    }

    async fn container_exec<S: AsRef<str>>(&self, container: &Container<'j>, cmd: S) -> Result<()> {
        let opts = ExecContainerOptions::builder()
            .cmd(vec!["/bin/sh", "-c", cmd.as_ref()])
            .attach_stdout(true)
            .attach_stderr(true)
            .build();

        let mut stream = container.exec(&opts);

        while let Some(result) = stream.next().await {
            match result? {
                TtyChunk::StdOut(chunk) => {
                    if self.verbose {
                        info!("{}", str::from_utf8(&chunk)?);
                    }
                }
                TtyChunk::StdErr(chunk) => {
                    if self.verbose {
                        error!("{}", str::from_utf8(&chunk)?);
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    async fn image_build(&mut self) -> Result<ImageState> {
        debug!("building image {}", &self.image.name);
        let images = self.docker.images();
        let opts = BuildOptions::builder(self.image.path.to_string_lossy().to_string())
            .tag(&format!("{}:latest", &self.image.name))
            .build();

        let mut stream = images.build(&opts);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            match chunk {
                ImageBuildChunk::Error {
                    error,
                    error_detail: _,
                } => {
                    return Err(anyhow!(error.to_string()));
                }
                ImageBuildChunk::Update { stream } => {
                    if self.verbose {
                        info!("{}", stream);
                    }
                }
                ImageBuildChunk::Digest { aux } => {
                    let state = ImageState::new(
                        &aux.id,
                        &self.image.name,
                        "latest",
                        &SystemTime::now(),
                        &self.docker,
                    )
                    .await?;

                    self.image_state
                        .borrow_mut()
                        .update(&self.image.name, &state);

                    return Ok(state);
                }
                _ => {}
            }
        }

        Err(anyhow!("stream ended before image id was received"))
    }

    pub async fn run(&mut self) -> Result<()> {
        if self.verbose {
            info!("running job {}", &self.id);
        }
        let image_state = self
            .image_build()
            .await
            .map_err(|e| anyhow!("failed to build image - {}", e))?;

        if self.verbose {
            info!("image: {}", image_state.image);
        }

        let id = self.container_spawn(&image_state).await?;
        let containers = self.docker.containers();
        let container = containers.get(&id);
        if self.verbose {
            info!("container id: {}", id);
        }

        info!("starting container");
        container.start().await?;

        for step in &self.recipe.build.steps {
            let cmd = Cmd::new(&step)?;
            if let Some(images) = cmd.images {
                if !images.contains(&self.image.name.as_str()) {
                    continue;
                }
            }
            self.container_exec(&container, &cmd.cmd).await?;
        }

        if let Err(e) = container
            .remove(
                &RmContainerOptions::builder()
                    .force(true)
                    .volumes(true)
                    .build(),
            )
            .await
        {
            error!("failed to delete container - {}", e);
        }

        Ok(())
    }
}
impl<'j> From<BuildCtx<'j>> for JobCtx<'j> {
    fn from(ctx: BuildCtx<'j>) -> Self {
        JobCtx::Build(ctx)
    }
}