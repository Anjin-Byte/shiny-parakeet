#[derive(Debug)]
pub enum RenderCommand {
    UpdateSample { value: f64 },
    UpdateRes { value: f64 },
    UpdateBounces { value: u16 },
    UpdateAspect { value: f64 },
    RestartRender,
    Interrupt,
    Pause,
    Wake,
}

pub enum RenderEvent {
    FrameReady,
    FinishedJob,
}