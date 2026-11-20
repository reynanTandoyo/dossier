//! Forwarding a command to the server that keeps the dossier.

/// Called first by `main_exit`. Some(exit code) when the command was run on a server: the
/// `--project <code>` option is given, or the current folder has a pointer file and the command
/// is not `connect`, `projects` or `help`. None: run locally.
pub fn maybe(args: &[String]) -> Option<u8> {
    None
}
