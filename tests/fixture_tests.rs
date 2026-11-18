//! The sample drawings under `fixtures/drawings/`, which the example and the custom host use:
//! each is accepted in its catalog, as `genui` checks it, before and after its patches.

mod common;

use adk_surface::genui::{DataModel, Validator};
use common::{draft, fixtures};

#[test]
fn every_sample_drawing_is_accepted_in_its_catalog_before_and_after_its_patches() {
    let fixtures = fixtures();
    assert_eq!(
        fixtures.len(),
        11,
        "six studio and insight drawings, five spatial ones"
    );
    for fixture in fixtures {
        let validator = Validator::new(fixture.catalog);
        let mut draft = draft(&fixture.screen);
        match validator.validate(&draft) {
            Err(refusal) => panic!("{} is refused: {}", fixture.name, refusal.technical()),
            // The dark-pattern warnings' false positives: none on a plain dashboard.
            Ok(validated) => assert!(
                validated.warnings.is_empty(),
                "{} raises warnings: {:?}",
                fixture.name,
                validated.warnings
            ),
        }
        for (index, patch) in fixture.patches.iter().enumerate() {
            let mut model = DataModel::new(draft.data_model.clone());
            for change in patch["data"].as_array().unwrap() {
                model
                    .apply(change["path"].as_str(), Some(change["value"].clone()))
                    .unwrap();
            }
            draft.data_model = model.value().clone();
            if let Err(refusal) = validator.validate(&draft) {
                panic!(
                    "{} patch {index} is refused: {}",
                    fixture.name,
                    refusal.technical()
                );
            }
        }
    }
}

#[test]
fn a_studio_drawing_needs_the_studio_catalog() {
    for fixture in fixtures()
        .into_iter()
        .filter(|fixture| fixture.catalog.id() == adk_ui::catalogs::STUDIO_ID)
    {
        assert!(
            Validator::new(adk_ui::catalogs::insight())
                .validate(&draft(&fixture.screen))
                .is_err(),
            "{} uses studio components",
            fixture.name
        );
    }
}
