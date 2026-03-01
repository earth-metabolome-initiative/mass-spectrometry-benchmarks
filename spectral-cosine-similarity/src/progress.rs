use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub const FIXED_STAGE_UNITS: u64 = 1;
pub const NON_COMPUTE_STAGE_COUNT: u64 = 4;

pub trait StageProgress {
    fn set_substep(&mut self, message: &str);
    fn clear_substep(&mut self);
    fn inc(&mut self, units: u64);
}

pub struct PipelineProgress {
    total_units: u64,
    completed_units: u64,
    finished: bool,
    _multi: MultiProgress,
    overall: ProgressBar,
    substep: ProgressBar,
}

impl PipelineProgress {
    pub fn new(total_units: u64) -> Self {
        let total_units = total_units.max(1);
        let multi = MultiProgress::new();

        let overall = multi.add(ProgressBar::new(total_units));
        overall.set_style(
            ProgressStyle::with_template("[pipeline] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .expect("invalid overall progress style")
                .progress_chars("=>-"),
        );
        overall.set_message("starting");

        let substep = multi.add(ProgressBar::new(1));
        substep.set_style(
            ProgressStyle::with_template("  {msg}").expect("invalid substep progress style"),
        );
        substep.set_message("");

        Self {
            total_units,
            completed_units: 0,
            finished: false,
            _multi: multi,
            overall,
            substep,
        }
    }

    pub fn start_stage<'a>(
        &'a mut self,
        stage_name: &'static str,
        stage_units: u64,
    ) -> StageHandle<'a> {
        self.overall.set_message(format!("[{stage_name}]"));
        StageHandle {
            pipeline: self,
            stage_name,
            stage_units,
            completed_stage_units: 0,
            finished: false,
        }
    }

    pub fn add_total_units(&mut self, additional_units: u64) {
        if additional_units == 0 {
            return;
        }
        self.total_units = self.total_units.saturating_add(additional_units);
        self.overall.set_length(self.total_units);
    }

    pub fn finish_all(&mut self) {
        if self.finished {
            return;
        }

        let remaining = self.total_units.saturating_sub(self.completed_units);
        if remaining > 0 {
            self.overall.inc(remaining);
            self.completed_units += remaining;
        }

        self.substep.finish_and_clear();
        self.overall.finish_with_message("[done]");
        self.finished = true;
    }
}

impl Drop for PipelineProgress {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.substep.finish_and_clear();
        self.overall.finish_and_clear();
    }
}

pub struct StageHandle<'a> {
    pipeline: &'a mut PipelineProgress,
    stage_name: &'static str,
    stage_units: u64,
    completed_stage_units: u64,
    finished: bool,
}

impl StageHandle<'_> {
    pub fn finish(&mut self) {
        if self.finished {
            return;
        }

        let remaining = self.stage_units.saturating_sub(self.completed_stage_units);
        if remaining > 0 {
            self.inc(remaining);
        }
        self.clear_substep();
        self.pipeline
            .overall
            .set_message(format!("[{} done]", self.stage_name));
        self.finished = true;
    }
}

impl StageProgress for StageHandle<'_> {
    fn set_substep(&mut self, message: &str) {
        self.pipeline.substep.set_message(message.to_string());
    }

    fn clear_substep(&mut self) {
        self.pipeline.substep.set_message(String::new());
    }

    fn inc(&mut self, units: u64) {
        let remaining = self.stage_units.saturating_sub(self.completed_stage_units);
        let delta = units.min(remaining);
        if delta == 0 {
            return;
        }
        self.completed_stage_units += delta;

        let overall_remaining = self
            .pipeline
            .total_units
            .saturating_sub(self.pipeline.completed_units);
        let overall_delta = delta.min(overall_remaining);
        if overall_delta == 0 {
            return;
        }
        self.pipeline.completed_units += overall_delta;
        self.pipeline.overall.inc(overall_delta);
    }
}

impl Drop for StageHandle<'_> {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_finish_consumes_remaining_stage_units() {
        let mut progress = PipelineProgress::new(10);
        {
            let mut stage = progress.start_stage("Test", 4);
            stage.inc(2);
        }

        assert_eq!(progress.completed_units, 4);
    }

    #[test]
    fn add_total_units_extends_pipeline_length() {
        let mut progress = PipelineProgress::new(2);
        progress.add_total_units(5);

        assert_eq!(progress.total_units, 7);
    }

    #[test]
    fn finish_all_consumes_remaining_units() {
        let mut progress = PipelineProgress::new(3);
        {
            let mut stage = progress.start_stage("Test", 1);
            stage.inc(1);
        }

        progress.finish_all();
        assert_eq!(progress.completed_units, progress.total_units);
    }
}
