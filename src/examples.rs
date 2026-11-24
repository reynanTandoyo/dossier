//! The examples shown by `help <command>` (docs/design.md section 20, R-OUT-5). This file holds
//! the first group of commands: for each, two PowerShell lines and one Linux shell line. The data
//! is invented (project P-017, tasks such as T-042, an axial compressor). Every line starts with
//! `dossier <command>`.

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

    #[test]
    fn every_command_of_the_first_group_has_three_examples_that_start_with_its_name() {
        for name in FIRST_GROUP {
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
        for name in FIRST_GROUP {
            let examples = of(name).unwrap_or_else(|| panic!("no examples for {name}"));
            assert_ne!(examples[0], examples[1], "{name}");
            assert_ne!(examples[0], examples[2], "{name}");
            assert_ne!(examples[1], examples[2], "{name}");
        }
    }
}
