use crate::global_state::ActiveVulkanState;

pub trait Seal {}

impl Seal for ActiveVulkanState {}

impl Seal for () {}
