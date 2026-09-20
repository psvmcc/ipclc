use assert_cmd::Command;

#[test]
fn ipv4_result_snapshot() {
    let output = Command::cargo_bin("ipclc")
        .unwrap()
        .args(["192.168.10.42/24", "--color", "never"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap());
}

#[test]
fn ipv6_result_snapshot() {
    let output = Command::cargo_bin("ipclc")
        .unwrap()
        .args(["2001:db8:1234:5678::42/64", "--color", "never"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap());
}
