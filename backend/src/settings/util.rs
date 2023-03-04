pub fn guess_smt(cpus: &Vec<crate::persist::CpuJson>) -> bool {
    let mut guess = true;
    for i in (0..cpus.len()).step_by(2) {
        guess &= cpus[i].online == cpus[i+1].online;
    }
    guess
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::persist::CpuJson;

    #[test]
    fn smt_guess_test() {
        let input = vec![
            cpu_with_online(true),
            cpu_with_online(true),
            cpu_with_online(true),
            cpu_with_online(true),
            cpu_with_online(true),
            cpu_with_online(true),
            cpu_with_online(true),
            cpu_with_online(true),
        ];
        assert_eq!(guess_smt(&input), true);

        let input = vec![
            cpu_with_online(true),
            cpu_with_online(false),
            cpu_with_online(true),
            cpu_with_online(false),
            cpu_with_online(true),
            cpu_with_online(false),
            cpu_with_online(true),
            cpu_with_online(false),
        ];
        assert_eq!(guess_smt(&input), false);
    }

    fn cpu_with_online(status: bool) -> CpuJson {
        CpuJson {
            online: status,
            clock_limits: None,
            governor: "schedutil".to_owned(),
        }
    }
}
