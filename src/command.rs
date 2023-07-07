use serenity::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub names: &'static [&'static str],
    pub desc: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub examples: &'static [&'static str],
}

#[derive(Clone)]
pub struct CommandGroupInfo {
    pub name: &'static str,
    pub commands: Vec<CommandInfo>,
    pub prefixes: &'static [&'static str],
}

pub struct CommandGroups {
    pub groups: Vec<CommandGroupInfo>,
}

pub struct CommandGroupsContainer;

impl TypeMapKey for CommandGroupsContainer {
    type Value = Arc<tokio::sync::Mutex<CommandGroups>>;
}

impl CommandGroupInfo {
    pub fn find_command(&self, name: &str) -> Option<&CommandInfo> {
        self.commands
            .iter()
            .find(|c: &&CommandInfo| c.names.iter().any(|&s| s == name))
    }
}

impl CommandGroups {
    pub fn find_group(&self, command_name: &str) -> Option<&CommandGroupInfo> {
        self.groups
            .iter()
            .find(|group| group.find_command(command_name).is_some())
    }

    pub fn find_command(&self, name: &str) -> Option<&CommandInfo> {
        if let Some(group) = self
            .groups
            .iter()
            .find(|group| group.find_command(name).is_some())
        {
            group.find_command(name)
        } else {
            None
        }
    }
}
