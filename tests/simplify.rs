use glifparser::Glif;
use MFEKmath::simplify::simplify;

#[test]
fn test_() {
    let mut glif: Glif<()> =
        glifparser::read_from_filename("/home/@home/coolpp/github/pathops/1.glif").unwrap();
    // glif.outline = Some(simplify(glif.outline.unwrap(), false));
    glifparser::write_to_filename(&glif, "/home/@home/coolpp/github/pathops/2.glif").unwrap();
    // let mut glif: Glif<()> =
    //     glifparser::read_from_filename("/home/@home/coolpp/github/pathops/1.glif").unwrap();
    // glif.outline = Some(simplify(glif.outline.unwrap(), true));
    // glifparser::write_to_filename(&glif, "/home/@home/coolpp/github/pathops/2_final.glif").unwrap();
    let mut glif: Glif<()> =
        glifparser::read_from_filename("/home/@home/coolpp/github/pathops/3.glif").unwrap();
    glif.outline = Some(simplify(glif.outline.unwrap(), true));
    glifparser::write_to_filename(&glif, "/home/@home/coolpp/github/pathops/4.glif").unwrap();
    // let mut glif: Glif<()> = glifparser::read_from_filename(
    //     "/home/@home/coolpp/github/pathops/A_SourceCodePro-BlackIt.glif",
    // )
    // .unwrap();
    // glif.outline = Some(simplify(glif.outline.unwrap(), false));
    // glifparser::write_to_filename(
    //     &glif,
    //     "/home/@home/coolpp/github/pathops/A_SourceCodePro-BlackIt_final.glif",
    // )
    // .unwrap();
    let mut glif: Glif<()> = glifparser::read_from_filename(
        "/home/@home/coolpp/github/pathops/AE_SourceCodePro-BlackIt.glif",
    )
    .unwrap();
    glif.outline = Some(simplify(glif.outline.unwrap(), false));
    glifparser::write_to_filename(
        &glif,
        "/home/@home/coolpp/github/pathops/AE_SourceCodePro-BlackIt_final.glif",
    )
    .unwrap();
    assert!(false);
}
