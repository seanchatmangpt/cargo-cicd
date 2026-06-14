//! `cargo cicd ui` — preview and exercise the terminal design system.
//!
//! STUB IMPLEMENTATION — frozen noun/verb surface. The owning agent builds the
//! full component showcase (`demo`) and wires adapters into the live dashboard
//! (`dashboard`). Public help text MUST stay within the public boundary (no
//! internal/forbidden terms).

use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct UiNoun;

impl UiNoun {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UiNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for UiNoun {
    fn name(&self) -> &'static str {
        "ui"
    }
    fn about(&self) -> &'static str {
        "Preview the cargo-cicd terminal UI design system"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(UiDemoVerb), Box::new(UiDashboardVerb)]
    }
}

pub struct UiDemoVerb;

impl VerbCommand for UiDemoVerb {
    fn name(&self) -> &'static str {
        "demo"
    }
    fn about(&self) -> &'static str {
        "Showcase the UI components: styles, badges, tables, panels, charts"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // STUB: agent builds the full showcase.
        println!("cargo-cicd ui demo");
        Ok(())
    }
}

pub struct UiDashboardVerb;

impl VerbCommand for UiDashboardVerb {
    fn name(&self) -> &'static str {
        "dashboard"
    }
    fn about(&self) -> &'static str {
        "Render the workspace status dashboard"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // STUB: agent populates DashboardData from adapters.
        let data = crate::ui::dashboard::DashboardData::default();
        print!("{}", crate::ui::dashboard::render(&data));
        Ok(())
    }
}
