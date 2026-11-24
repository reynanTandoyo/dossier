//! The examples shown by `help <command>` (docs/design.md section 20, R-OUT-5). This file holds
//! the first and second groups of commands: for each, two PowerShell lines and one Linux shell
//! line. The data is invented (project P-017, tasks such as T-042, an axial compressor). Every
//! line starts with `dossier <command>`.

/// Two PowerShell examples and one for a Linux shell; None for a command without examples.
pub fn of(command: &str) -> Option<[&'static str; 3]> {
    let examples = match command {
        "init" => [
            "dossier init --code P-017",
            "dossier init --code P-017 --code-project",
            "dossier init --code P-017 --with-team ~/team",
        ],
        "where" => [
            "dossier where",
            "dossier where --json",
            "dossier where --dossier ~/work/P-017",
        ],
        "add" => [
            "dossier add finding --title 'Inlet profile was measured' --section 'Observation=The rig inlet profile is flat within 2 percent.' --task T-042",
            "dossier add lesson --title 'Steady runs overpredict stall margin' --field 'confidence=simulated' --section 'Statement=Steady mixing-plane runs converge where the transient run shows rotating stall.' --section 'Valid for=Transonic front stages, tip Mach number above 1.1.' --section 'Evidence=Steady and transient speedlines, report section 4.2.'",
            "dossier add lesson --title 'Tip mesh converges at two million cells' --from notes/lesson.md --field 'confidence=measured' --task T-043",
        ],
        "decide" => [
            "dossier decide 'Run the last three speedline points transient' --reason 'Steady runs converged where the transient run already showed rotating stall.' --task T-042",
            "dossier decide 'Use a finer mesh at the blade tip' --reason 'Tip losses only converge with it.' --rejected 'Keep the coarse mesh: the tip losses are off by 4 percent.' --machine 'axial compressor' --analysis 'steady RANS'",
            "dossier decide 'Run speedline points near stall transient too' --reason 'Steady runs overpredict the stall margin near the limit.' --replaces DEC-012",
        ],
        "found" => [
            "dossier found 'Halving the time step did not remove the oscillation' --evidence 'Runs 12 and 13, monitor plots in results' --task T-042",
            "dossier found 'Mesh refinement at the tip changed nothing' --evidence 'Runs 14 to 16' --dead-end --task T-042",
            "dossier found 'Residual rises again after 40 iterations' --evidence 'Run 21, residual plot' --later --software 'ExampleSolver 2406'",
        ],
        "learn" => [
            "dossier learn 'Steady mixing-plane runs overpredict stall margin near the stability limit' --valid-for 'Transonic front stages, tip Mach number above 1.1' --evidence 'Steady and transient speedlines, report section 4.2' --confidence simulated --task T-042",
            "dossier learn 'Transient runs match the rig at the last three speedline points' --valid-for 'Axial compressor at design speed' --evidence 'Rig report 3, speedline plot' --confidence measured --review 2027-06-01 --machine 'axial compressor'",
            "dossier learn 'Steady mixing-plane runs overpredict stall margin' --valid-for 'Transonic front stages, tip Mach number above 1.1' --evidence 'Report section 4.2 and rig report 3' --confidence measured --update LES-004",
        ],
        "assume" => [
            "dossier assume 'The inlet profile is uniform over the span' --reason 'No measured profile was delivered'",
            "dossier assume 'The working fluid behaves as air' --reason 'Its gas composition was not measured' --affects 'T-042, T-043' --task T-040",
            "dossier assume 'The wall roughness is the value of the datasheet' --reason 'The rig surface was not measured' --machine 'axial compressor' --tags rig",
        ],
        "propose" => [
            "dossier propose 'Run the last three speedline points transient' --reason 'Steady runs converged where the transient run already showed rotating stall.' --task T-042",
            "dossier propose 'Extend the transient rule to the rear stages' --reason 'The steady runs were never checked there.' --rejected 'Keep the rule for front stages only: it hides the rear stage question.' --machine 'axial compressor'",
            "dossier propose 'Run the 2406 solver with the new turbulence model' --reason 'It removed the stall offset in the test case.' --software 'ExampleSolver 2406' --task T-043",
        ],
        "accept" => [
            "dossier accept DEC-013",
            "dossier accept dec-014",
            "dossier accept DEC-015",
        ],
        "decline" => [
            "dossier decline DEC-014 --reason 'The tip mesh was checked in the last review and changes nothing.'",
            "dossier decline dec-016 --reason 'Rear stages are not part of the rig campaign.'",
            "dossier decline DEC-017 --reason 'The new turbulence model is not validated yet.'",
        ],
        "confirm" => [
            "dossier confirm ASM-003 --by 'measured on the rig, report 3'",
            "dossier confirm asm-004 --by 'ben, from the inlet survey of the rig'",
            "dossier confirm ASM-005 --by 'anna, from the roughness test, report 5'",
        ],
        "refute" => [
            "dossier refute ASM-003 --by 'anna, from the rig measurements'",
            "dossier refute asm-004 --by 'ben, from the pressure taps at station 2'",
            "dossier refute ASM-005 --by 'anna, from the roughness test'",
        ],
        "cleared" => [
            "dossier cleared DEC-012 --assumption ASM-003 --note 'the rig measured a flat inlet profile'",
            "dossier cleared results\\speedline.md --assumption ASM-003 --note 'the speedline run used the measured profile'",
            "dossier cleared FND-007 --assumption ASM-003 --note 'the probe data shows the profile is flat'",
        ],
        "set" => [
            "dossier set DEC-012 --field 'machine=axial compressor'",
            "dossier set LES-004 --section 'Valid for=Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.'",
            "dossier set LES-004 --field 'confidence=measured' --section 'Evidence=Rig report 4, section 2.'",
        ],
        "retitle" => [
            "dossier retitle DEC-012 'Run the last three speedline points transient'",
            "dossier retitle fnd-007 'Halving the time step did not remove the oscillation'",
            "dossier retitle LES-004 'Steady runs overpredict the stall margin'",
        ],
        "link" => [
            "dossier link DEC-012 FND-007",
            "dossier link ASM-003 LES-004",
            "dossier link DEC-012 LES-004",
        ],
        "edit" => [
            "dossier edit DEC-012",
            "dossier edit fnd-007",
            "dossier edit LES-004",
        ],
        "show" => [
            "dossier show DEC-012",
            "dossier show asm-003",
            "dossier show LES-004 --json",
        ],
        "list" => [
            "dossier list --kind decision --status approved",
            "dossier list --machine 'axial compressor' --task T-042",
            "dossier list --status draft --author ben",
        ],
        "search" => [
            "dossier search speedline",
            "dossier search 'stall margin' --full",
            "dossier search --similar LES-004",
        ],
        "history" => [
            "dossier history DEC-012",
            "dossier history T-042",
            "dossier history LES-004",
        ],
        "diff" => [
            "dossier diff DEC-012",
            "dossier diff DEC-012 2",
            "dossier diff LES-004 1",
        ],
        "restore" => [
            "dossier restore DEC-012 2",
            "dossier restore asm-003 1",
            "dossier restore LES-004 3",
        ],
        "log" => [
            "dossier log --person ben --since 2026-11-20",
            "dossier log --kind decision --event approved",
            "dossier log --person anna --event created --limit 20",
        ],
        "brief" => [
            "dossier brief",
            "dossier brief --task T-042",
            "dossier brief --budget 40",
        ],
        "status" => [
            "dossier status",
            "dossier status --milestone T-130",
            "dossier status --since last --keep",
        ],
        "review-task" => [
            "dossier review-task T-042",
            "dossier review-task T-043 --json",
            "dossier review-task T-044",
        ],
        "drafts" => [
            "dossier drafts",
            "dossier drafts --mine",
            "dossier drafts --json",
        ],
        "approve" => [
            "dossier approve DEC-013",
            "dossier approve DEC-013 LES-008",
            "dossier approve dec-014",
        ],
        "reject" => [
            "dossier reject LES-005 --reason 'The stall margin figure comes from one run only.'",
            "dossier reject dec-016",
            "dossier reject LES-006 --reason 'Its evidence is the report of another project.'",
        ],
        "supersede" => [
            "dossier supersede DEC-012 --by DEC-018",
            "dossier supersede les-004 --by LES-007",
            "dossier supersede dec-013 --by DEC-019",
        ],
        "retire" => [
            "dossier retire LES-004 --reason 'The rig was rebuilt, so the old runs are not comparable.'",
            "dossier retire ASM-003 --reason 'The inlet profile was measured in report 4.'",
            "dossier retire dec-015 --reason 'The campaign ended.'",
        ],
        "remove" => [
            "dossier remove FND-009 --yes",
            "dossier remove dec-016 --yes",
            "dossier remove ASM-004 --yes",
        ],
        "reopen" => [
            "dossier reopen DEC-013",
            "dossier reopen dec-014",
            "dossier reopen LES-004",
        ],
        "check" => [
            "dossier check",
            "dossier check --json",
            "dossier check --fix",
        ],
        "agents" => [
            "dossier agents install",
            "dossier agents install --yes",
            "dossier agents check",
        ],
        "pack" => [
            "dossier pack --to backup\\P-017.tar",
            "dossier pack --to backup\\P-017-team.tar --team",
            "dossier pack --to ~/backup/P-017.tar",
        ],
        "unpack" => [
            "dossier unpack backup\\P-017.tar --into restore\\P-017",
            "dossier unpack backup\\P-017-team.tar --into C:\\restore\\P-017",
            "dossier unpack ~/backup/P-017.tar --into ~/restore/P-017",
        ],
        "connect" => [
            "dossier connect anna@rig-server:/srv/dossiers/P-017 --code P-017",
            "dossier connect anna@rig-server:/srv/dossiers/P-017 --check",
            "dossier connect ben@rig-server:/srv/dossiers/P-017",
        ],
        "projects" => [
            "dossier projects",
            "dossier projects remove P-017",
            "dossier projects --json",
        ],
        "todo" => [
            "dossier todo show T-042",
            "dossier todo brief T-042",
            "dossier todo next --brief",
        ],
        "value" => [
            "dossier value set design.massflow '12.4 kg/s' --source 'Rig report 3, section 2' --key",
            "dossier value show design.massflow",
            "dossier value approve design.tip_mach",
        ],
        "values" => [
            "dossier values",
            "dossier values design",
            "dossier values --on 2026-11-20",
        ],
        "attach" => [
            "dossier attach T-042 results/speedline.csv --role result --note 'Speedline points of runs 14 to 16.'",
            "dossier attach T-042 models/compressor.cfd --role model",
            "dossier attach T-042 --moved models/old.cfd models/compressor.cfd",
        ],
        "detach" => [
            "dossier detach T-042 results/speedline_old.csv",
            "dossier detach T-042 setup\\inlet.cfg",
            "dossier detach T-043 models/compressor_v1.cfd",
        ],
        "reattach" => [
            "dossier reattach T-042 results/speedline.csv",
            "dossier reattach T-042 setup\\inlet.cfg",
            "dossier reattach T-043 models/compressor.cfd",
        ],
        "files" => [
            "dossier files --task T-042",
            "dossier files --task T-043 --role result",
            "dossier files speedline --json",
        ],
        "which" => [
            "dossier which results\\speedline.csv",
            "dossier which models\\compressor.cfd",
            "dossier which results/speedline.csv",
        ],
        "used" => [
            "dossier used ExampleSolver --version 2406 --task T-042 --model models/compressor.cfd --param 'time_step=0.001' --output results/speedline.csv --note 'Transient run, 40 iterations per step.'",
            "dossier used ExampleSolver --version 2406 --task T-043 --howto HOW-001 --deviations 'Tip mesh at two million cells instead of three.'",
            "dossier used ExampleSolver --version 2406 --task T-044 --output results/residuals.csv",
        ],
        "runs" => [
            "dossier runs --task T-042",
            "dossier runs --software ExampleSolver --model models/compressor.cfd",
            "dossier runs --compare USE-012 USE-014",
        ],
        "image" => [
            "dossier image add FND-007 figures/speedline.png --caption 'Speedline with the transient points.'",
            "dossier image remove FND-007 figures/speedline.png",
            "dossier image add LES-004 figures/tip_mesh.png --caption 'Tip mesh at two million cells.'",
        ],
        "received" => [
            "dossier received notes/rig_survey.pdf --from 'rig team' --date 2026-11-20 --task T-042",
            "dossier received data/inlet_survey.csv data/roughness.csv --from ben --note 'Inlet survey and roughness data.'",
            "dossier received notes/rig_survey.pdf --from anna --date 2026-11-21",
        ],
        "delivered" => [
            "dossier delivered reports/speedline.pdf --to 'customer office' --date 2026-11-22 --based-on T-042",
            "dossier delivered reports/speedline.pdf reports/inlet.pdf --to anna --based-on T-042 --note 'Final figures for the review.'",
            "dossier delivered reports/speedline.pdf --to 'rig team' --based-on FND-007",
        ],
        "exchange" => [
            "dossier exchange",
            "dossier exchange --received",
            "dossier exchange --delivered --json",
        ],
        "map" => [
            "dossier map",
            "dossier map set models 'Compressor models and their setup files.' --start models/README.md",
            "dossier map remove scripts/old",
        ],
        "hooks" => [
            "dossier hooks install",
            "dossier hooks install --yes",
            "dossier hooks uninstall --yes",
        ],
        "skill" => [
            "dossier skill install",
            "dossier skill install --yes",
            "dossier skill install --json",
        ],
        _ => return None,
    };
    Some(examples)
}

#[cfg(test)]
mod tests {
    use super::of;

    /// The commands of the first group.
    const FIRST_GROUP: [&str; 26] = [
        "init", "where", "add", "decide", "found", "learn", "assume", "propose", "accept",
        "decline", "confirm", "refute", "cleared", "set", "retitle", "link", "edit", "show",
        "list", "search", "history", "diff", "restore", "log", "brief", "status",
    ];

    /// The commands of the second group.
    const SECOND_GROUP: [&str; 31] = [
        "review-task",
        "drafts",
        "approve",
        "reject",
        "supersede",
        "retire",
        "remove",
        "reopen",
        "check",
        "agents",
        "pack",
        "unpack",
        "connect",
        "projects",
        "todo",
        "value",
        "values",
        "attach",
        "detach",
        "reattach",
        "files",
        "which",
        "used",
        "runs",
        "image",
        "received",
        "delivered",
        "exchange",
        "map",
        "hooks",
        "skill",
    ];

    /// The commands of both groups.
    fn all_groups() -> impl Iterator<Item = &'static str> {
        FIRST_GROUP.into_iter().chain(SECOND_GROUP)
    }

    #[test]
    fn every_command_of_both_groups_has_three_examples_that_start_with_its_name() {
        for name in all_groups() {
            let examples = of(name).unwrap_or_else(|| panic!("no examples for {name}"));
            for example in examples {
                let words: Vec<&str> = example.split_whitespace().collect();
                assert!(words.len() >= 2, "{name}: an example too short: {example}");
                assert_eq!(words[0], "dossier", "{name}: {example}");
                assert_eq!(words[1], name, "{name}: {example}");
            }
        }
    }

    #[test]
    fn the_examples_of_a_command_are_three_different_lines() {
        for name in all_groups() {
            let examples = of(name).unwrap_or_else(|| panic!("no examples for {name}"));
            assert_ne!(examples[0], examples[1], "{name}");
            assert_ne!(examples[0], examples[2], "{name}");
            assert_ne!(examples[1], examples[2], "{name}");
        }
    }
}
