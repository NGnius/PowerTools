pub fn guess_smt(cpus: &Vec<crate::persist::CpuJson>) -> bool {
    let mut guess = true;
    for i in (0..cpus.len()).step_by(2) {
        guess &= cpus[i].online == cpus[i+1].online;
    }
    guess
}
