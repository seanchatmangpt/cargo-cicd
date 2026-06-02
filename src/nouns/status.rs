use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs};
use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};

pub struct StatusNoun;
impl StatusNoun { pub fn new() -> Self { Self } }
