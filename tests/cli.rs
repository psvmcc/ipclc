use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prints_ipv4_network_information() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .arg("192.168.10.42/24")
        .assert()
        .success()
        .stdout(predicate::str::contains("Network:"))
        .stdout(predicate::str::contains("192.168.10.0"));
}

#[test]
fn prints_total_address_power_for_ipv4() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .arg("192.168.10.42/24")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total addresses:  256 (2^8)"));
}

#[test]
fn accepts_cidr_with_dotted_ipv4_mask() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .arg("192.168.10.42/255.255.255.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("192.168.10.0"));
}

#[test]
fn prints_ipv6_json() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["2001:db8::42/64", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"version\": \"6\""));
}

#[test]
fn prints_ipv6_short_and_expanded_forms_together() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .arg("2001:db8::42/64")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Address:          2001:db8::42 (2001:0db8:0000:0000:0000:0000:0000:0042)",
        ))
        .stdout(predicate::str::contains(
            "Total addresses:  18446744073709551616 (2^64)",
        ))
        .stdout(predicate::str::contains("Expanded address:").not());
}

#[test]
fn rejects_invalid_input() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .arg("192.168.1.1/33")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid prefix"));
}

#[test]
fn splits_ipv4_network() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["192.168.1.0/24", "--split", "26"])
        .assert()
        .success()
        .stdout(predicate::str::contains("192.168.1.0/26"))
        .stdout(predicate::str::contains("192.168.1.192/26"));
}

#[test]
fn checks_network_membership() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["10.0.0.0/8", "--contains", "10.1.2.3"])
        .assert()
        .success()
        .stdout("yes\n");
}

#[test]
fn rejects_mismatched_contains_address_family() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["10.0.0.0/8", "--contains", "2001:db8::1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("same IP version"));
}

#[test]
fn rejects_conflicting_output_operations() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["10.0.0.0/8", "--json", "--split", "16"])
        .assert()
        .failure();
}

#[test]
fn clap_requires_address_except_for_completions() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .assert()
        .code(2)
        .stderr(predicate::str::contains("required"));
}

#[test]
fn prints_reverse_dns() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["192.0.2.10/32", "--reverse-dns"])
        .assert()
        .success()
        .stdout("10.2.0.192.in-addr.arpa.\n");
}

#[test]
fn prints_reverse_zone() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["192.168.10.0/24", "--reverse-zone"])
        .assert()
        .success()
        .stdout("10.168.192.in-addr.arpa.\n");
}

#[test]
fn checks_overlap() {
    Command::cargo_bin("ipclc")
        .unwrap()
        .args(["10.0.0.0/24", "--overlaps", "10.0.0.128/25"])
        .assert()
        .success()
        .stdout("yes\n");
}
